use anyhow::Result;

use crate::{
    daemon::storage::{
        application_storage::UsageRecordEntity,
        record_storage::{ColorIndexStorage, RecordFileHandle, RecordStorage},
    },
    utils::time::get_current_time,
};

use super::module::EventProcessor;

pub struct DataSaver<R: RecordStorage, Cs> {
    records_storage: R,
    current_handle: Option<R::RecordFile>,
    color_storage: Cs,
}

impl<R: RecordStorage, Cs> DataSaver<R, Cs> {
    async fn move_file_handle(&mut self) -> Result<R::RecordFile> {
        let current_file = self.current_handle.take();
        let now = get_current_time().date_naive();
        match current_file {
            Some(file) if file.get_date() != now => self.records_storage.compact_file(file).await?,
            Some(v) => return Ok(v),
            None => {}
        };
        let next_day = now.succ_opt().unwrap();
        self.records_storage.create_or_append_record(next_day).await
    }
}

impl<R: RecordStorage, Cs: ColorIndexStorage> EventProcessor for DataSaver<R, Cs> {
    async fn process_next(
        &mut self,
        message: crate::daemon::storage::record_event::Record,
    ) -> anyhow::Result<()> {
        let active_file = self.move_file_handle().await?;

        if let Some(color) = message.color {
            self.color_storage
                .add_element(message.process_name.as_ref(), color)
                .await?;
        }

        active_file
            .append(UsageRecordEntity {
                window_name: message.window_name,
                process_name: message.process_name,
                moment: message.timestamp,
                afk: message.afk,
            })
            .await?;

        Ok(())
    }

    async fn finalize(&mut self) -> anyhow::Result<()> {
        self.color_storage.finalize().await?;
        Ok(())
    }
}
