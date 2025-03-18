use std::{env, path::Path, process::Stdio};

use anyhow::Result;
use sysinfo::{get_current_pid, Signal, System};


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
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
    }

    println!("Spawning");
    #[allow(clippy::zombie_processes)]
    let _ = command.spawn()?;
    println!("Success");
    Ok(())
}
