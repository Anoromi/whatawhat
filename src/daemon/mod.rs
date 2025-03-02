use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use collection::{afk::AfkEvaluator, collector::DataCollectionModule, producer::WindowData};
use pipeline_event::PipeEvent;
use processing::{local_save::LocalSaver, ProcessingModule};
use storage::{
    application_storage::ApplicationStorageImpl, record_event::Record,
    record_storage::ColorIndexStorage,
};
use tokio::sync::{broadcast, mpsc};

use crate::windows_api::{GenericWindowManager, WindowManager};

pub mod collection;
pub mod pipeline_event;
pub mod processing;
pub mod storage;
pub mod update;

const DEFAULT_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);

// fn init_file() -> Result<()> {
//
// }

pub async fn start_daemon(dir: PathBuf) -> Result<()> {
    let (sender, receiver) = mpsc::channel::<PipeEvent<Record>>(10);
    let manager = GenericWindowManager::new()?;
    let collector = DataCollectionModule::new(
        sender,
        Box::new(manager) as Box<dyn WindowManager>,
        AfkEvaluator::from_s(60 * 2),
        DEFAULT_COLLECTION_INTERVAL,
        Box::new(chrono::Utc::now),
    );
    let storage = ApplicationStorageImpl::create(dir.join("records"))?;

    let saver = LocalSaver::new(storage, NoColorIndex, Box::new(chrono::Utc::now));
    let processing = ProcessingModule::new(receiver, saver);
    //let
    // let service_execution = tokio::join![processing.start()];
    // service_execution.0?;

    tokio::select! {
        v = update::detect_messages() => {
            v?;
        }
        v = processing.run() => {
            v?;
        }
        v = collector.run() => {
            v?;
        }

    }
    // tokio::select! {
    //     _ => processing.run(), collector.run()};
    // let (a, b, c) = tokio::join!{
    //     update::detect_messages(), processing.run(), collector.run()
    // };
    //
    // a?;
    // b?;
    // c?;

    Ok(())
}

struct NoColorIndex;

impl ColorIndexStorage for NoColorIndex {
    async fn add_element(&self, key: &str, value: storage::record_event::Color) -> Result<()> {
        Ok(())
    }

    async fn finalize(&self) -> Result<()> {
        Ok(())
    }
}
