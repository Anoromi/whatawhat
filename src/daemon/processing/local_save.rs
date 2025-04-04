use anyhow::Result;

use crate::{
    daemon::storage::{
        entities::UsageRecordEntity,
        record_event::RecordEvent,
        record_storage::{RecordFileHandle, RecordStorage},
    },
    utils::clock::Clock,
};

use super::module::EventProcessor;

/// Represents saving module. Saving module main goal is to bridge
/// [ProcessingModule](super::ProcessingModule) and [RecordStorage]
/// In the future it might also combine multiple savers, like color storage + record storage.
pub struct LocalSaver<R: RecordStorage> {
    records_storage: R,
    current_handle: Option<R::RecordFile>,
    date_provider: Box<dyn Clock>,
}

impl<R: RecordStorage> LocalSaver<R> {
    pub fn new(records_storage: R, date_provider: Box<dyn Clock>) -> Self {
        Self {
            records_storage,
            current_handle: None,
            date_provider,
        }
    }

    async fn move_file_handle(&mut self) -> Result<R::RecordFile> {
        let current_file = self.current_handle.take();
        let now = self.date_provider.time().date_naive();

        match current_file {
            Some(mut file) if file.get_date() != now => {
                file.flush().await?;
            }
            Some(v) => return Ok(v),
            None => {}
        };
        self.records_storage.create_or_append_record(now).await
    }
}

impl<R: RecordStorage> EventProcessor for LocalSaver<R> {
    async fn process_next(&mut self, message: RecordEvent) -> anyhow::Result<()> {
        let mut active_file = self.move_file_handle().await?;

        active_file
            .append(vec![UsageRecordEntity {
                window_name: message.window_name,
                process_name: message.process_name,
                moment: message.timestamp,
                afk: message.afk,
            }])
            .await?;

        Ok(())
    }

    async fn finalize(&mut self) -> Result<()> {
        if let Some(v) = self.current_handle.as_mut() {
            v.flush().await?;
        }
        Ok(())
    }
}
