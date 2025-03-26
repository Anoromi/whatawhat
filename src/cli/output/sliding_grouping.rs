use std::{
    fmt::{Debug, Display},
    pin::Pin,
};

use anyhow::Result;
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Timelike, Utc};
use clap::ValueEnum;
use futures::{Stream, StreamExt};
use now::DateTimeNow;
use tracing::{instrument, trace};

use crate::daemon::storage::entities::UsageIntervalEntity;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeOption {
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
}

impl Display for TimeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeOption::Weeks => write!(f, "weeks"),
            TimeOption::Days => write!(f, "days"),
            TimeOption::Hours => write!(f, "hours"),
            TimeOption::Minutes => write!(f, "minutes"),
            TimeOption::Seconds => write!(f, "seconds"),
        }
    }
}

/// Intended for creating simple to understand intervals.
/// The reason this struct was used instead of Duration is to introduce
/// compile time safety into [clean_time_start]. Also it's easier for user to
/// type when writing cli commands.
#[derive(Debug, Clone, Copy)]
pub struct SlidingInterval {
    duration: u32,
    time: TimeOption,
}

impl SlidingInterval {
    pub fn new_opt(duration: u32, time: TimeOption) -> Option<Self> {
        match time {
            // These values are reasonable limitations of what should be allowed
            TimeOption::Hours if duration < 24 => Some(Self { duration, time }),
            TimeOption::Minutes if duration < 60 => Some(Self { duration, time }),
            TimeOption::Seconds if duration < 60 => Some(Self { duration, time }),
            TimeOption::Days if duration < 7 => Some(Self { duration, time }),
            TimeOption::Weeks if duration < 2 => Some(Self { duration, time }),
            TimeOption::Hours
            | TimeOption::Minutes
            | TimeOption::Seconds
            | TimeOption::Days
            | TimeOption::Weeks => None,
        }
    }

    pub fn duration(&self) -> u32 {
        self.duration
    }

    pub fn time(&self) -> &TimeOption {
        &self.time
    }

    pub fn as_duration(&self) -> Duration {
        match self.time() {
            TimeOption::Hours => Duration::hours(self.duration as i64),
            TimeOption::Minutes => Duration::minutes(self.duration as i64),
            TimeOption::Seconds => Duration::seconds(self.duration as i64),
            TimeOption::Weeks => Duration::weeks(self.duration as i64),
            TimeOption::Days => Duration::days(self.duration as i64),
        }
    }
}

/// Creates a start of a timeline that's easier to comprehend.
/// It's easier to interpret if your timeline starts at 01:10:00 than at 01:11:32.
pub fn clean_time_start<Tz: TimeZone>(
    rough_start: DateTime<Tz>,
    scale: &SlidingInterval,
) -> DateTime<Tz> {
    match scale.time() {
        TimeOption::Weeks => rough_start.beginning_of_week(),
        TimeOption::Days => rough_start.beginning_of_day(),
        TimeOption::Hours => {
            let lower_bound = rough_start
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();

            let duration = rough_start.clone() - lower_bound.clone();
            let remainder = duration.num_hours() as u32 % scale.duration();
            lower_bound
                .clone()
                .with_hour(rough_start.clone().hour() - remainder)
                .unwrap()
        }
        TimeOption::Minutes => {
            let lower_bound = rough_start
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();

            let duration = rough_start.clone() - lower_bound.clone();
            let remainder = duration.num_minutes() as u32 % scale.duration();
            lower_bound
                .with_minute(rough_start.minute() - remainder)
                .unwrap()
        }
        TimeOption::Seconds => {
            let lower_bound = rough_start
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();

            let duration = rough_start.clone() - lower_bound.clone();
            let remainder = duration.num_seconds() as u32 % scale.duration();
            lower_bound
                .with_second(rough_start.second() - remainder)
                .unwrap()
        }
    }
}

