use std::{env, fs::File, path::PathBuf, thread::sleep, time::Duration};

use anyhow::Result;
use cli::{application_default_path, run_cli};
use tracing::error;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use windows_api::{GenericWindowManager, WindowManager};

pub mod api;
pub mod cli;
pub mod daemon;
pub mod fs;
pub mod utils;
pub mod windows_api;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    // unsafe {
    // let thread_id = GetCurrentThreadId();
    // dbg!(GetConsoleWindow());
    // SetWindowsHookExW(WH_CALLWNDPROC, Some(test_process), None, thread_id).unwrap();
    // SetConsoleCtrlHandler(Some(test2), true).unwrap();
    // }
    // print_endlessly();

    // loop {
    //     let _ = dbg!(GenericWindowManager::new()?.get_active_window_data());
    //     sleep(Duration::from_secs(1));
    // }
    // get_active();
    // loop {
    //     is_afk();
    // }
    // enable_logging();
    // tracing_subscriber::FmtSubscriber::
    // let appender = tracing_appender::rolling::daily(get_app_dir()?.join("logs"), "myapp-logs");
    // let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    //
    // let stdout = std::io::stdout.with_max_level(tracing::Level::INFO);
    //
    // tracing_subscriber::fmt()
    //     .with_writer(stdout.and(non_blocking_appender))
    //     .init();
    enable_logging()?;

    run_cli().await.inspect_err(|e| {
        error!("{e:?}");
    })?;
    Ok(())
}

fn enable_logging() -> Result<()> {
    let appender = tracing_appender::rolling::daily(application_default_path().join("logs"), "app");

    let stdout = std::io::stdout.with_max_level(tracing::Level::TRACE);

    tracing_subscriber::fmt()
        .with_writer(stdout.and(appender))
        .pretty()
        .init();
    Ok(())
}

fn init_application_directory() {}

