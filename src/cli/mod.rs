pub mod process;

use std::{
    env, os::unix::process::CommandExt, path::PathBuf, process::Stdio, thread::sleep, time::Duration
};

use clap::Parser;
use sysinfo::{get_current_pid, System};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init,
    Serve { params: String },
}

pub fn run_cli() {
    let args = Args::parse();
    // println!("{:?}", env::current_exe().unwrap().to_str().unwrap());

    match args {
        Args::Init => {
            run_restart_server();
        }
        Args::Serve { params } => loop {
            println!("Hello there");
            sleep(Duration::from_millis(1000));
        },
    }
}

