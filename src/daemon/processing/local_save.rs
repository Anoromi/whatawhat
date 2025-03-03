use std::ops::DerefMut;

use anyhow::Result;

use crate::{
    daemon::storage::{
        entities::UsageRecordEntity,
        record_event::Record,
        record_storage::{ColorIndexStorage, RecordFileHandle, RecordStorage},
    },
    utils::{
        date_provider::{self, DateProvider},
        time::get_current_time,
    },
};

use super::module::EventProcessor;

pub struct LocalSaver<R: RecordStorage, Cs> {
    records_storage: R,
    current_handle: Option<R::RecordFile>,
    color_storage: Cs,
    date_provider: DateProvider,
    buffered_records: Vec<Record>,
}

impl<R: RecordStorage, Cs> LocalSaver<R, Cs> {
    pub fn new(records_storage: R, color_storage: Cs, date_provider: DateProvider) -> Self {
        Self {
            records_storage,
            current_handle: None,
            color_storage,
            date_provider,
            buffered_records: Default::default(),
        }
    }

    async fn move_file_handle(&mut self) -> Result<R::RecordFile> {
        let current_file = self.current_handle.take();
        let now = self.date_provider.deref_mut()().date_naive();
        println!("now {now}");
        match current_file {
            // This shouldn't be the responsibility of LocalSaver.
            // TODO detach it somewhere
            Some(file) if file.get_date() != now => self.records_storage.compact_file(file).await?,
            Some(v) => return Ok(v),
            None => {}
        };
        self.records_storage.create_or_append_record(now).await
    }


}

impl<R: RecordStorage, Cs: ColorIndexStorage> EventProcessor for LocalSaver<R, Cs> {
    
    async fn process_next(&mut self, message: Record) -> anyhow::Result<()> {
        let mut active_file = self.move_file_handle().await?;

        if let Some(color) = message.color {
            self.color_storage
                .add_element(message.process_name.as_ref(), color)
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
        self.color_storage.finalize().await?;
        Ok(())
    }
}
