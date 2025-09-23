use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::time::Instant;

/// Represents an entity responsible for providing dates across application. This can allow it to
/// be used for testing
#[async_trait]
pub trait Clock: Sync + Send + 'static {
    fn time(&self) -> DateTime<Utc>;

    fn instant(&self) -> Instant;

    async fn sleep(&self, duration: Duration);

    async fn sleep_until(&self, instant: tokio::time::Instant);
}

pub struct DefaultClock;

#[async_trait]
impl Clock for DefaultClock {
    fn time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn instant(&self) -> Instant {
        Instant::now()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    async fn sleep_until(&self, instant: tokio::time::Instant) {
        tokio::time::sleep_until(instant).await;
    }
}
