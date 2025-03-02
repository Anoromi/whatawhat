use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::Result;
use chrono::Utc;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};
use tracing::{error, info};

use crate::{
    daemon::{pipeline_event::PipeEvent, storage::record_event::Record},
    windows_api::WindowManager,
};

use super::{afk::AfkEvaluator, producer::WindowData};

pub struct DataCollectionModule {
    next: mpsc::Sender<PipeEvent<Record>>,
    producer: Box<dyn WindowManager>,
    afk_evaluator: AfkEvaluator,
    collection_frequency: Duration,
    time_provider: Box<dyn FnMut() -> chrono::DateTime<Utc>>,
}

impl DataCollectionModule {
    pub fn new(
        next: mpsc::Sender<PipeEvent<Record>>,
        producer: Box<dyn WindowManager>,
        afk_evaluator: AfkEvaluator,
        collection_frequency: Duration,
        time_provider: Box<dyn FnMut() -> chrono::DateTime<Utc>>,
    ) -> Self {
        Self {
            next,
            producer,
            collection_frequency,
            afk_evaluator,
            time_provider,
        }
    }

    fn collect_data(&mut self) -> Result<Record> {
        let window_data = self.producer.get_active_window_data()?;
        let idle_ms = self.producer.get_idle_time()?;
        let afk = self.afk_evaluator.is_afk(idle_ms);
        let timestamp = self.time_provider.deref_mut()();

        Ok(Record {
            window_name: window_data.window_title,
            process_name: window_data.process_name,
            color: None,
            afk,
            timestamp,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let execution_start = Instant::now();

            let elapsed = execution_start.elapsed();

            match self.collect_data() {
                Ok(v) => {
                    info!("Sending message {:?}", v);
                    self.next.send(PipeEvent::Next(v)).await?;
                }
                Err(e) => {
                    error!("Encountered an error during execution {}", e)
                }
            }
            sleep(self.collection_frequency - elapsed).await;
        }
    }
}
