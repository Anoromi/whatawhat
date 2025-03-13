use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use collection::{afk::AfkEvaluator, collector::DataCollectionModule};
use processing::{local_save::LocalSaver, ProcessingModule};
use storage::{record_event::RecordEvent, record_storage::RecordStorageImpl};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::{
    utils::clock::DefaultClock,
    windows_api::{GenericWindowManager, WindowManager},
};

pub mod collection;
pub mod processing;
pub mod storage;
pub mod update;

const DEFAULT_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);

/// Represents the starting point for the daemon
pub async fn start_daemon(dir: PathBuf) -> Result<()> {
    // TODO
    std::env::set_current_dir("/");
    let (sender, receiver) = mpsc::channel::<RecordEvent>(10);
    let manager = GenericWindowManager::new()?;

    let shudown_token = CancellationToken::new();

    let collector = create_collector(sender, manager, &shudown_token);

    let processor = create_processor(dir, receiver)?;

    let (_, collection_result, processing_result) = tokio::join!(
        update::detect_shutdown(shudown_token),
        collector.run(),
        processor.run(),
    );

    if let Err(collection_result) = collection_result {
        error!(
            "Collection module resulted in an error {}",
            collection_result
        );
    }

    if let Err(processing_result) = processing_result {
        error!(
            "Processing module resulted in an error {}",
            processing_result
        );
    }

    Ok(())
}

fn create_collector(
    sender: mpsc::Sender<RecordEvent>,
    manager: GenericWindowManager,
    shudown_token: &CancellationToken,
) -> DataCollectionModule {
    DataCollectionModule::new(
        sender,
        Box::new(manager) as Box<dyn WindowManager>,
        shudown_token.clone(),
        AfkEvaluator::from_seconds(60 * 2),
        DEFAULT_COLLECTION_INTERVAL,
        Box::new(DefaultClock),
    )
}

fn create_processor(
    dir: PathBuf,
    receiver: mpsc::Receiver<RecordEvent>,
) -> Result<ProcessingModule<LocalSaver<RecordStorageImpl>>, anyhow::Error> {
    let storage = RecordStorageImpl::new(dir.join("records"))?;
    let saver = LocalSaver::new(storage, Box::new(DefaultClock));
    Ok(ProcessingModule::new(receiver, saver))
}
