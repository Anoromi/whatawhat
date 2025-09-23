use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, info_span, Instrument};
use whatawhat_lib::WindowManager;

use crate::{daemon::storage::record_event::RecordEvent, utils::clock::Clock};

pub struct DataCollectionModule {
    next: mpsc::Sender<RecordEvent>,
    producer: Box<dyn WindowManager + Send + Sync>,
    shutdown: CancellationToken,
    collection_frequency: Duration,
    time_provider: Box<dyn Clock>,
}

impl DataCollectionModule {
    pub fn new(
        next: mpsc::Sender<RecordEvent>,
        producer: Box<dyn WindowManager + Send + Sync>,
        shutdown: CancellationToken,
        collection_frequency: Duration,
        time_provider: Box<dyn Clock>,
    ) -> Self {
        Self {
            next,
            producer,
            collection_frequency,
            time_provider,
            shutdown,
        }
    }

    async fn collect_data(&mut self) -> Result<RecordEvent> {
        let window_data = self.producer.get_active_window_data()?;
        let afk = self.producer.is_idle()?;
        let timestamp = self.time_provider.time();

        Ok(RecordEvent {
            window_name: window_data.window_title,
            process_name: window_data.process_path,
            afk,
            timestamp,
            app_identifier: window_data.app_identifier,
            application_name: window_data.app_name,
        })
    }

    /// Executes the collector event loop.
    pub async fn run(mut self) -> Result<()> {
        let mut collection_point = self.time_provider.instant();
        loop {
            collection_point += self.collection_frequency;

            match self.collect_data().await {
                Ok(record) => {
                    let span = info_span!("Processing collected data");
                    debug!("Sending message {:?}", record);
                    self.next
                        .send(record)
                        .instrument(span)
                        .await
                        .inspect_err(|e| error!("Unexpected error during sending {e:?}"))?;
                    info!("Successfully sent message")
                }
                Err(e) => {
                    error!("Encountered an error during collection {:?}", e)
                }
            }

            tokio::select! {
                // Cancelation means we stop execution of the event loop. Which means we also drop
                // the sender channel and consequently stop processing module.
                _ = self.shutdown.cancelled() => {
                    return Ok(())
                }
                _ = self.time_provider.sleep_until(collection_point) => ()
            }
        }
    }
}
