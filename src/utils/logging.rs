use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::cli::application_default_path;

pub fn enable_logging(application_data_path: Option<&str>) -> Result<()> {
    let application_data_path = match application_data_path {
        Some(v) => PathBuf::from(v),
        None => application_default_path()?.join("logs"),
    };
    let appender =
        tracing_appender::rolling::daily(application_data_path, "app");

    let stdout = std::io::stdout.with_max_level(tracing::Level::TRACE);

    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
            "{}={level}",
            env!("CARGO_PKG_NAME").replace("-", "_"),
        )))
        .with_writer(stdout.and(appender))
        .pretty()
        .init();
    Ok(())
}

pub static TEST_LOGGING: LazyLock<()> = LazyLock::new(|| {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_test_writer().pretty().init()
});
