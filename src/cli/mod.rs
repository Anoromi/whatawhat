pub mod process;
pub mod termination;

use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use process::{kill_previous_servers, restart_server};
use tracing::{info, instrument};

use crate::daemon::start_daemon;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init {
        #[clap(flatten)]
        default_params: DaemonParams,
    },
    Serve {
        #[clap(flatten)]
        default_params: DaemonParams,
    },
    Stop {},
}

#[derive(Parser, Debug)]
struct DaemonParams {
    #[arg(long)]
    dir: Option<PathBuf>,
}

#[instrument(level = "error")]
pub async fn run_cli() -> Result<()> {
    info!("Hello there");
    let args = Args::parse();

    dbg!(&args);

    match args {
        Args::Init { .. } => {
            restart_server()?;
            Ok(())
        }
        Args::Serve { .. } => {
            start_daemon().await?;
            Ok(())
        }
        Args::Stop {} => {
            let process_name = env::current_exe().unwrap();
            kill_previous_servers(&process_name);
            info!("Success");
            Ok(())
        }
    }
}
