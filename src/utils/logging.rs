use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::cli::create_application_default_path;

pub fn enable_logging(
    application_data_path: Option<&str>,
    log_level: Option<LevelFilter>,
    show_std: bool
) -> Result<()> {
    let application_data_path = match application_data_path {
        Some(v) => PathBuf::from(v),
        None => create_application_default_path()?.join("logs"),
    };
    let appender = tracing_appender::rolling::daily(application_data_path, "app");

    let stdout = std::io::stdout.with_filter(move |_| show_std);

    let level =
        log_level.map(|v| v.to_string()).unwrap_or_else(|| std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()));

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
        .with_test_writer()
        .pretty()
        .init()
});
