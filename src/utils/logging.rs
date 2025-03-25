use std::{path::{Path, PathBuf}, sync::LazyLock};

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::{format::FmtSpan, writer::MakeWriterExt};

pub const CLI_PREFIX : &str = "cli";
pub const DAEMON_PREFIX : &str = "daemon";

pub fn enable_logging(
    prefix: &str,
    application_data_path: &Path,
    log_level: Option<LevelFilter>,
    show_std: bool,
) -> Result<()> {

    let appender = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::DAILY)
        .max_log_files(5)
        .filename_prefix(prefix)
        .build(application_data_path.join("logs"))?;

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

pub static TEST_LOGGING: LazyLock<()> = LazyLock::new(|| {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_test_writer()
        .pretty()
        .init()
});
