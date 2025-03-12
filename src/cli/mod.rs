pub mod output;
pub mod process;

use std::{env, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Duration, Local, TimeDelta, Utc};
use clap::{error::ErrorKind, CommandFactory, Parser};
use dateparser::DateTimeUtc;
use futures::Stream;
use output::{
    analysis::{analyze_processes, analyze_windows},
    extract_between,
    sliding_grouping::{sliding_interval_grouping, SlidingInterval, TimeOption},
};
use process::{kill_previous_servers, restart_server};
use tokio::io;
use tracing::info;

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
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Serve {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Stop {},
    Timeline {
        #[command(flatten)]
        command: TimelineCommand,
    },
}

#[derive(Debug, Parser)]
struct TimelineCommand {
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
    #[arg(short, long = "window", default_value_t = false)]
    use_window_name: bool,

    #[arg(short, long, help = "Include afk time")]
    afk: bool,
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

pub async fn run_cli() -> Result<()> {
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
            Ok(())
        }
        Args::Serve { .. } => {
            start_daemon(application_default_path()?).await?;
            Ok(())
        }
        Args::Timeline { command } => process_timeline_command(command).await,
    }
}

async fn process_timeline_command(
    TimelineCommand {
        start,
        end,
        interval,
        treat_as_days,
        min_percentage,
        use_window_name,
        afk,
    }: TimelineCommand,
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
            with_afk: afk,
            start: start.into(),
            end: end.into(),
        },
    );

    if use_window_name {
        print_window_grouping(interval, min_shown_duration, results).await?;
    } else {
        print_processes_grouping(interval, min_shown_duration, results).await?;
    }
    Ok(())
}

async fn print_processes_grouping(
    interval: SlidingInterval,
    min_shown_duration: TimeDelta,
    results: impl Stream<Item = std::result::Result<UsageIntervalEntity, anyhow::Error>>,
) -> Result<()> {
    let intervals = sliding_interval_grouping::<_, Local>(results, interval, |v| {
        analyze_processes(v, interval.as_duration(), min_shown_duration)
    })
    .await?;
    for (time, value) in intervals {
        let Some(analyzed) = value else {
            continue;
        };

        let time = time.with_timezone(&Local);

        if analyzed.is_empty() {
            println!("{}", time.format("%D %H:%M:%S"),);
        } else {
            let mut overall = Duration::zero();
            for usage in analyzed {
                println!(
                    "{}\t{}%\t{}\t{}",
                    time.format("%D %H:%M:%S"),
                    *interval.percentage_for(usage.duration) as i32,
                    format_duration(usage.duration),
                    clean_process_name(&usage.process_name)
                );
                overall += usage.duration
            }
            // println!("{}\t{}%\t{}\tInactive", time.format("%D %H:%M:%S"), *interval.percentage_for(overall) as i32, format_duration(overall));
        }
        println!();
    }
    Ok(())
}

async fn print_window_grouping(
    interval: SlidingInterval,
    min_shown_duration: TimeDelta,
    results: impl Stream<Item = std::result::Result<UsageIntervalEntity, anyhow::Error>>,
) -> Result<()> {
    let intervals = sliding_interval_grouping::<_, Local>(results, interval, |v| {
        analyze_windows(v, min_shown_duration)
    })
    .await?;
    for (time, value) in intervals {
        let Some(analyzed) = value else {
            continue;
        };

        let time = time.with_timezone(&Local);

        if analyzed.is_empty() {
            println!("{}", time.format("%D %H:%M:%S"),);
        }

        for usage in analyzed {
            println!(
                "{}\t{}%\t{}\t{}\t{}",
                time.format("%D %H:%M:%S"),
                *interval.percentage_for(usage.duration) as u32,
                format_duration(usage.duration),
                clean_process_name(&usage.process_name),
                usage.window_name,
            );
        }
        println!();
    }
    Ok(())
}

fn format_duration(v: Duration) -> String {
    if v.num_hours() > 0 {
        format!(
            "{}h{}m{}s",
            v.num_hours(),
            v.num_minutes() % 60,
            v.num_seconds() % 60
        )
    } else if v.num_minutes() > 0 {
        format!("{}m{}s", v.num_minutes() % 60, v.num_seconds() % 60)
    } else {
        format!("{}s", v.num_seconds() % 60)
    }
}

fn clean_process_name(value: &str) -> String {
    PathBuf::from(value)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| value.to_string())
}

pub fn application_default_path() -> Result<PathBuf> {
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
        Err(v) => Err(v.into()),
    }
}
