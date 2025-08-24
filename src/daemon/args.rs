use std::{path::PathBuf};

use chrono::Duration;
use clap::Parser;
use tracing::level_filters::LevelFilter;


#[derive(Parser)]
pub struct DaemonArgs {
  #[arg(long)]
  pub force: bool,
  #[arg(long)]
  pub dir: Option<PathBuf>,
  /// This option is for debugging purposes only.
  #[arg(long = "log-console")]
  pub log_console : bool,
  #[arg(long = "log-filter")]
  pub log: Option<LevelFilter>,
  #[arg(long = "collect")]
  pub collect: bool,
  #[arg(long = "collect-interval", value_parser = parse_duration)]
  pub collect_interval: Option<Duration>,
  #[arg(long = "idle-time", value_parser = parse_duration)]
  pub idle_time: Option<Duration>,
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(Duration::seconds(seconds))
}