use std::{
    cell::Cell,
    collections::VecDeque,
    io::{Cursor, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt},
};
use tracing::{info, warn};

use crate::{
    fs::{async_file_ext::FileLock, operations::seek_line_backwards},
    utils::retry::run_with_retry,
};

use super::{
    entities::{UsageIntervalEntity, UsageRecordEntity},
    record_storage::{RecordFileHandle, RecordStorage},
};

pub trait CompactFileHandle {
    fn get_items() -> Vec<UsageIntervalEntity>;
}

pub struct ApplicationStorageImpl {
    record_dir: PathBuf,
}

impl ApplicationStorageImpl {
    pub fn create(record_dir: PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(&record_dir)?;

        Ok(Self { record_dir })
    }

    async fn get_all_inner(&self, path: &Path) -> Result<Vec<UsageIntervalEntity>> {
        match fs::read_to_string(path).await {
            Ok(s) => Ok(csv::Reader::from_reader(s.as_bytes())
                .deserialize()
                .map(|v| v.unwrap())
                .collect::<Vec<UsageIntervalEntity>>()),
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(vec![])
                } else {
                    Err(e)?
                }
            }
        }
    }
}

impl RecordStorage for ApplicationStorageImpl {
    type RecordFile = UsageIntervalRecordFile<File>;

    /// this is intended for future use to potentially reduce file size
    async fn compact_files(&self) -> Result<()> {
        unimplemented!("Api isn't ready")
    }

    async fn create_or_append_record(&self, date: chrono::NaiveDate) -> Result<Self::RecordFile> {
        let file_name = date.format("%Y-%m-%d").to_string();
        let path = self.record_dir.join(file_name);

        let v = File::options()
            .write(true)
            .create(true)
            .append(true)
            .read(true)
            .open(path)
            .await?;

        Ok(UsageIntervalRecordFile::new(v, date))
    }

    async fn get_data_for(&self, date: chrono::NaiveDate) -> Result<Vec<UsageIntervalEntity>> {
        let file_name = date.format("%Y-%m-%d").to_string();
        let path = self.record_dir.join(file_name);
        let data = self.get_all_inner(&path).await?;
        Ok(data)
    }
}

pub struct UsageIntervalRecordFile<F> {
    file: F,
    date: NaiveDate,
}

impl<F: AsyncSeek + AsyncRead + AsyncWrite + FileLock + Unpin> RecordFileHandle
    for UsageIntervalRecordFile<F>
{
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

impl<F: AsyncSeek + AsyncRead + AsyncWrite + FileLock + Unpin> UsageIntervalRecordFile<F> {
    fn new(file: F, date: NaiveDate) -> Self {
        Self { file, date }
    }

    async fn extract_line_backwards(file: &mut F) -> Result<String, anyhow::Error> {
        seek_line_backwards(file, &mut vec![0; 1024]).await?;
        let mut last_line = String::new();
        file.read_to_string(&mut last_line).await?;
        Ok(last_line)
    }

    async fn append_inner(&mut self, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        // Semi-safe aquire-release
        info!("appending");
        self.file.lock_exclusive_in_place()?;
        info!("locking");
        let result = Self::append_with_file(&mut self.file, usage_record).await;
        info!("writing");
        info!("hehe");
        dbg!(self.file.unlock_in_place())?;
        info!("unlocked");
        result
    }

    async fn append_with_file(file: &mut F, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        dbg!(file.seek(std::io::SeekFrom::End(0)).await?);

        let last_line = Self::extract_line_backwards(file).await?;

        dbg!(
            file.seek(std::io::SeekFrom::Current(-(last_line.len() as i64)))
                .await?
        );

        let last_interval: Option<UsageIntervalEntity> = if last_line.is_empty() {
            None
        } else {
            match serde_json::from_str::<UsageIntervalEntity>(&last_line) {
                Ok(v) => Some(v),
                Err(e) => {
                    // Might happen due to shutdown cutting of the write into a file.
                    warn!("Last record was corrupted {e}");
                    None
                }
            }
        };

        let collapsed = collapse_records(last_interval, usage_record);

        dbg!(&collapsed);
        let mut buffer = Vec::<u8>::new();
        for interval in collapsed {
            serde_json::to_writer(&mut buffer, &interval)?;
            buffer.push(b'\n');
        }

        file.write_all(&buffer).await?;
        Ok(())
    }
}

fn collapse_records(
    last_interval: Option<UsageIntervalEntity>,
    usage_records: impl IntoIterator<Item = UsageRecordEntity>,
) -> Vec<UsageIntervalEntity> {
    let mut intervals = Vec::new();
    if let Some(last) = last_interval {
        intervals.push(last);
    }

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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use chrono::{Duration, Utc};
    use tempfile::tempfile;
    use tokio::{fs::File, io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt}};

    use crate::daemon::storage::entities::{UsageIntervalEntity, UsageRecordEntity};

    use super::UsageIntervalRecordFile;

    #[tokio::test]
    async fn test_appender() {
        let mut previous = serde_json::to_string(&UsageIntervalEntity {
            window_name: "initial".into(),
            process_name: "initial".into(),
            start: Utc::now() - Duration::seconds(2),
            end: Utc::now() - Duration::seconds(1),
            afk: false,
        })
        .unwrap();

        previous.push('\n');

        previous += &serde_json::to_string(&UsageIntervalEntity {
            window_name: "window".into(),
            process_name: "process".into(),
            start: Utc::now() - Duration::seconds(2),
            end: Utc::now() - Duration::seconds(1),
            afk: true,
        }).unwrap();
        previous += "\n";


        let mut file = Cursor::new(previous.into_bytes());
        file.seek(std::io::SeekFrom::End(0)).await.unwrap();

        let mut usage = UsageIntervalRecordFile::new(file, Utc::now().date_naive());

        usage.append_inner(vec![UsageRecordEntity {
            window_name: "window".into(),
            process_name: "process".into(),
            moment: Utc::now(),
            afk: true,
        }]).await.unwrap();

        usage.file.rewind().await.unwrap();
        let mut s = String::new();
        usage.file.read_to_string(&mut s).await.unwrap();
        assert_eq!(s.lines().count(), 2);

        
    }

    #[tokio::test]
    async fn test_file_appender() {
        let mut previous = serde_json::to_string(&UsageIntervalEntity {
            window_name: "initial".into(),
            process_name: "initial".into(),
            start: Utc::now() - Duration::seconds(2),
            end: Utc::now() - Duration::seconds(1),
            afk: false,
        })
        .unwrap();

        previous.push('\n');

        previous += &serde_json::to_string(&UsageIntervalEntity {
            window_name: "window".into(),
            process_name: "process".into(),
            start: Utc::now() - Duration::seconds(2),
            end: Utc::now() - Duration::seconds(1),
            afk: true,
        }).unwrap();
        previous += "\n";

        let file = tempfile().unwrap();
        let mut file = File::from_std(file);

        file.write_all(previous.as_bytes()).await.unwrap();

        file.seek(std::io::SeekFrom::End(0)).await.unwrap();



        let mut usage = UsageIntervalRecordFile::new(file, Utc::now().date_naive());

        usage.append_inner(vec![UsageRecordEntity {
            window_name: "window".into(),
            process_name: "process".into(),
            moment: Utc::now(),
            afk: true,
        }]).await.unwrap();

        usage.file.rewind().await.unwrap();
        let mut s = String::new();
        usage.file.read_to_string(&mut s).await.unwrap();
        assert_eq!(s.lines().count(), 2);

        
    }
}
