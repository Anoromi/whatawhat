pub mod daemon_path;
pub mod output;
pub mod process;
pub mod timeline;

use std::{env, ffi::OsString, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use daemon_path::to_daemon_path;
use process::{kill_previous_daemons, restart_daemon};
use timeline::{TimelineCommand, process_timeline_command};
use tracing::level_filters::LevelFilter;

use crate::utils::{
        dir::create_application_default_path,
        logging::{CLI_PREFIX, enable_logging},
        runtime::multi_thread_runtime,
    };

#[derive(Parser, Debug)]
#[command(name = "Whatawhat", version, long_about = None)]
#[command(about = "Application for monitoring user activity", long_about = None)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
    #[arg(long, help = "Enable logging")]
    log: bool,
    #[arg(
        long,
        help = "Application directory. By default tries to save into $XDG_STATE_HOME or $HOME/.local/state on Linux. Or %APPDATA% on Windows"
    )]
    dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
enum Commands {
    #[command(about = "Starts a daemon for the application")]
    Restart {},
    #[command(about = "Display a timeline of user activity")]
    Timeline {
        #[command(flatten)]
        command: TimelineCommand,
    },
    #[command(about = "Stop currently running daemon.")]
    Stop {},
}

pub fn run_cli(values: impl Iterator<Item = OsString>) -> Result<()> {
    let args = match Args::try_parse_from(values) {
        Ok(v) => v,
        Err(e) => {
            e.exit();
        }
    };

    let app_dir = args
        .dir
        .map_or_else(create_application_default_path, Ok)?;

    let logging_level = if args.log {
        Some(LevelFilter::TRACE)
    } else {
        None
    };

    enable_logging(CLI_PREFIX, &app_dir.join("logs"), logging_level, args.log)?;

    match args.commands {
        Commands::Restart { .. } => {
            restart_daemon()?;
            Ok(())
        }
        Commands::Stop {} => {
            stop_daemon();
            Ok(())
        }
        Commands::Timeline { command } => {
            multi_thread_runtime()?
                .block_on(async move { process_timeline_command(command).await })?;
            Ok(())
        }
    }
}

fn stop_daemon() {
    let process_name =
        to_daemon_path(env::current_exe().expect("Failed to get current executable"));
    println!("Inferred daemon name {process_name:?}");
    match kill_previous_daemons(&process_name) {
        Ok(_) => {
            println!("Previous daemons killed")
        },
        Err(e) => {
            eprintln!("Failed killing daemons {e}")
        },
    };
}
