pub mod output;
pub mod process;
pub mod termination;

use std::{env, ops::Deref, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use clap::{error::ErrorKind, CommandFactory, Parser};
use dateparser::DateTimeUtc;
use futures::StreamExt;
use output::{
    analysis::analyze_processes,
    extract_between,
    sliding_grouping::{sliding_interval_grouping, SlidingInterval, TimeOption},
};
use process::{kill_previous_servers, restart_server};
use tokio::io;
use tracing::{info, instrument};

use crate::{
    daemon::{
        start_daemon,
        storage::{entities::UsageIntervalEntity, record_storage::RecordStorageImpl},
    },
    utils::{
        percentage::Percentage,
        time::{day_start, next_day_start},
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
    // StopProcess {
    //     pid: u32,
    // },
    Today {
        #[command(flatten)]
        command: TodayCommand,
    },
}

#[derive(Debug, Parser)]
struct TodayCommand {
    #[arg(long, short)]
    start: Option<DateTimeUtc>,
    #[arg(long, short)]
    end: Option<DateTimeUtc>,
    #[arg(long = "days", default_value_t = false)]
    treat_as_days: bool,
    #[command(flatten)]
    interval: PrintInterval,
    #[arg(short = 'p', long = "percentage", default_value_t = Percentage::new_opt(10.).unwrap())]
    min_percentage: Percentage,
}

#[derive(Parser, Debug)]
struct DaemonParams {
    #[arg(long)]
    dir: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
struct PrintCurrent {
    #[arg(long, help = "Include afk", default_value_t = false)]
    afk: bool,
    // #[arg(long, default_value_t = Default::default())]
    #[command(flatten)]
    interval: PrintInterval,
}

#[derive(Debug, Clone, Copy, clap::Args)]
#[command(flatten_help = true)]
pub struct PrintInterval {
    #[arg(short)]
    duration: u32,
    #[arg(short)]
    option: TimeOption,
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
        // Args::StopProcess { pid } => {
        // terminate_app(pid)?;
        //     Ok(())
        // }
        Args::Today { command } => process_today(command).await,
    }
}

async fn process_today(
    TodayCommand {
        start,
        end,
        interval,
        treat_as_days,
        min_percentage,
    }: TodayCommand,
) -> Result<()> {
    let Some(interval) = SlidingInterval::new_opt(interval.duration, interval.option) else {
        return Err(Args::command()
            .error(ErrorKind::ValueValidation, "Print interval must be present")
            .into());
    };
    let min_shown_duration = interval.as_duration() * (*min_percentage as i32) / 100;

    let mut start = start
        .map(|v| v.0.with_timezone(&Local))
        .unwrap_or_else(|| Local::now() - interval.as_duration() * 10);

    let mut end = end
        .map(|v| v.0.with_timezone(&Local))
        .unwrap_or(Local::now());

    if treat_as_days {
        start = day_start(start);
        end = next_day_start(end);
    }

    println!(
        "{}\n{}",
        DateTime::<Utc>::from(start),
        DateTime::<Utc>::from(end)
    );
    let application = RecordStorageImpl::new(application_default_path()?.join("records"))?;

    let results = extract_between(
        application,
        output::PrintConfig {
            with_afk: false,
            start: start.into(),
            end: end.into(),
        },
    );

    let v = sliding_interval_grouping::<_, Local>(results, interval, |v| {
        analyze_processes(v, min_shown_duration)
    })
    .await?;

    for (time, value) in v {
        let Some(v) = value else {
            continue;
        };

        let time = time.with_timezone(&Local);

        println!("{}", time.format("%D:%H:%M:%S"),);

        for usage in v {
            println!(
                "{}\t{}\t{}",
                time.format("%D:%H:%M:%S"),
                interval.percentage_for(usage.duration),
                clean_process_name(&usage.name)
            );
        }
        println!();
    }
    // let mut results = std::pin::pin!(results);
    // while let Some(v) = results.next().await {
    //     match v {
    //         Ok(v) => {
    //             print_interval(v);
    //         }
    //         Err(e) => {
    //             println!("Encountered an error {e}")
    //         }
    //     }
    // }
    Ok(())
}

fn format_duration(v: Duration) -> String {
    if v.num_hours() > 0 {
        format!("{} hours {} minutes", v.num_hours(), v.num_minutes() % 60)
    } else if v.num_minutes() > 0 {
        format!("{} minutes", v.num_minutes())
    } else {
        format!("{} seconds", v.num_seconds())
    }
}

fn clean_process_name(value: &str) -> String {
    PathBuf::from(value)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| value.to_string())
}

fn print_interval(interval: UsageIntervalEntity) {
    let process_name = PathBuf::from(interval.process_name.deref())
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| interval.process_name.to_string());

    let end: DateTime<Local> = interval.end().into();
    let start: DateTime<Local> = interval.start.into();
    let duration = format_duration(interval.duration);

    let window_name = interval.window_name;

    println!(
        "{}\t{}\t{}\t{}",
        start.format("%H:%M:%S"),
        duration,
        process_name,
        window_name
    )
}

pub fn application_default_path() -> Result<PathBuf> {
    let path = {
        #[cfg(windows)]
        {
            let mut path =
                PathBuf::from(env::var("APPDATA").expect("APPDATA should be present on Windows"));
            path.push("whatawhat");
        }
        #[cfg(target_os = "linux")]
        {
            let path = PathBuf::from(
                env::var("XDG_STATE_HOME")
                    .or_else(|_| env::var("HOME").map(|home| home + ".local/state"))
                    .expect("Couldn't find neither XDG_STATE_HOME nor HOME"),
            );
            path
        }
    };

    match std::fs::create_dir(&path) {
        Ok(_) => Ok(path),
        Err(v) if v.kind() == io::ErrorKind::AlreadyExists => Ok(path),
        Err(v) => return Err(v.into()),
    }
}
