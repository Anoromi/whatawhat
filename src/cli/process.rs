use std::{env, path::Path, process::Stdio};

use anyhow::Result;
use daemonize::Daemonize;
use sysinfo::{Signal, System, get_current_pid};

use crate::daemon::start_daemon;

use super::create_application_default_path;

pub fn kill_previous_servers(name: &Path) {
    let system = System::new_all();
    let current_id = get_current_pid().unwrap();
    for (pid, process) in system.processes().iter() {
        if *pid == current_id {
            continue;
        }
        if matches!(process.parent(), Some(p) if p == current_id) {
            continue;
        }

        if process
            .exe()
            .filter(|v| v.exists())
            .filter(|v| name == *v)
            .is_some()
        {
            // This will forcefully terminate the process on Windows. Anything better will require a
            // lot more work.
            if process.kill_with(Signal::Term).is_none() {
                process.kill();
            }
            process.wait();
        }
    }
}

/// Intended for shutting down previous server and starting new one. Currently for simplicity sake
/// it operates using a detached process. This is not great but it's not as hard to configure.
pub fn restart_server() -> Result<()> {
    // The program use executable passed into the process. It's not the best option but it will do
    // the job in most cases.
    let process_name = env::current_exe().expect("Can't operate without an executable");
    kill_previous_servers(&process_name);
    #[cfg(unix)]
    {
        run_linux();
    }

    let mut command = std::process::Command::new(process_name);
    command.args(["serve"]);

    #[cfg(feature = "win")]
    {
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::DETACHED_PROCESS;
        command.creation_flags(DETACHED_PROCESS.0);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
    }

    println!("Spawning");
    #[allow(clippy::zombie_processes)]
    let _ = command.spawn()?;
    println!("Success");
    Ok(())
}

fn run_linux() {
    let daemonize = Daemonize::new()
        //.pid_file("/tmp/test.pid") // Every method except `new` and `start`
        //.chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/") // for default behaviour.
        .group(0)        // or group id.
        .umask(0o777)    // Set umask, `0o027` by default.
        .stdout(daemonize::Stdio::devnull())  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(daemonize::Stdio::devnull())  // Redirect stderr to `/tmp/daemon.err`.
        ;
    daemonize.start().unwrap();
    tokio::runtime::Builder::new_multi_thread()
        .build()
        .unwrap()
        .block_on(async {
            start_daemon(create_application_default_path().unwrap()).await.unwrap();
        });
}
