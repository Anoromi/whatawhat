pub mod analysis;
pub mod sliding_grouping;

use std::{future, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use futures::{stream, Stream, StreamExt};
use tracing::error;

use crate::daemon::storage::{entities::UsageIntervalEntity, record_storage::RecordStorage};

pub struct ExtractConfig {
    pub end: DateTime<Utc>,
    pub start: DateTime<Utc>,
}

impl ExtractConfig {
    fn filter(&self, entity: UsageIntervalEntity) -> Option<UsageIntervalEntity> {
        entity.clamp(self.start, self.end)
    }
}

pub fn extract_between(
    storage: impl RecordStorage,
    print_config: ExtractConfig,
) -> impl Stream<Item = Result<UsageIntervalEntity>> {
    let storage = Arc::new(storage);
    let start = print_config.start;
    let end = print_config.end;


    let date_iteration = date_range(start.date_naive(), end.date_naive());

    let files = date_iteration
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
        .filter_map(move |v| future::ready(v.map(|v| print_config.filter(v)).transpose()));

    result
}

fn date_range(start: NaiveDate, end: NaiveDate) -> impl Stream<Item = NaiveDate> {
    stream::unfold(
        (start, end),
        |(mut current, end)| {
            future::ready({
                if current <= end {
                    let last_current = current;
                    current = current.succ_opt().expect("End of time should never happen");
                    Some(((last_current), (current, end)))
                } else {
                    None
                }
            })
        },
    )
}

