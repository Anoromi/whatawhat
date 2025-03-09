use anyhow::Result;
use module::EventProcessor;
use tokio::sync::mpsc::Receiver;
use tracing::debug;

use super::{pipeline_event::PipeEvent, storage::record_event::RecordEvent};

pub mod combined_processing;
pub mod local_save;
pub mod module;

pub struct ProcessingModule<Processor> {
    receiver: Receiver<PipeEvent<RecordEvent>>,
    processor: Processor,
}

impl<P: EventProcessor> ProcessingModule<P> {
    pub fn new(receiver: Receiver<PipeEvent<RecordEvent>>, processor: P) -> Self {
        Self {
            receiver,
            processor,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            match message {
                PipeEvent::Next(record) => {
                    debug!("Processing message {:?}", record);
                    self.processor.process_next(record).await?
                }
            }
        }

        let result = self.processor.finalize().await;
        self.receiver.close();
        result
    }
}
