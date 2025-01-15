pub mod process;
pub mod termination;

use std::{
    io::{BufRead, BufReader, Write},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use interprocess::local_socket::{
    prelude::*, traits::ListenerExt, GenericNamespaced, ListenerOptions, Stream, ToNsName,
};
use process::restart_server;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{CallNextHookEx, SetWindowsHookExW, WH_CALLWNDPROC},
};

use crate::server::start_server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init,
    Serve,
    Test,
}

pub async fn run_cli() -> Result<()> {
    let args = Args::parse();

    match args {
        Args::Init => {
            let _ = dbg!(send());

            restart_server();
            Ok(())
        }
        Args::Serve => {
            loop {
                sleep(Duration::from_millis(500));
                // receive()?;
                start_server();

            }
        }
        Args::Test => {
            unsafe {
                SetWindowsHookExW(WH_CALLWNDPROC, Some(test_process), None, 0);
            }
            print_endlessly();
        }
    }
}

pub unsafe extern "system" fn test_process(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    println!("received something");
    CallNextHookEx(None, code, wparam, lparam)
}

pub fn print_endlessly() -> ! {
    loop {
        println!("Hello there");
        sleep(Duration::from_millis(1000));
    }
}

// pub fn send() -> Result<()> {
//     let name = PRINTNAME.to_ns_name::<GenericNamespaced>()?;
//     let _buffer = String::with_capacity(128);
//     let conn = Stream::connect(name)?;
//     let mut conn = BufReader::new(conn);
//     conn.get_mut().write_all("Hello there".as_bytes())?;
//     Ok(())
// }

