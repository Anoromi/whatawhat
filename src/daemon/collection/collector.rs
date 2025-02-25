use std::time::Duration;

use anyhow::Result;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};
use tracing::error;

use crate::windows_api::WindowManager;

use super::{afk::AfkEvaluator, producer::WindowData};

pub struct DataCollectionModule<P> {
    next: mpsc::Sender<WindowData>,
    producer: P,
    afk_evaluator: AfkEvaluator,
    collection_frequency: Duration,
}

impl<P: WindowManager> DataCollectionModule<P> {
    pub fn new(
        next: mpsc::Sender<WindowData>,
        producer: P,
        afk_evaluator: AfkEvaluator,
        collection_frequency: Duration,
    ) -> Self {
        Self {
            next,
            producer,
            collection_frequency,
            afk_evaluator,
        }
    }

    fn collect_data(&mut self) -> Result<WindowData> {
        let window_data = self.producer.get_active_window_data()?;
        let idle_ms = self.producer.get_idle_time()?;
        let afk = self.afk_evaluator.is_afk(idle_ms);

        Ok(WindowData {
            window_title: window_data.window_title,
            process_name: window_data.process_name,
            color: None,
            afk,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let execution_start = Instant::now();

        let elapsed = execution_start.elapsed();

        match self.collect_data() {
            Ok(v) => {
                self.next.send(v);
            }
            Err(e) => {
                error!("Encountered an error during execution {}", e)
            }
        }
        sleep(self.collection_frequency - elapsed).await;
        Ok(())
    }
}
