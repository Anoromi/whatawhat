use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Duration, Local};
use chrono_english::parse_date_string;
use clap::{CommandFactory, Parser, ValueEnum};
use futures::Stream;
use now::DateTimeNow;

use crate::{
    daemon::storage::{entities::UsageIntervalEntity, record_storage::RecordStorageImpl},
    utils::{
        percentage::{Percentage, duration_percentage},
        time::next_day_start,
    },
};

use super::{
    Args, create_application_default_path,
    output::{
        self,
        analysis::{analyze_processes, analyze_windows},
        extract_between,
        sliding_grouping::{SlidingInterval, TimeOption, sliding_interval_grouping},
    },
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DateStyle {
    Uk,
    Us,
}

impl From<DateStyle> for chrono_english::Dialect {
    fn from(value: DateStyle) -> Self {
        match value {
            DateStyle::Uk => Self::Uk,
            DateStyle::Us => Self::Us,
        }
    }
}

impl Display for DateStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateStyle::Uk => write!(f, "uk"),
            DateStyle::Us => write!(f, "us"),
        }
    }
}

#[derive(Debug, Parser)]
pub struct TimelineCommand {
    #[arg(
        long = "start",
        short,
        help = "Start of the range. Examples are \"yesterday\", \"1 hour ago\", \"15/03/2025\", \"12:00 16/03/2025\", \"12 AM 16/03/2025\""
    )]
    start_date: Option<String>,
    #[arg(
        long = "end",
        short,
        help = "End of the range. Examples are \"yesterday\", \"1 hour ago\", \"15/03/2025\", \"12:00 16/03/2025\", \"12 AM 16/03/2025\""
    )]
    end_date: Option<String>,
    #[arg(long, default_value_t = DateStyle::Uk, help = "Style of dates used during parsing. For Uk it's day/month/year. For Us it's month/day/year")]
    date_style: DateStyle,
    #[arg(
        long = "days",
        default_value_t = false,
        help = "Take inputs as whole days. For example if start and end are both 15/03/2025 this option allows to extract the whole day"
    )]
    treat_as_days: bool,
    #[command(flatten)]
    interval: PrintInterval,
    #[arg(short = 'p', long = "percentage", help = "Filter apps to have at least specified percentage", default_value_t = Percentage::new_opt(1.).unwrap()) ]
    min_percentage: Percentage,
    #[arg(short, long = "processes", help = "Ignore window names")]
    use_processes: bool,

    #[arg(
        short,
        long,
        help = "Include time afk. Person is considered afk after 2 minutes of idle time."
    )]
    afk: bool,
}

#[derive(Parser, Debug)]
struct DaemonParams {
    #[arg(long)]
    dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, clap::Args)]
#[command(flatten_help = true)]
pub struct PrintInterval {
    #[arg(
        short,
        help = "Duration of interval. Combines with option to create interval -d 15 -o minutes"
    )]
    duration: u32,
    #[arg(
        short,
        help = "Time option of interval. Combines with option to create interval -d 15 -o minutes"
    )]
    option: TimeOption,
}

const DEFAULT_PRINTED_INTERVALS: i32 = 10;

