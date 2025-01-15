use anyhow::Result;
use processing::ProcessingService;
use storage::record_event::RecordEventDto;

pub mod collection;
pub mod processing;
pub mod server_update;
mod test;
pub mod storage;

async fn start_server() -> Result<()> {
    // let processing: ProcessingService<RecordEventDto> = todo!();
    // let service_execution = tokio::join![processing.start()];
    // service_execution.0?;

    Ok(())
}
