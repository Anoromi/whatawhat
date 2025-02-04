
use anyhow::Result;
use listener::receive_interprocess_messages;
use tokio::select;

pub mod listener;

pub const DAEMON_CHANNEL_NAME: &str = "PROCESS_DETECTOR_NOTIFICATION.sock";

async fn detect_messages() -> Result<()> {
    let hehe = select! {
        _ = tokio::signal::ctrl_c() => {
            let cat = 3;
        },
        _ = receive_interprocess_messages() => {
        }
    };

    Ok(())
}
