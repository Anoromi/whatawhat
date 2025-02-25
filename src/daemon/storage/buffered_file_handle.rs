
use std::time::{Duration };

use tokio::time::Instant;

use super::{entities::UsageRecordEntity, record_storage::RecordFileHandle};

struct BufFileHandle<T> {
    handle: T,
    saved_data: Vec<UsageRecordEntity>,
    sync_threshold: Duration,
    last_sync: Option<Instant>
}

impl<T: RecordFileHandle> BufFileHandle<T> {
}

impl<T: RecordFileHandle> RecordFileHandle for BufFileHandle<T> {
    async fn append(
        &mut self,
        usage_records: Vec<UsageRecordEntity>,
    ) -> anyhow::Result<()> {
        // usage_records.
        todo!()
    }

    fn get_date(&self) -> chrono::NaiveDate {
        self.handle.get_date()
    }
}
