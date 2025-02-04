
use anyhow::Result;
use cli::run_cli;
use data_collection::windows::is_afk;
use env_logger::Target;

pub mod api;
pub mod cli;
pub mod data_collection;
pub mod daemon;
pub mod utils;

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
    loop {
        is_afk();
    }
    // enable_logging();
    // run_cli().await?;
    Ok(())
}

fn enable_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();
}
