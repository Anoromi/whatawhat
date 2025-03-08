use std::env;

use anyhow::Result;
use cli::{application_default_path, run_cli};
use tracing::error;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub mod api;
pub mod cli;
pub mod daemon;
pub mod fs;
pub mod utils;
pub mod windows_api;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    enable_logging()?;

    run_cli().await.inspect_err(|e| {
        error!("{e:?}");
    })?;
    Ok(())
}

fn enable_logging() -> Result<()> {
    let appender =
        tracing_appender::rolling::daily(application_default_path()?.join("logs"), "app");

    let stdout = std::io::stdout.with_max_level(tracing::Level::TRACE);

    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
            "{}={level}",
            env!("CARGO_PKG_NAME").replace("-", "_"),
        )))
        .with_writer(stdout.and(appender))
        .pretty()
        .init();
    Ok(())
}
