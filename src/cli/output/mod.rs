pub mod analysis;
pub mod sliding_grouping;

use std::{future, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use futures::{stream, Stream, StreamExt};
use tracing::error;

use crate::daemon::storage::{entities::UsageIntervalEntity, record_storage::RecordStorage};

pub struct PrintConfig {
    pub with_afk: bool,
    pub end: DateTime<Utc>,
    pub start: DateTime<Utc>,
}

impl PrintConfig {
    fn filter(&self, entity: UsageIntervalEntity) -> Option<UsageIntervalEntity> {
        if !self.with_afk && entity.afk  {
            return None
        }
        entity.filter_by_interval(self.start, self.end)
    }
}

pub fn extract_between(
    storage: impl RecordStorage,
    print_config: PrintConfig,
) -> impl Stream<Item = Result<UsageIntervalEntity>> {
    let storage = Arc::new(storage);
    let start = print_config.start;
    let end = print_config.end;

    let pipe = tokio_stream::iter(DateRangeIter::new(start.date_naive(), end.date_naive()));

    let files = pipe
        .map(move |day| {
            let storage = storage.clone();
            async move { (day, storage.get_data_for(day).await) }
        })
        .buffered(8);

    let result = files
        .flat_map(|(day, data)| match data {
            Ok(data) => stream::iter(data).map(Ok).boxed(),
            Err(e) => {
                error!("Failed to process file {day} {e}");
                stream::once(future::ready(Err(e))).boxed()
            }
        })
        .filter_map(move |v| {
            future::ready(
                v.map(|v| print_config.filter(v)).transpose() 
            )
        });

    result
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
