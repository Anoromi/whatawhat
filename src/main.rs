use std::{thread::sleep, time::Duration};

use cfg_if::cfg_if;
use cli::run_cli;

pub mod api;
pub mod data_collection;
pub mod cli;

fn main() {
    run_cli();
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
