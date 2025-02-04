use std::time::Duration;

use anyhow::Result;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};

use super::producer::{WindowData, WindowDataProvider};

pub struct DataCollectionModule<P> {
    next: mpsc::Sender<WindowData>,
    producer: P,
    collect_frequency: Duration,
}

impl<P: WindowDataProvider> DataCollectionModule<P> {
    pub fn new(next: mpsc::Sender<WindowData>, producer: P, collect_frequency: Duration) -> Self {
        Self {
            next,
            producer,
            collect_frequency,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let execution_start = Instant::now();

        let elapsed = execution_start.elapsed();

        self.next.send(self.producer.get_active_window_data()?);

        sleep(self.collect_frequency - elapsed);
        Ok(())
    }
}
