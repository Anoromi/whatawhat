pub mod process;
pub mod termination;

use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use process::{kill_previous_servers, restart_server};
use termination::gracefuly_terminate;
use tokio::io;
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
    StopProcess {
        pid: u32,
    },
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
        Args::Stop {} => {
            let process_name = env::current_exe().unwrap();
            kill_previous_servers(&process_name);
            info!("Success");
            Ok(())
        }
        // used internally to run the process in daemon mode
        Args::Serve { .. } => {
            start_daemon(application_default_path()?).await?;
            Ok(())
        }
        // used internally to stop processes
        Args::StopProcess { pid } => {
            gracefuly_terminate(pid)?;
            Ok(())
        }
    }
}

pub fn application_default_path() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        let mut path =
            PathBuf::from(env::var("APPDATA").expect("APPDATA should be present on Windows"));
        path.push("whatawhat");
        match std::fs::create_dir(&path) {
            Ok(_) => {}
            Err(v) if v.kind() == io::ErrorKind::AlreadyExists => {}
            Err(v) => return Err(v.into())
        };
        Ok(path)
    }
}
