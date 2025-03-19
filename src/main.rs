use anyhow::Result;
use tracing::error;
use whatawhat::cli::run_cli;


//#[tokio::main]
fn main() -> Result<()> {

    run_cli().inspect_err(|e| {
        error!("Error running cli {e:?}");
    })?;
    Ok(())
}

