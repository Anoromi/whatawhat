use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Represents an entity responsible for providing dates across application. This can allow it to
/// be used for testing
#[async_trait]
pub trait Clock: Sync + Send + 'static {
    fn time(&self) -> DateTime<Utc>;

    async fn sleep(&self, duration: Duration);

    async fn sleep_until(&self, duration: tokio::time::Instant) {
        tokio::time::sleep_until(duration).await;
    }
}

pub struct DefaultClock;

#[async_trait]
impl Clock for DefaultClock {
    fn time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    async fn sleep_until(&self, duration: tokio::time::Instant) {
        tokio::time::sleep_until(duration).await;
    }
}
