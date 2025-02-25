use std::time::Duration;

use anyhow::Result;
use collection::producer::WindowData;
use tokio::sync::broadcast;

pub mod collection;
pub mod pipeline_event;
pub mod processing;
pub mod storage;
pub mod update;

const DEFAULT_COLLECTION_INTERVAL: Duration = Duration::from_secs(5);

pub async fn start_daemon() -> Result<()> {
    let (sender, receiver) = broadcast::channel::<WindowData>(10);
    // let collector = DataCollectorImpl::new(next, producer, collect_frequency)
    // let processing: ProcessingService<RecordEventDto> = todo!();
    // let service_execution = tokio::join![processing.start()];
    // service_execution.0?;

    update::detect_messages().await;

    Ok(())
}
