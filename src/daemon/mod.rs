use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use collection::{afk::AfkEvaluator, collector::DataCollectionModule};
use pipeline_event::PipeEvent;
use processing::{local_save::LocalSaver, ProcessingModule};
use storage::{
    record_event::RecordEvent,
    record_storage::{ColorIndexStorage, RecordStorageImpl},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::{
    utils::date_provider::DefaultClock,
    windows_api::{GenericWindowManager, WindowManager},
};

pub mod collection;
pub mod pipeline_event;
pub mod processing;
pub mod storage;
pub mod update;

const DEFAULT_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);


/// Represents the starting point for the daemon
pub async fn start_daemon(dir: PathBuf) -> Result<()> {
    let (sender, receiver) = mpsc::channel::<PipeEvent<RecordEvent>>(10);
    let manager = GenericWindowManager::new()?;

    let shudown_token = CancellationToken::new();

    let collector = DataCollectionModule::new(
        sender,
        Box::new(manager) as Box<dyn WindowManager>,
        shudown_token.clone(),
        AfkEvaluator::from_seconds(60 * 2),
        DEFAULT_COLLECTION_INTERVAL,
        Box::new(DefaultClock),
    );

    let storage = RecordStorageImpl::new(dir.join("records"))?;

    let saver = LocalSaver::new(storage, NoColorIndex, Box::new(DefaultClock));
    let processing = ProcessingModule::new(receiver, saver);

    let (_, collection_result, processing_result) = tokio::join!(
        update::detect_shutdown(shudown_token),
        collector.run(),
        processing.run(),
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

struct NoColorIndex;

impl ColorIndexStorage for NoColorIndex {
    async fn recover_shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }

    async fn update_color_index(
        &self,
        _process_name: &str,
        _color: storage::record_event::Color,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_colors_for(
        &self,
        _names: std::collections::BTreeSet<String>,
    ) -> Result<std::collections::BTreeMap<String, Option<storage::record_event::Color>>> {
        Ok(Default::default())
    }
}
