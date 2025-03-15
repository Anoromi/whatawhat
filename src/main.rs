use anyhow::Result;
use tracing::error;
use whatawhat::cli::run_cli;


#[tokio::main]
async fn main() -> Result<()> {

    run_cli().await.inspect_err(|e| {
        error!("Error running cli {e:?}");
    })?;
    Ok(())
}

