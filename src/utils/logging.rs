use std::{path::Path, sync::LazyLock};

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::{format::FmtSpan, writer::MakeWriterExt};

pub const CLI_PREFIX : &str = "cli";
pub const DAEMON_PREFIX : &str = "daemon";

/// Logging for the application is done in 2 ways. First there's `logs` directory in
/// application_data_path that's used to store logs in files for each hour. Second, user can enable
/// logging into stdout.
pub fn enable_logging(
    prefix: &str,
    path: &Path,
    log_level: Option<LevelFilter>,
    show_std: bool,
) -> Result<()> {

    // HOURLY rotation is used because of how tracing_appender calls pruning. Tracing appender does
    // not delete previous data immediately, only after it needs to change from 1 date to another.
    // This means that for a DAILY rotation the user needs to hit 00:00 to trigger file removal.
    let appender = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::HOURLY)
        .max_log_files(8)
        .filename_prefix(prefix)
        .build(path)?;


    let stdout = std::io::stdout.with_filter(move |_| show_std);

    let level = log_level
        .map(|v| v.to_string())
        .unwrap_or_else(|| std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()));

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
            "{}={level}",
            env!("CARGO_PKG_NAME").replace("-", "_"),
        )))
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(stdout.and(appender))
        .pretty()
        .init();
    Ok(())
}

/// Logger used for testing
pub static TEST_LOGGING: LazyLock<()> = LazyLock::new(|| {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_test_writer()
        .pretty()
        .init()
});
