pub mod output;
pub mod process;
pub mod timeline;

use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use process::{kill_previous_servers, restart_server};
use timeline::{process_timeline_command, TimelineCommand};
use tokio::io;
use tracing::level_filters::LevelFilter;

use crate::{daemon::start_daemon, utils::logging::enable_logging};

#[derive(Parser, Debug)]
#[command(name = "Whatawhat", version, long_about = None)]
#[command(about = "Application for monitoring user activity", long_about = None)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
    #[arg(long, help = "Enable logging")]
    log: bool,
}

#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
enum Commands {
    #[command(about = "Starts a daemon for the application")]
    Init {
        #[arg(
            long,
            help = "Application directory. By default tries to save into $XDG_STATE_HOME or $HOME/.local/state"
        )]
        dir: Option<PathBuf>,
    },
    #[command(about = "Display a timeline of user activity")]
    Timeline {
        #[command(flatten)]
        command: TimelineCommand,
    },
    #[command(
        about = "Run a daemon directly in current console. Used for creating a daemon internally and for debugging"
    )]
    Serve {
        #[arg(
            long,
            help = "Application directory. By default tries to save into $XDG_STATE_HOME or $HOME/.local/state"
        )]
        dir: Option<PathBuf>,
    },
    #[command(about = "Stop currently running daemon.")]
    Stop {},
}

pub async fn run_cli() -> Result<()> {
    let args = Args::parse();

    let logging_level = if args.log {
        Some(LevelFilter::TRACE)
    } else {
        None
    };
    enable_logging(None, logging_level, args.log)?;

    match args.commands {
        Commands::Init { .. } => {
            restart_server()?;
            Ok(())
        }
        Commands::Stop {} => {
            let process_name = env::current_exe().unwrap();
            kill_previous_servers(&process_name);
            Ok(())
        }
        Commands::Serve { .. } => {
            start_daemon(create_application_default_path()?).await?;
            Ok(())
        }
        Commands::Timeline { command } => process_timeline_command(command).await,
    }
}

pub fn create_application_default_path() -> Result<PathBuf> {
    let path = {
        #[cfg(windows)]
        {
            let mut path =
                PathBuf::from(env::var("APPDATA").expect("APPDATA should be present on Windows"));
            path.push("whatawhat");
            path
        }
        #[cfg(target_os = "linux")]
        {
            let mut path = env::var("XDG_STATE_HOME")
                    .map(PathBuf::from)
                    .or_else(|_| {
                        env::var("HOME").map(|home| {
                            let mut path = PathBuf::from(home);
                            path.push(".local/state");
                            path
                        })
                    })
                    .expect("Couldn't find neither XDG_STATE_HOME nor HOME");
            path.push("whatawhat");
            path
        }
    };

    match std::fs::create_dir_all(&path) {
        Ok(_) => Ok(path),
        Err(v) if v.kind() == io::ErrorKind::AlreadyExists => Ok(path),
        Err(v) => Err(v.into()),
    }
}