#[instrument(skip(values, analyzer))]
pub async fn sliding_interval_grouping<T, Tz: TimeZone>(
    values: impl Stream<Item = Result<UsageIntervalEntity>>,
    scale: SlidingInterval,
    mut analyzer: impl FnMut(Vec<UsageIntervalEntity>) -> T,
) -> Result<Vec<(DateTime<Utc>, Option<T>)>>
where
    DateTime<Tz>: From<DateTime<Utc>>,
{
    let mut values = std::pin::pin!(values);

    let Some(first) = values.next().await.transpose()? else {
        return Ok(vec![]);
    };

    // we only need timezone to get the point in time from which to run interval grouping.
    let rough_start = DateTime::<Tz>::from(first.start);
    let mut collapse_start = clean_time_start(rough_start, &scale).to_utc();

    let duration = scale.as_duration();
    let mut collapse_end = collapse_start + duration;

    let move_time = |previous_end: DateTime<Utc>| {
        let start = previous_end;
        let mut end = start + duration;
        // days are supposed to be self enclosed. There should be no overflow from one day to
        // another.
        let local_start = DateTime::<Tz>::from(start);
        let local_end = DateTime::<Tz>::from(end);
        if local_start.date_naive() != local_end.date_naive() {
            end = local_end.with_time(NaiveTime::MIN).unwrap().to_utc()
        }
        (start, end)
    };

    let mut collected: Vec<(DateTime<Utc>, Option<T>)> = vec![];

    let mut backlog: Option<UsageIntervalEntity> = Some(first);
    trace!("Start end {collapse_start} {collapse_end}");
    while let GroupResult::Values {
        backlog: updated_backlog,
        data,
    } = sliding_group_next(
        &mut values,
        backlog,
        collapse_start,
        collapse_end,
        &mut analyzer,
    )
    .await?
    {
        backlog = updated_backlog;
        collected.push((collapse_start, data));
        (collapse_start, collapse_end) = move_time(collapse_end);

        trace!("Start end {collapse_start} {collapse_end}");
    }

    return Ok(collected);
}

enum GroupResult<T> {
    End,
    Values {
        backlog: Option<UsageIntervalEntity>,
        data: Option<T>,
    },
}

#[instrument(skip(stream, analyzer))]
async fn sliding_group_next<S: Stream<Item = Result<UsageIntervalEntity>>, T, Tz: TimeZone>(
    stream: &mut Pin<&mut S>,
    mut backlog: Option<UsageIntervalEntity>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    analyzer: &mut impl FnMut(Vec<UsageIntervalEntity>) -> T,
) -> Result<GroupResult<T>>
where
    DateTime<Tz>: From<DateTime<Utc>>,
{
    let mut collected: Vec<UsageIntervalEntity> = vec![];

    while let Some(usage_interval) = take_or_poll_ok(&mut backlog, stream).await? {
        if usage_interval.end() < start {
            continue;
        }

        trace!(
            "{} {} {}",
            usage_interval.process_name,
            usage_interval.start,
            usage_interval.end()
        );
        match usage_interval.split_by(end) {
            (None, None) => unreachable!(),
            (None, Some(after)) => {
                return Ok(GroupResult::Values {
                    backlog: Some(after),
                    data: Some(analyzer(collected)),
                });
            }
            (Some(before), Some(after)) => {
                collected.push(before);
                return Ok(GroupResult::Values {
                    backlog: Some(after),
                    data: Some(analyzer(collected)),
                });
            }
            (Some(before), None) => {
                collected.push(before);
            }
        }
    }

    if collected.is_empty() {
        Ok(GroupResult::End)
    } else {
        Ok(GroupResult::Values {
            backlog: None,
            data: Some(analyzer(collected)),
        })
    }
}

async fn take_or_poll_ok<T>(
    value: &mut Option<T>,
    stream: &mut Pin<&mut impl Stream<Item = Result<T>>>,
) -> Result<Option<T>> {
    if let Some(v) = value.take() {
        Ok(Some(v))
    } else {
        stream.next().await.transpose()
    }
}

