use std::{
    future::Future,
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::{Duration, NaiveDate};
use fs4::tokio::AsyncFileExt;
use tokio::{
    fs::File,
    io::{
        AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite,
        AsyncWriteExt, BufReader,
    },
};
use tracing::{trace, warn};

use crate::{
    fs::operations::seek_line_backwards,
    utils::time::date_to_record_name,
};

use super::entities::{UsageIntervalEntity, UsageRecordEntity};

/// Interface for abstracting data save
pub trait RecordStorage {
    type RecordFile: RecordFileHandle;

    fn create_or_append_record(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Self::RecordFile>>;

    fn get_data_for(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<UsageIntervalEntity>>> + Send;
}

impl<T: Deref> RecordStorage for T
where
    T::Target: RecordStorage,
{
    type RecordFile = <T::Target as RecordStorage>::RecordFile;

    fn create_or_append_record(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Self::RecordFile>> {
        self.deref().create_or_append_record(date)
    }

    fn get_data_for(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<UsageIntervalEntity>>> + Send {
        self.deref().get_data_for(date)
    }
}

pub trait RecordFileHandle {
    fn append(&mut self, usage_records: Vec<UsageRecordEntity>)
        -> impl Future<Output = Result<()>>;
    fn get_date(&self) -> NaiveDate;
    fn flush(&mut self) -> impl Future<Output = Result<()>>;
}

pub struct RecordStorageImpl {
    record_dir: PathBuf,
}

impl RecordStorageImpl {
    pub fn new(record_dir: PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(&record_dir)?;

        Ok(Self { record_dir })
    }

    async fn get_all_inner(&self, path: &Path) -> Result<Vec<UsageIntervalEntity>> {
        let extract = async || -> Result<Vec<UsageIntervalEntity>, std::io::Error> {
            trace!("Extracting {path:?}");
            let file = File::open(path).await?;
            file.lock_shared()?;
            let buffer = BufReader::new(file);
            let mut lines = buffer.lines();
            let mut intervals = vec![];
            while let Ok(Some(v)) = lines.next_line().await {
                match serde_json::from_str::<UsageIntervalEntity>(&v) {
                    Ok(v) => intervals.push(v),
                    Err(e) => {
                        // ignore illegal values. Might happen after shutdowns
                        warn!(
                            "During parsing in path {:?} found illegal json string {}:  {e}",
                            path, &v
                        )
                    }
                }
            }

            lines.into_inner().into_inner().unlock_async().await?;

            Ok(intervals)
        };

        match extract().await {
            Ok(s) => Ok(s),
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

impl RecordStorage for RecordStorageImpl {
    type RecordFile = UsageIntervalRecordFile<File>;

    async fn create_or_append_record(&self, date: chrono::NaiveDate) -> Result<Self::RecordFile> {
        let file_name = date_to_record_name(date);
        let path = self.record_dir.join(file_name);

        let v = File::options()
            .write(true)
            .create(true)
            .read(true)
            .truncate(false)
            .open(path)
            .await?;

        Ok(UsageIntervalRecordFile::new(v, date))
    }

    async fn get_data_for(&self, date: chrono::NaiveDate) -> Result<Vec<UsageIntervalEntity>> {
        let file_name = date_to_record_name(date);
        let path = self.record_dir.join(file_name);
        let data = self.get_all_inner(&path).await?;
        Ok(data)
    }
}

pub struct UsageIntervalRecordFile<F> {
    file: F,
    date: NaiveDate,
}

impl<F: AsyncSeek + AsyncRead + AsyncWrite + fs4::tokio::AsyncFileExt + Unpin> RecordFileHandle
    for UsageIntervalRecordFile<F>
{
    async fn append(&mut self, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        self.append_inner(usage_record.clone()).await
    }

    fn get_date(&self) -> chrono::NaiveDate {
        self.date
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<F: AsyncSeek + AsyncRead + AsyncWrite + fs4::tokio::AsyncFileExt + Unpin>
    UsageIntervalRecordFile<F>
{
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
        // Semi-safe acquire-release for a file
        self.file.lock_exclusive()?;
        let result = Self::append_with_file(&mut self.file, usage_record).await;
        self.file.unlock_async().await?;
        result
    }

    async fn append_with_file(file: &mut F, usage_record: Vec<UsageRecordEntity>) -> Result<()> {
        file.seek(std::io::SeekFrom::End(0)).await?;

        let last_line = Self::extract_line_backwards(file).await?;

        file.seek(std::io::SeekFrom::Current(-(last_line.len() as i64)))
            .await?;

        file.stream_position().await?;

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

        let mut buffer = Vec::<u8>::new();
        for interval in collapsed {
            serde_json::to_writer(&mut buffer, &interval)?;
            buffer.push(b'\n');
        }

        file.write_all(&buffer).await?;
        file.flush().await?;
        Ok(())
    }
}

/// Value used to bridge gap between window transitions. There should be a limit though so that an
/// event that happened an hour ago didn't affect new events
const MAX_MERGE_DURATION: Duration = Duration::seconds(2);

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
                interval.set_end(record.moment)
            }
            Some(_) | None => {
                let mut next_interval: UsageIntervalEntity = record.into();
                match intervals.last() {
                    Some(previous) if next_interval.start - previous.end() < MAX_MERGE_DURATION => {
                        next_interval.start = previous.end();
                    }
                    _ => (),
                }
                intervals.push(next_interval);
            }
        }
    }

    intervals
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anyhow::Result;
    use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
    use tempfile::{tempdir, tempfile};
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    use crate::daemon::storage::{
        entities::{UsageIntervalEntity, UsageRecordEntity},
        record_storage::{collapse_records, RecordFileHandle, RecordStorage, RecordStorageImpl},
    };

    use super::UsageIntervalRecordFile;

    const START_DATE: NaiveDateTime =
        NaiveDateTime::new(NaiveDate::from_ymd_opt(2018, 7, 4).unwrap(), NaiveTime::MIN);

    #[tokio::test]
    async fn test_appender_basic() -> Result<()> {
        let file = tokio::fs::File::from_std(tempfile().unwrap());

        let mut usage = UsageIntervalRecordFile::new(file, Utc::now().date_naive());
        usage
            .append_inner(vec![UsageRecordEntity {
                window_name: "initial".into(),
                process_name: "initial".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            }])
            .await?;

        usage
            .append_inner(vec![UsageRecordEntity {
                window_name: "window".into(),
                process_name: "process".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(1),
                afk: true,
            }])
            .await?;

        usage
            .append_inner(vec![UsageRecordEntity {
                window_name: "third".into(),
                process_name: "process".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(2),
                afk: true,
            }])
            .await?;

        usage
            .append_inner(vec![UsageRecordEntity {
                window_name: "third".into(),
                process_name: "process".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(3),
                afk: true,
            }])
            .await?;

        usage.file.rewind().await?;
        let mut s = String::new();
        usage.file.read_to_string(&mut s).await?;
        assert_eq!(s.lines().count(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_appender_overwrite() -> Result<()> {
        let mut previous = serde_json::to_string(&UsageIntervalEntity {
            window_name: "initial".into(),
            process_name: "initial".into(),
            start: Utc::now() - Duration::seconds(2),
            duration: Duration::seconds(1),
            afk: false,
        })?;

        previous.push('\n');

        previous += &serde_json::to_string(&UsageIntervalEntity {
            window_name: "window".into(),
            process_name: "process".into(),
            start: Utc::now() - Duration::seconds(2),
            duration: Duration::seconds(1),
            afk: true,
        })?;
        previous += "\n";

        let mut file = tempfile().unwrap();
        file.write_all(previous.as_bytes())?;
        let mut file = tokio::fs::File::from_std(file);
        file.seek(std::io::SeekFrom::End(0)).await?;

        let mut usage = UsageIntervalRecordFile::new(file, Utc::now().date_naive());

        usage
            .append_inner(vec![UsageRecordEntity {
                window_name: "window".into(),
                process_name: "process".into(),
                moment: Utc::now(),
                afk: true,
            }])
            .await?;

        usage.file.rewind().await?;
        let mut s = String::new();
        usage.file.read_to_string(&mut s).await?;
        assert_eq!(s.lines().count(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_record_storage_basic() -> Result<()> {
        let dir = tempdir()?;
        let storage = RecordStorageImpl::new(dir.path().to_owned())?;
        let mut record = storage.create_or_append_record(START_DATE.date()).await?;
        let records = [
            UsageRecordEntity {
                window_name: "test".into(),
                process_name: "test process".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            },
        ];
        record.append_inner(vec![records[0].clone()]).await?;

        record.append_inner(vec![records[1].clone()]).await?;

        record.flush().await?;

        let values = storage.get_data_for(START_DATE.into()).await?;

        assert_eq!(
            values,
            vec![records[0].clone().into(), records[1].clone().into()]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_record_storage_appending() -> Result<()> {
        let dir = tempdir()?;
        let storage = RecordStorageImpl::new(dir.path().to_owned())?;
        let mut record = storage.create_or_append_record(START_DATE.date()).await?;
        let records = [
            UsageRecordEntity {
                window_name: "test".into(),
                process_name: "test process".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(1),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(5),
                afk: false,
            },
        ];
        record.append_inner(vec![records[0].clone()]).await?;

        record.append_inner(vec![records[1].clone()]).await?;

        record.append_inner(vec![records[2].clone()]).await?;

        record.flush().await?;

        let values = storage.get_data_for(START_DATE.into()).await?;

        assert_eq!(
            values,
            vec![
                records[0].clone().into(),
                UsageIntervalEntity::from(records[1].clone())
                    .with_start(records[0].moment)
                    .with_duration(Duration::seconds(5))
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_record_collapsing_basic() -> Result<()> {
        let records = [
            UsageRecordEntity {
                window_name: "test".into(),
                process_name: "test process".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(1),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(5),
                afk: false,
            },
        ];
        let values = collapse_records(None, records.clone());

        assert_eq!(values.len(), 2);
        assert_eq!(
            values,
            vec![
                records[0].clone().into(),
                UsageIntervalEntity::from(records[1].clone())
                    .with_start(records[0].moment)
                    .with_duration(Duration::seconds(5))
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_record_collapsing_cutoff() -> Result<()> {
        let records = [
            UsageRecordEntity {
                window_name: "test".into(),
                process_name: "test process".into(),
                moment: Utc.from_utc_datetime(&START_DATE),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(3),
                afk: false,
            },
            UsageRecordEntity {
                window_name: "test 2".into(),
                process_name: "test process 2".into(),
                moment: Utc.from_utc_datetime(&START_DATE) + Duration::seconds(5),
                afk: false,
            },
        ];
        let values = collapse_records(None, records.clone());

        assert_eq!(values.len(), 2);
        assert_eq!(
            values,
            vec![
                records[0].clone().into(),
                UsageIntervalEntity::from(records[1].clone())
                    .with_start(records[1].moment)
                    .with_duration(Duration::seconds(2))
            ]
        );

        Ok(())
    }
}
