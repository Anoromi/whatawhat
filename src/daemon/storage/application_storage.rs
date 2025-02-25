use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

use crate::{fs::operations::seek_line_backwards, utils::retry::run_with_retry};

use super::{
    entities::{UsageIntervalEntity, UsageRecordEntity},
    record_storage::{RecordFileHandle, RecordStorage},
};

pub trait CompactFileHandle {
    fn get_items() -> Vec<UsageIntervalEntity>;
}

struct ApplicationStorageImpl {
    record_dir: Box<Path>,
}

impl RecordStorage for ApplicationStorageImpl {
    type RecordFile = UsageIntervalRecordFile;

    /// this is intended for future use to potentially compact local data
    async fn compact_files(&self) -> Result<()> {
        unimplemented!("Api isn't ready")
    }

    async fn create_or_append_record(&self, date: chrono::NaiveDate) -> Result<Self::RecordFile> {
        todo!()
    }

    async fn get_data_for(&self, date: chrono::NaiveDate) -> Result<Vec<UsageIntervalEntity>> {
        todo!()
    }
}

struct UsageIntervalRecordFile {
    file_path: Box<Path>,
    date: NaiveDate,
}

trait GetItems<T> {
    async fn get_all(&self) -> Result<Vec<T>>;
}

impl GetItems<UsageRecordEntity> for UsageIntervalRecordFile {
    async fn get_all(&self) -> Result<Vec<UsageRecordEntity>> {
        run_with_retry(10, Duration::from_millis(100), || self.get_all_inner()).await
    }
}

impl RecordFileHandle for UsageIntervalRecordFile {
    async fn append(&mut self, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        run_with_retry(10, Duration::from_millis(100), async || {
            self.append_inner(usage_record.clone()).await
        })
        .await
    }

    fn get_date(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl UsageIntervalRecordFile {
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
                    Err(e)?
                }
            }
        }
    }

    async fn extract_line_backwards(file: &mut File) -> Result<String, anyhow::Error> {
        seek_line_backwards(file, &mut vec![0; 1024]).await?;
        let mut last_line = String::new();
        file.read_to_string(&mut last_line).await?;
        Ok(last_line)
    }

    async fn append_inner(&mut self, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        let mut file = File::options()
            .write(true)
            .create(true)
            .append(true)
            .read(true)
            .open(&self.file_path)
            .await?;

        let mut usage_record = VecDeque::from(usage_record);

        file.seek(std::io::SeekFrom::End(0));

        let mut last_line = Self::extract_line_backwards(&mut file).await?;
        if last_line.is_empty() {
            todo!()
            // return Ok(())
        } else {
            let reader = csv::Reader::from_reader(Cursor::new(last_line.into_bytes()));
            // let value = reader.deserialize::<UsageIntervalEntity>().next().unwrap()?;
            // usage_record.push_back(value);
        }

        // usage_record

        // let mut writer = csv::Writer::from_writer(vec![]);
        // writer.serialize(usage_record)?;
        // let mut buffer = writer.into_inner()?;
        // seek_line_backwards(file, buffer)
        // seek_line_backwards(buffer, buffer)
        // file.write_all_buf(&mut Cursor::new(buffer.as_mut_slice()))
        //     .await?;
        drop(file);
        Ok(())
    }
}

fn collapse_records(
    usage_records: impl IntoIterator<Item = UsageRecordEntity>,
) -> Vec<UsageIntervalEntity> {
    let mut intervals = Vec::<UsageIntervalEntity>::new();
    for record in usage_records {
        match intervals.last_mut() {
            Some(interval)
                if interval.window_name == record.window_name
                    && interval.process_name == record.process_name
                    && interval.afk == record.afk =>
            {
                interval.end = record.moment
            }
            Some(_) | None => {
                intervals.push(record.into());
            }
        }
    }

    intervals
}