/// Command to process `timeline` command. Timeline command is intended to provide information
/// about user activity from `start_date` to `end_date`.
pub async fn process_timeline_command(
    TimelineCommand {
        start_date,
        end_date,
        date_style,
        interval,
        treat_as_days,
        min_percentage,
        use_processes,
        afk,
    }: TimelineCommand,
) -> Result<()> {
    let ParamParseResult {
        interval,
        start,
        end,
        show_time,
    } = match parse_values(start_date, end_date, date_style, interval, treat_as_days) {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    let application = RecordStorageImpl::new(create_application_default_path()?.join("records"))?;

    // We create a stream representing timeline between dates.
    let results = extract_between(
        application,
        output::ExtractConfig {
            start: start.into(),
            end: end.into(),
        },
    );

    if use_processes {
        print_processes_grouping(interval, min_percentage, afk, show_time, results).await?;
    } else {
        print_window_grouping(interval, min_percentage, afk, show_time, results).await?;
    }
    Ok(())
}

struct ParamParseResult {
    interval: SlidingInterval,
    start: DateTime<Local>,
    end: DateTime<Local>,
    show_time: bool,
}

/// Also provides sensible defaults for `timeline` command.
fn parse_values(
    start_date: Option<String>,
    end_date: Option<String>,
    date_style: DateStyle,
    interval: PrintInterval,
    treat_as_days: bool,
) -> Result<ParamParseResult> {
    let treat_as_days = treat_as_days || interval.option <= TimeOption::Days;
    let Some(interval) = SlidingInterval::new_opt(interval.duration, interval.option) else {
        return Err(Args::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                format!(
                    "Can't create an interval using {} and {}",
                    interval.duration, interval.option
                ),
            )
            .into());
    };
    let now = Local::now();
    let dialect: chrono_english::Dialect = date_style.into();
    let mut start = match start_date.map(|s| parse_date_string(&s, now, dialect)) {
        Some(Ok(v)) => v.with_timezone(&Local),
        Some(Err(e)) => {
            return Err(Args::command()
                .error(
                    clap::error::ErrorKind::ValueValidation,
                    format!("Failed to valiate start date {e}"),
                )
                .into());
        }
        None => Local::now() - interval.as_duration() * DEFAULT_PRINTED_INTERVALS,
    };
    let mut end = match end_date.map(|s| parse_date_string(&s, now, dialect)) {
        Some(Ok(v)) => v.with_timezone(&Local),
        Some(Err(e)) => {
            return Err(Args::command()
                .error(
                    clap::error::ErrorKind::ValueValidation,
                    format!("Failed to valiate end date {e}"),
                )
                .into());
        }
        None => Local::now(),
    };
    if treat_as_days {
        start = start.beginning_of_day();
        end = next_day_start(end);
    }

    let show_time = *interval.time() > TimeOption::Days;

    Ok(ParamParseResult {
        interval,
        start,
        end,
        show_time,
    })
}

// Realistically print_processes_grouping and print_window_grouping can be combined, however this
// will require abstractions for just 80 lines of code.

async fn print_processes_grouping(
    interval: SlidingInterval,
    min_percentage: Percentage,
    afk: bool,
    show_time: bool,
    results: impl Stream<Item = std::result::Result<UsageIntervalEntity, anyhow::Error>>,
) -> Result<()> {
    let intervals = sliding_interval_grouping::<_, Local>(results, interval, |v| {
        analyze_processes(v, min_percentage, afk)
    })
    .await?;
    for (time, value) in intervals {
        let Some((analyzed, computer_on_duration)) = value else {
            continue;
        };

        let time = time.with_timezone(&Local);

        let time_format = if show_time { "%x %H:%M:%S" } else { "%x" };

        if !analyzed.is_empty() {
            for entry in analyzed {
                println!(
                    "{}\t{}%\t{}\t{}",
                    time.format(time_format),
                    *duration_percentage(entry.duration, computer_on_duration) as i32,
                    format_duration(entry.duration),
                    clean_process_name(&entry.process_name)
                );
            }
            println!();
        }
    }
    Ok(())
}

async fn print_window_grouping(
    interval: SlidingInterval,
    min_percentage: Percentage,
    afk: bool,
    show_time: bool,
    results: impl Stream<Item = std::result::Result<UsageIntervalEntity, anyhow::Error>>,
) -> Result<()> {
    let intervals = sliding_interval_grouping::<_, Local>(results, interval, |v| {
        analyze_windows(v, min_percentage, afk)
    })
    .await?;
    for (time, value) in intervals {
        let Some((analyzed, computer_on_duration)) = value else {
            continue;
        };

        let time = time.with_timezone(&Local);

        let time_format = if show_time { "%x %H:%M:%S" } else { "%x" };

        if !analyzed.is_empty() {
            for entry in analyzed {
                println!(
                    "{}\t{}%\t{}\t{}\t{}",
                    time.format(time_format),
                    *duration_percentage(entry.duration, computer_on_duration) as i32,
                    format_duration(entry.duration),
                    clean_process_name(&entry.process_name),
                    entry.window_name
                );
            }
            println!();
        }
    }
    Ok(())
}

fn format_duration(v: Duration) -> String {
    if v.num_hours() > 0 {
        format!(
            "{}h{}m{}s",
            v.num_hours(),
            v.num_minutes() % 60,
            v.num_seconds() % 60
        )
    } else if v.num_minutes() > 0 {
        format!("{}m{}s", v.num_minutes() % 60, v.num_seconds() % 60)
    } else {
        format!("{}s", v.num_seconds() % 60)
    }
}

fn clean_process_name(value: &str) -> String {
    PathBuf::from(value)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| value.to_string())
}
