use std::env;

use anyhow::Result;
use cli::run_cli;
use tracing::error;
use utils::logging::enable_logging;

pub mod cli;
pub mod daemon;
pub mod fs;
pub mod utils;
pub mod windows_api;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    enable_logging(None)?;

    run_cli().await.inspect_err(|e| {
        error!("{e:?}");
    })?;
    Ok(())
}

