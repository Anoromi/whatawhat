
use anyhow::Result;
use listener::receive_interprocess_messages;
use tokio::select;
use tracing::{info, warn};

pub mod listener;
pub mod termination;

pub const DAEMON_CHANNEL_NAME: &str = "PROCESS_DETECTOR_NOTIFICATION.sock";

pub async fn detect_messages() -> Result<()> {
    let hehe = select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("Received ctrl_c event");
        },
    };

    Ok(())
}
