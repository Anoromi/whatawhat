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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init,
    Serve { params: String },
    Test,
}

pub async fn run_cli() -> Result<()> {
    let args = Args::parse();
    // println!("{:?}", env::current_exe().unwrap().to_str().unwrap());

    match args {
        Args::Init => {
            let _ = dbg!(send());

            restart_server();
            Ok(())
        }
        Args::Serve { params } => {
            println!("Hello there");
            loop {
                sleep(Duration::from_millis(500));
                let _ = dbg!(receive());
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

const PRINTNAME: &str = "PROCESS_DETECTOR_COLLECOR.sock";
pub fn send() -> Result<()> {
    let name = PRINTNAME.to_ns_name::<GenericNamespaced>()?;
    let _buffer = String::with_capacity(128);
    let conn = Stream::connect(name)?;
    let mut conn = BufReader::new(conn);
    conn.get_mut().write_all("Hello there".as_bytes())?;
    Ok(())
}

pub fn receive() -> Result<()> {
    let name = PRINTNAME.to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_sync()?;
    let mut buffer = String::with_capacity(128);

    println!("hehe hehe hehe");
    for conn in listener.incoming() {
        // Wrap the connection into a buffered receiver right away
        // so that we could receive a single line from it.
        let mut conn = BufReader::new(conn?);
        println!("Incoming connection!");

        // Since our client example sends first, the server should receive a line and only then
        // send a response. Otherwise, because receiving from and sending to a connection cannot
        // be simultaneous without threads or async, we can deadlock the two processes by having
        // both sides wait for the send buffer to be emptied by the other.
        conn.read_line(&mut buffer)?;

        // // Now that the receive has come through and the client is waiting on the server's send, do
        // // it. (`.get_mut()` is to get the sender, `BufReader` doesn't implement a pass-through
        // // `Write`.)
        // conn.get_mut().write_all(b"Hello from server!\n")?;

        // Print out the result, getting the newline for free!
        print!("Client answered: {buffer}");

        // Clear the buffer so that the next iteration will display new data instead of messages
        // stacking on top of one another.
        buffer.clear();
    }

    // let console_window = unsafe { GetConsoleWindow() };
    // unsafe {
    //     // let k = dbg!(ShowWindow(console_window, SW_HIDE));
    //     // let t: isize = 0x00000008;
    //     // dbg!(SetWindowLongPtrA(
    //     //     console_window,
    //     //     GWL_EXSTYLE,
    //     //     t // transmute::<_, i32>(WS_EX_NOACTIVATE.0).into()
    //     // ));
    //     // sleep(Duration::from_secs(10));
    //     // k.unwrap();
    // };
    println!("Done");

    // unsafe { AllocConsole().unwrap() }
    loop {
        // unsafe { FreeConsole().unwrap() };
        println!("Hello there");
        sleep(Duration::from_millis(1000));
    }
}
