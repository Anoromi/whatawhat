// #![windows_subsystem = "windows"]

use std::{
    collections::VecDeque,
    env::{args, current_exe},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use whatawhat::{
    daemon::{args::DaemonArgs, start_daemon},
    utils::{
        dir::create_application_default_path,
        logging::{DAEMON_PREFIX, enable_logging},
        runtime::single_thread_runtime,
    },
};

fn main() {
    run_service(args().collect::<Vec<_>>()).unwrap();
}

fn run_service(mut command_args: Vec<String>) -> Result<()> {
    let args = DaemonArgs::parse_from(&command_args);

    if !args.force {
        #[cfg(feature = "win")]
        {
            println!("Starting detached process");
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::DETACHED_PROCESS;

            command_args.push("--force".into());
            let process_name = current_exe()?;
            println!("{:?}", process_name);
            let mut command = std::process::Command::new(process_name);
            command.args(command_args.into_iter().skip(1));
            command.creation_flags(DETACHED_PROCESS.0);
            command.stdin(std::process::Stdio::null());
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::null());
            #[allow(clippy::zombie_processes)]
            command.spawn()?;
            return Ok(());
        }
        #[cfg(unix)]
        {
            use daemonize::Daemonize;
            use tracing::{error, info};

            let daemonize = Daemonize::new()
                .stdout(daemonize::Stdio::devnull())
                .stderr(daemonize::Stdio::devnull())
                .execute();
            match daemonize {
                daemonize::Outcome::Parent(parent) => {
                    parent
                        .inspect_err(|e| error!("Failed to create daemon on parent side {e:?}"))?;
                    println!("Created");
                    return std::process::exit(0);
                }
                daemonize::Outcome::Child(_) => (),
            }
        }
    }

    run(args)
}

fn run(args: DaemonArgs) -> Result<()> {
    let app_dir = args.dir.map_or_else(create_application_default_path, Ok)?;
    enable_logging(DAEMON_PREFIX, &app_dir, args.log, args.log_console).unwrap();
    single_thread_runtime()?.block_on(async move { start_daemon(app_dir).await })?;
    Ok(())
}
