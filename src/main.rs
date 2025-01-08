use std::{thread::sleep, time::Duration};

use cfg_if::cfg_if;

pub mod data_collection;
pub mod api;
pub mod process;

fn main() {
    let k: i32 = {
        #[cfg(target_os = "windows")]
        {

            use api::windows::active_process::get_active;
            get_active().unwrap();
            sleep(Duration::from_millis(500));
            // hello_there().unwrap();
            3i32
        }
        // #[cfg(all(not(any(target_os = "windows"))))]
        #[cfg(target_os = "linux")]
        {
            // std::process::Command::new("Hello").
            let g = 3;
            3i32
        }

        // cfg_if!(
        //     if #[cfg(target_os = "windows")] {
        //         {
        //         use winstuff::hello_there;
        //         hello_there();
        //         3i32
        //         }
        //     }
        //     else {
        //         {
        //         let g = 3;
        //         // g.saturating_add(34);
        //         3i32
        //         }
        //     }
        // )
    };
}
