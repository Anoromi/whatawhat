pub mod process;
pub mod termination;

use anyhow::Result;
use clap::Parser;
use process::restart_server;

use crate::daemon::start_server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init,
    Serve,
}

pub async fn run_cli() -> Result<()> {
    let args = Args::parse();

    match args {
        Args::Init => {
            restart_server();
            Ok(())
        }
        Args::Serve => {
            start_server().await?;
            Ok(())
        }
    }
}
