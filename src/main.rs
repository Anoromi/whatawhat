use std::{
    thread::sleep,
    time::{Duration, Instant, UNIX_EPOCH},
};

use anyhow::Result;
use cli::run_cli;
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
    // run_cli().await?;
    Ok(())
}

pub unsafe extern "system" fn test2(ctrltype: u32) -> BOOL {
    println!("received something {}", ctrltype);
    true.into()
}

fn get_active() {
    #[cfg(target_os = "windows")]
    {
        use api::windows::active_process::get_active;
        get_active().unwrap();
        sleep(Duration::from_millis(500));
        // hello_there().unwrap();
        // 3i32
    }
    // #[cfg(all(not(any(target_os = "windows"))))]
    #[cfg(target_os = "linux")]
    {
        // std::process::Command::new("Hello").
        let g = 3;
        // 3i32
    }
}
