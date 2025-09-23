use anyhow::Result;
use tracing::error;
use whatawhat::cli::run_cli;

fn main() -> Result<()> {
    run_cli(std::env::args_os()).inspect_err(|e| {
        eprintln!("Error running cli {e}");
        error!("Error running cli {e:?}");
    })?;
    Ok(())
}