#[cfg(test)]
mod clean_time_tests {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Utc};

    use super::{SlidingInterval, clean_time_start};

    const TEST_DATE: NaiveDate = NaiveDate::from_ymd_opt(2024, 4, 5).unwrap();

    fn test_time(time: NaiveTime, scale: &SlidingInterval, expected: NaiveTime) {
        let offset = Utc.fix();
        let start = clean_time_start(
            offset.from_utc_datetime(&NaiveDateTime::new(TEST_DATE, time)),
            scale,
        );
        assert_eq!(start.time(), expected, "{time} {scale:?} {expected}");
    }

    #[test]
    fn test_clean_time_hours() {
        test_time(
            NaiveTime::from_hms_opt(12, 24, 54).unwrap(),
            &SlidingInterval::new_opt(2, super::TimeOption::Hours).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(11, 56, 5).unwrap(),
            &SlidingInterval::new_opt(1, super::TimeOption::Hours).unwrap(),
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            &SlidingInterval::new_opt(1, super::TimeOption::Hours).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
            &SlidingInterval::new_opt(5, super::TimeOption::Hours).unwrap(),
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        );
    }

    #[test]
    fn test_clean_time_minutes() {
        test_time(
            NaiveTime::from_hms_opt(4, 24, 54).unwrap(),
            &SlidingInterval::new_opt(15, super::TimeOption::Minutes).unwrap(),
            NaiveTime::from_hms_opt(4, 15, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(11, 56, 5).unwrap(),
            &SlidingInterval::new_opt(10, super::TimeOption::Minutes).unwrap(),
            NaiveTime::from_hms_opt(11, 50, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(16, 5, 0).unwrap(),
            &SlidingInterval::new_opt(1, super::TimeOption::Minutes).unwrap(),
            NaiveTime::from_hms_opt(16, 5, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
            &SlidingInterval::new_opt(5, super::TimeOption::Minutes).unwrap(),
            NaiveTime::from_hms_opt(23, 55, 0).unwrap(),
        );
    }

    #[test]
    fn test_clean_time_seconds() {
        test_time(
            NaiveTime::from_hms_opt(4, 24, 4).unwrap(),
            &SlidingInterval::new_opt(5, super::TimeOption::Seconds).unwrap(),
            NaiveTime::from_hms_opt(4, 24, 0).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(4, 24, 59).unwrap(),
            &SlidingInterval::new_opt(15, super::TimeOption::Seconds).unwrap(),
            NaiveTime::from_hms_opt(4, 24, 45).unwrap(),
        );

        test_time(
            NaiveTime::from_hms_opt(4, 24, 45).unwrap(),
            &SlidingInterval::new_opt(15, super::TimeOption::Seconds).unwrap(),
            NaiveTime::from_hms_opt(4, 24, 45).unwrap(),
        );
    }
}

#[cfg(test)]
mod sliding_groupnig_test {
    use std::convert::identity;

    use anyhow::Result;
    use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
    use futures::stream;
    use tokio_stream::StreamExt;

    use crate::{
        cli::output::sliding_grouping::{SlidingInterval, TimeOption, sliding_interval_grouping},
        daemon::storage::entities::UsageIntervalEntity,
        utils::logging::TEST_LOGGING,
    };

    const TEST_DATE: NaiveDate = NaiveDate::from_ymd_opt(2024, 4, 5).unwrap();
    const TEST_DATE_TIME: NaiveDateTime =
        NaiveDateTime::new(TEST_DATE, NaiveTime::from_hms_opt(12, 0, 0).unwrap());

    #[tokio::test]
    async fn sliding_interval_grouping_basic() -> Result<()> {
        *TEST_LOGGING;

        let entity_a = UsageIntervalEntity {
            window_name: "entity a".into(),
            process_name: "process a".into(),
            start: Utc.from_utc_datetime(&TEST_DATE_TIME),
            duration: Duration::zero(),
            afk: false,
        };
        let entity_b = UsageIntervalEntity {
            window_name: "entity b".into(),
            process_name: "process b".into(),
            start: Utc.from_utc_datetime(&TEST_DATE_TIME),
            duration: Duration::zero(),
            afk: false,
        };
        let mut values = vec![];

        let mut current_time = Utc.from_utc_datetime(&TEST_DATE_TIME);
        for _ in 0..5 {
            values.push(
                entity_a
                    .clone()
                    .with_start(current_time)
                    .with_duration(Duration::seconds(5)),
            );
            current_time += Duration::seconds(5);
            values.push(
                entity_b
                    .clone()
                    .with_start(current_time)
                    .with_duration(Duration::seconds(5)),
            );
            current_time += Duration::seconds(5);
        }

        let stream = stream::iter(values);

        let grouping = sliding_interval_grouping::<_, Utc>(
            stream.map(Ok),
            SlidingInterval::new_opt(30, TimeOption::Seconds).unwrap(),
            identity,
        )
        .await?;

        assert_eq!(
            vec_duration(grouping[0].1.as_ref().unwrap().iter()).num_seconds(),
            30
        );
        assert_eq!(
            vec_duration(grouping[1].1.as_ref().unwrap().iter()).num_seconds(),
            20
        );

        Ok(())
    }

    fn vec_duration<'a>(values: impl Iterator<Item = &'a UsageIntervalEntity>) -> Duration {
        values.fold(Duration::zero(), |ac, next| ac + next.duration)
    }
}
