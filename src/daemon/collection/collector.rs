use std::time::Duration;

use anyhow::Result;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{
    daemon::storage::record_event::RecordEvent, utils::clock::Clock,
    window_api::WindowManager,
};

use super::afk::AfkEvaluator;

pub struct DataCollectionModule {
    next: mpsc::Sender<RecordEvent>,
    producer: Box<dyn WindowManager>,
    shutdown: CancellationToken,
    afk_evaluator: AfkEvaluator,
    collection_frequency: Duration,
    time_provider: Box<dyn Clock>,
}

impl DataCollectionModule {
    pub fn new(
        next: mpsc::Sender<RecordEvent>,
        producer: Box<dyn WindowManager>,
        shutdown: CancellationToken,
        afk_evaluator: AfkEvaluator,
        collection_frequency: Duration,
        time_provider: Box<dyn Clock>,
    ) -> Self {
        Self {
            next,
            producer,
            collection_frequency,
            afk_evaluator,
            time_provider,
            shutdown,
        }
    }

    fn collect_data(&mut self) -> Result<RecordEvent> {
        let window_data = self.producer.get_active_window_data()?;
        let idle_ms = self.producer.get_idle_time()?;
        let afk = self.afk_evaluator.is_afk(idle_ms);
        let timestamp = self.time_provider.time();

        Ok(RecordEvent {
            window_name: window_data.window_title,
            process_name: window_data.process_name,
            afk,
            timestamp,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let execution_start = Instant::now();

            let elapsed = execution_start.elapsed();

            match self.collect_data() {
                Ok(record) => {
                    debug!("Sending message {:?}", record);
                    self.next.send(record).await?;
                }
                Err(e) => {
                    error!("Encountered an error during collection {:?}", e)
                }
            }
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    return Ok(())
                }
                _ = sleep(self.collection_frequency - elapsed) => ()
            }
        }
    }
}
