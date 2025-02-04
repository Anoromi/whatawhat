use std::{
    io::{Cursor, ErrorKind},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::utils::retry::run_with_retry;

use super::record_storage::{RecordFileHandle, RecordStorage};

pub trait CompactFileHandle {
    fn get_items() -> Vec<UsageIntervalEntity>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct UsageIntervalEntity {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub afk: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct UsageRecordEntity {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    pub moment: DateTime<Utc>,
    pub afk: bool,
}

struct ApplicationStorageImpl {
    record_dir: Box<Path>

}

impl RecordStorage for ApplicationStorageImpl {
    type RecordFile = RecordHandle;

    async fn compact_files(&self) -> Result<()> {
        let mut entries = fs::read_dir(self.record_dir.clone()).await?;
        while let Some(file) = entries.next_entry().await? {
            if !file.file_type().await?.is_file() {
                continue;
            }


        }

        todo!()
    }

    async fn create_or_append_record(&self, date: chrono::NaiveDate) -> Result<Self::RecordFile> {
        todo!()
    }

    async fn get_data_for(&self, date: chrono::NaiveDate) -> Result<Vec<UsageIntervalEntity>> {
        todo!()
    }
}

struct RecordHandle {
    file_path: Box<Path>,
    date: NaiveDate,
}

trait GetItems<T> {
    async fn get_all(&self) -> Result<Vec<T>>;
}

impl GetItems<UsageRecordEntity> for RecordHandle {
    async fn get_all(&self) -> Result<Vec<UsageRecordEntity>> {
        run_with_retry(10, Duration::from_millis(100), || self.get_all_inner()).await
    }
}

impl RecordFileHandle for RecordHandle {
    async fn append(&self, usage_record: UsageRecordEntity) -> Result<()> {
        run_with_retry(10, Duration::from_millis(100), move || {
            self.append_inner(usage_record.clone())
        })
        .await
    }

    fn get_date(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl RecordHandle {
    async fn get_all_inner(&self) -> Result<Vec<UsageRecordEntity>> {
        match fs::read_to_string(&self.file_path).await {
            Ok(s) => Ok(csv::Reader::from_reader(s.as_bytes())
                .deserialize()
                .map(|v| v.unwrap())
                .collect::<Vec<UsageRecordEntity>>()),
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(vec![])
                } else {
                    return Err(e)?;
                }
            }
        }
    }

    async fn append_inner(&self, usage_record: UsageRecordEntity) -> Result<()> {
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;
        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(usage_record)?;
        let mut buffer = writer.into_inner()?;
        file.write_all_buf(&mut Cursor::new(buffer.as_mut_slice()));
        drop(file);
        Ok(())
    }
}
