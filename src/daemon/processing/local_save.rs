use std::ops::DerefMut;

use anyhow::Result;

use crate::{
    daemon::storage::{
        entities::UsageRecordEntity,
        record_event::RecordEvent,
        record_storage::{ColorIndexStorage, RecordFileHandle, RecordStorage},
    }, utils::date_provider::Clock,
};

use super::module::EventProcessor;

pub struct LocalSaver<R: RecordStorage, Cs> {
    records_storage: R,
    current_handle: Option<R::RecordFile>,
    color_storage: Cs,
    date_provider: Box<dyn Clock>,
}

impl<R: RecordStorage, Cs> LocalSaver<R, Cs> {
    pub fn new(records_storage: R, color_storage: Cs, date_provider: Box<dyn Clock>) -> Self {
        Self {
            records_storage,
            current_handle: None,
            color_storage,
            date_provider,
        }
    }

    async fn move_file_handle(&mut self) -> Result<R::RecordFile> {
        let current_file = self.current_handle.take();
        let now = self.date_provider.time().date_naive();
        println!("now {now}");

        match current_file {
            Some(mut file) if file.get_date() != now => {
                file.flush().await?;
                // This shouldn't be the responsibility of LocalSaver.
                // TODO detach it somewhere
                self.records_storage.compact_file(file).await?
            }
            Some(v) => return Ok(v),
            None => {}
        };
        self.records_storage.create_or_append_record(now).await
    }
}

impl<R: RecordStorage, Cs: ColorIndexStorage> EventProcessor for LocalSaver<R, Cs> {
    async fn process_next(&mut self, message: RecordEvent) -> anyhow::Result<()> {
        let mut active_file = self.move_file_handle().await?;

        if let Some(color) = message.color {
            self.color_storage
                .update_color_index(message.process_name.as_ref(), color)
                .await?;
        }

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

    async fn finalize(&mut self) -> anyhow::Result<()> {
        if let Some(v) = self.current_handle.as_mut() {
            v.flush().await?;
        }
        self.color_storage.flush().await?;
        Ok(())
    }
}
