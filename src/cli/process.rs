use std::{
    backtrace::{Backtrace, BacktraceStatus},
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;
use sysinfo::{get_current_pid, System};
use tracing::{error, info};

use crate::cli::termination::gracefuly_terminate;

pub fn kill_previous_servers(name: &Path) {
    let system = System::new_all();
    let current_id = get_current_pid().unwrap();
    for (pid, process) in system.processes().iter() {
        // println!("Process {:?}", process.exe());
        // println!("This id {:?}", this_pid);
        // println!("This id {:?}", process);
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
            info!("It happened");
            // process.kill();
            let process_name = env::current_exe().expect("Can't operate without an excutable");
            let mut command = std::process::Command::new(process_name);
            command.args(["stop-process", &pid.as_u32().to_string()]);
            command.spawn().unwrap().wait().unwrap();


            // let v = Proce
            // gracefuly_terminate(pid.as_u32()).inspect_err(|e| {
            //     error!("{:?} {}", e, e.backtrace());
            // });
            info!("Waiting to die");

            // TODO Gracefuly kill
            // process.kill_with(Signal::Term);
            // process.kill();
            process.wait();
        }
    }
}

pub fn restart_server() -> Result<()> {
    let process_name = env::current_exe().expect("Can't operate without an excutable");
    // kill_previous_servers(&process_name);
    let mut command = std::process::Command::new(process_name);
    command.args(["serve"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::CREATE_NEW_CONSOLE;

        command.creation_flags(CREATE_NEW_CONSOLE.0);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    println!("Spawning");
    #[allow(clippy::zombie_processes)]
    let ch = command.spawn()?;
    Ok(())
}
