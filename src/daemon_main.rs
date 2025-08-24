// This runs daemon on windows without creating a console. Disable during development to see
// stdout.
#![windows_subsystem = "windows"]

use std::
    env::args
;

use anyhow::Result;
use clap::Parser;
use whatawhat::{
    cli::subprocess_window_collector::run_collector,
    daemon::{args::DaemonArgs, start_daemon_with_intervals},
    utils::{
        dir::create_application_default_path,
        logging::{DAEMON_PREFIX, enable_logging},
        runtime::single_thread_runtime,
    },
};

fn main() {
    run_service(args().collect::<Vec<_>>()).unwrap();
}

fn run_service(command_args: Vec<String>) -> Result<()> {
    let args = DaemonArgs::parse_from(&command_args);

    if !args.force {
        #[cfg(feature = "win")]
        {
            let mut command_args = command_args;
            println!("Starting detached process");
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::DETACHED_PROCESS;

            command_args.push("--force".into());
            let process_name = std::env::current_exe()?;
            println!("Process {:?}", process_name);
            let mut command = std::process::Command::new(process_name);
            command.args(command_args.into_iter().skip(1));
            command.creation_flags(DETACHED_PROCESS.0);
            command.stdin(std::process::Stdio::null());
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::null());
            #[allow(clippy::zombie_processes)]
            command.spawn()?;
            println!("Created daemon");
            return Ok(());
        }
        #[cfg(unix)]
        {
            use daemonize::Daemonize;
            use tracing::error;

            let daemonize = Daemonize::new()
                .stdout(daemonize::Stdio::devnull())
                .stderr(daemonize::Stdio::devnull())
                .execute();
            match daemonize {
                daemonize::Outcome::Parent(parent) => {
                    parent
                        .inspect_err(|e| error!("Failed to create daemon on parent side {e:?}"))?;
                    println!("Created daemon");
                    return Ok(());
                }
                daemonize::Outcome::Child(_) => (),
            }
        }
    }

    run(args)
}

fn run_collection_subprocess(args: DaemonArgs) -> Result<()> {
    let collection_interval = args.collect_interval
        .map(|d| std::time::Duration::from_secs(d.num_seconds() as u64))
        .unwrap_or(std::time::Duration::from_secs(1));
    let idle_time = args.idle_time
        .map(|d| std::time::Duration::from_secs(d.num_seconds() as u64))
        .unwrap_or(std::time::Duration::from_secs(5 * 60));

    // Run the collector that writes to stdout
    run_collector(std::io::stdout(), collection_interval, idle_time)?;

    Ok(())
}

fn run(args: DaemonArgs) -> Result<()> {
    // If collect flag is set, run in collection subprocess mode
    if args.collect {
        run_collection_subprocess(args)?;
        return Ok(());
    }

    let app_dir = args.dir.map_or_else(create_application_default_path, Ok)?;
    enable_logging(DAEMON_PREFIX, &app_dir.join("logs"), args.log, args.log_console).unwrap();

    // Convert chrono::Duration to std::time::Duration
    let collection_interval = args.collect_interval
        .map(|d| std::time::Duration::from_secs(d.num_seconds() as u64))
        .unwrap_or(std::time::Duration::from_secs(1));
    let idle_interval = args.idle_time
        .map(|d| std::time::Duration::from_secs(d.num_seconds() as u64))
        .unwrap_or(std::time::Duration::from_secs(5 * 60));

    single_thread_runtime()?.block_on(async move {
        start_daemon_with_intervals(app_dir, collection_interval, idle_interval).await
    })?;
    Ok(())
}
