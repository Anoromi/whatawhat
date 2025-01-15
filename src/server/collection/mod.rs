pub mod collector;
#[cfg(windows)]
pub mod windows;

use std::sync::mpsc::Sender;

use anyhow::Result;
use collector::Collector;

use super::storage::record_event::RecordEventDto;


struct CollectionService<C: Collector> {
    collector: C,
    processor_sender: Sender<RecordEventDto>
}


pub async fn collect() -> Result<()> {
    Ok(())
}
