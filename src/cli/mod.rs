pub mod output;
pub mod process;
pub mod termination;

use std::{env, ops::Deref, path::PathBuf, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta, Utc};
use clap::Parser;
use futures::StreamExt;
use output::extract_between;
use process::{kill_previous_servers, restart_server};
use termination::gracefuly_terminate;
use tokio::io;
use tracing::{info, instrument};

use crate::daemon::{
    start_daemon,
    storage::{
        entities::UsageIntervalEntity,
        record_storage::{RecordStorage, RecordStorageImpl},
    },
};

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
    Current {
        #[command(flatten)]
        params: PrintCurrent,
    },
}

#[derive(Parser, Debug)]
struct DaemonParams {
    #[arg(long)]
    dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct PrintCurrent {
    #[arg(long, default_value_t = 0u32)]
    min_duration: u32,
    #[arg(long, help = "Include afk", default_value_t = false)]
    afk: bool,
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
        Args::Serve { .. } => {
            start_daemon(application_default_path()?).await?;
            Ok(())
        }
        Args::StopProcess { pid } => {
            gracefuly_terminate(pid)?;
            Ok(())
        }
        Args::Current { params } => {
            let application = RecordStorageImpl::new(application_default_path()?.join("records"))?;

            let mut results = extract_between(
                application,
                Utc::now(),
                Utc::now(),
                output::PrintConfig {
                    min_duration: TimeDelta::minutes(params.min_duration.into()),
                    with_afk: params.afk,
                },
            );

            while let Some(v) = results.next().await {
                // println!("Some");
                match v {
                    Ok(v) => {
                        print_interval(v);
                        // println!("{} {} {} {}", v.process_name, v.window_name, v.start, v.end)
                    }
                    Err(e) => {
                        println!("Encountered an error {e}")
                    }
                }
            }
            Ok(())
        }
    }
}

fn print_interval(interval: UsageIntervalEntity) {
    let process_name = PathBuf::from(interval.process_name.deref())
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| interval.process_name.to_string());

    let window_name = interval.window_name;
    let start: DateTime<Utc> = interval.start.into();
    let end: DateTime<Utc> = interval.end.into();
    //DateTime::<Local>::from_naive_utc_and_offset(interva, offset)

    // let start = Local::from (interval.start);
    println!(
        "{}\t{window_name}\t{}\t{}",
        process_name,
        start.format("%H:%M"),
        end.format("%H:%M")
    )
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
            Err(v) => return Err(v.into()),
        };
        Ok(path)
    }
}
