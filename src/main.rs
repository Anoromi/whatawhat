use std::{
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use cli::run_cli;
use env_logger::Target;
use windows::Win32::Foundation::BOOL;

pub mod api;
pub mod cli;
pub mod data_collection;
pub mod server;

#[tokio::main]
async fn main() -> Result<()> {
    // unsafe {
    // let thread_id = GetCurrentThreadId();
    // dbg!(GetConsoleWindow());
    // SetWindowsHookExW(WH_CALLWNDPROC, Some(test_process), None, thread_id).unwrap();
    // SetConsoleCtrlHandler(Some(test2), true).unwrap();
    // }
    // print_endlessly();

    // get_active();
    enable_logging();
    run_cli().await?;
    Ok(())
}

fn enable_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();
}
