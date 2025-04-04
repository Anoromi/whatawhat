use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use collection::{afk::AfkEvaluator, collector::DataCollectionModule};
use processing::{local_save::LocalSaver, ProcessingModule};
use storage::{record_event::RecordEvent, record_storage::RecordStorageImpl};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::{
    utils::clock::{Clock, DefaultClock},
    window_api::{GenericWindowManager, WindowManager},
};

pub mod args;
pub mod collection;
pub mod processing;
pub mod shutdown;
pub mod storage;

const DEFAULT_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);

/// Represents the starting point for the daemon
pub async fn start_daemon(dir: PathBuf) -> Result<()> {
    std::env::set_current_dir("/")?;

    let (sender, receiver) = mpsc::channel::<RecordEvent>(10);
    let manager = GenericWindowManager::new()?;

    let shutdown_token = CancellationToken::new();

    let collector = create_collector(sender, manager, &shutdown_token, DefaultClock);

    let processor = create_processor(dir.join("records"), receiver, DefaultClock)?;

    let (_, collection_result, processing_result) = tokio::join!(
        shutdown::detect_shutdown(shutdown_token),
        collector.run(),
        processor.run(),
    );

    if let Err(collection_result) = collection_result {
        error!("Collection module got an error {:?}", collection_result);
    }

    if let Err(processing_result) = processing_result {
        error!("Processing module got an error {:?}", processing_result);
    }

    Ok(())
}

fn create_collector(
    sender: mpsc::Sender<RecordEvent>,
    manager: impl WindowManager + 'static,
    shutdown_token: &CancellationToken,
    clock: impl Clock,
) -> DataCollectionModule {
    DataCollectionModule::new(
        sender,
        Box::new(manager),
        shutdown_token.clone(),
        AfkEvaluator::from_seconds(60 * 2),
        DEFAULT_COLLECTION_INTERVAL,
        Box::new(clock),
    )
}

fn create_processor(
    record_dir: PathBuf,
    receiver: mpsc::Receiver<RecordEvent>,
    clock: impl Clock,
) -> Result<ProcessingModule<LocalSaver<RecordStorageImpl>>, anyhow::Error> {
    let storage = RecordStorageImpl::new(record_dir)?;
    let saver = LocalSaver::new(storage, Box::new(clock));
    Ok(ProcessingModule::new(receiver, saver))
}

#[cfg(test)]
mod daemon_tests {
    use std::{fs, time::Duration};

    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
    use tempfile::tempdir;
    use tokio::{sync::mpsc, time::Instant};
    use tokio_util::sync::CancellationToken;

    use crate::{
        daemon::{
            create_collector, create_processor,
            storage::{
                record_event::RecordEvent,
                record_storage::{RecordStorage, RecordStorageImpl},
            },
        },
        utils::{clock::Clock, logging::TEST_LOGGING},
        window_api::{ActiveWindowData, MockWindowManager},
    };

    const TEST_START_DATE: NaiveDateTime =
        NaiveDateTime::new(NaiveDate::from_ymd_opt(2018, 7, 4).unwrap(), NaiveTime::MIN);

    fn test_items() -> Vec<ActiveWindowData> {
        vec![
            ActiveWindowData {
                window_title: "test".into(),
                process_name: "test".into(),
            },
            ActiveWindowData {
                window_title: "test".into(),
                process_name: "test".into(),
            },
            ActiveWindowData {
                window_title: "test b".into(),
                process_name: "test b".into(),
            },
        ]
    }

    #[derive(Clone)]
    struct TestClock {
        start_time: DateTime<Utc>,
        reference: Instant,
    }

    #[async_trait]
    impl Clock for TestClock {
        fn time(&self) -> DateTime<Utc> {
            self.start_time + self.reference.elapsed()
        }

        fn instant(&self) -> Instant {
            Instant::now()
        }

        async fn sleep(&self, duration: Duration) {
            tokio::time::sleep(duration).await;
        }

        async fn sleep_until(&self, instant: tokio::time::Instant) {
            tokio::time::sleep_until(instant).await;
        }
    }

    /// Very simple smoke test to check if the application is working properly. It can be improved
    /// by warping time so that it takes 10 times less time, but for now we have what we have.
    #[tokio::test]
    async fn smoke_test_daemon() -> Result<()> {
        *TEST_LOGGING;
        let mut mock_window_manager = MockWindowManager::new();
        mock_window_manager
            .expect_get_idle_time()
            .returning(|| Ok(0));
        let mut items = test_items().into_iter().cycle();
        mock_window_manager
            .expect_get_active_window_data()
            .returning(move || Ok(items.next().unwrap()))
            .times(..7);

        let shutdown_token = CancellationToken::new();

        let (sender, receiver) = mpsc::channel::<RecordEvent>(10);
        let test_clock = TestClock {
            start_time: Utc.from_utc_datetime(&TEST_START_DATE),
            reference: Instant::now(),
        };
        let collector = create_collector(
            sender,
            mock_window_manager,
            &shutdown_token,
            test_clock.clone(),
        );

        let dir = tempdir()?;

        let processor = create_processor(dir.path().to_path_buf(), receiver, test_clock.clone())?;

        let (_, collection_result, processing_result) = tokio::join!(
            async {
                tokio::time::sleep(Duration::from_millis(5500)).await;
                shutdown_token.cancel()
            },
            collector.run(),
            processor.run(),
        );

        collection_result?;
        processing_result?;

        let files = fs::read_dir(dir.path())?.collect::<Vec<_>>();
        assert_eq!(files.len(), 1);

        let storage = RecordStorageImpl::new(dir.path().to_path_buf())?;

        let data = storage.get_data_for(TEST_START_DATE.date()).await?;

        assert_eq!(data.len(), 4);

        Ok(())
    }
}
