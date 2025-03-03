use std::{future, path::PathBuf, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use futures::{stream, Stream, StreamExt};
use tracing::error;

use crate::daemon::storage::{
    record_storage::RecordStorage, entities::UsageIntervalEntity,
};

pub struct PrintConfig {
    pub min_duration: TimeDelta,
    pub with_afk: bool,
}

impl PrintConfig {
    fn should_display(&self, entity: &UsageIntervalEntity) -> bool {
        (self.with_afk || !entity.afk) && entity.duration() > self.min_duration
    }
}

pub fn extract_between(
    storage: impl RecordStorage,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    print_config: PrintConfig,
) -> impl Stream<Item = Result<UsageIntervalEntity>> + Unpin {
    let storage = Arc::new(storage);

    let pipe = tokio_stream::iter(DateRangeIter::new(start.date_naive(), end.date_naive()));

    let files = pipe
        .map(move |day| {
            let storage = storage.clone();
            async move { (day, storage.get_data_for(day).await) }
        })
        .buffered(8);

    files
        .flat_map(|(day, data)| match data {
            Ok(data) => stream::iter(data).map(Ok).boxed(),
            Err(e) => {
                error!("Failed to process file {day} {e}");
                stream::once(future::ready(Err(e))).boxed()
            }
        })
        // .filter(move |v| {
        //     future::ready(match v {
        //         Ok(event) => print_config.should_display(event),
        //         Err(_) => true,
        //     })
        // })
}

struct DateRangeIter {
    current: NaiveDate,
    end: NaiveDate,
}

impl DateRangeIter {
    fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for DateRangeIter {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let current = self.current;
            self.current = self
                .current
                .succ_opt()
                .expect("End of time should never happen");
            Some(current)
        } else {
            None
        }
    }
}
