use std::path::PathBuf;

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
  pub log: Option<LevelFilter>
}
