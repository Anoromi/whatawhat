use std::sync::mpsc::Receiver;

use anyhow::Result;

use super::storage::{multi_factor::Repository, record_event::RecordEventDto};

pub struct ProcessingService<R: Repository<RecordEventDto>> {
    repository: R,
    message_receiver: Receiver<RecordEventDto>

}

impl<R: Repository<RecordEventDto>> ProcessingService<R> {
    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
}

