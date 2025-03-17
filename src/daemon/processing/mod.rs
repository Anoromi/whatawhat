use anyhow::Result;
use module::EventProcessor;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info};

use super::storage::record_event::RecordEvent;

pub mod local_save;
pub mod module;

/// Represents collector of records. This module is responsible for receiving events and saving
/// them using various means.
pub struct ProcessingModule<Processor> {
    receiver: Receiver<RecordEvent>,
    processor: Processor,
}

impl<P: EventProcessor> ProcessingModule<P> {
    pub fn new(receiver: Receiver<RecordEvent>, processor: P) -> Self {
        Self {
            receiver,
            processor,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(record) = self.receiver.recv().await {
            debug!("Processing event {:?}", record);
            match self.processor.process_next(record.clone()).await {
                Ok(_) => {
                    info!("Processed event {:?}", record)
                }
                Err(e) => {
                    error!("Error processing event {:?}: {e:?}", record)
                }
            }
        }

        let result = self.processor.finalize().await;
        self.receiver.close();
        result
    }
}
