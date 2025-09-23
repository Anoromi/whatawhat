use std::{
    env,
    path::Path,
    process::Command,
};

use anyhow::{anyhow, Result};
use sysinfo::{Signal, System, get_current_pid};
use tracing::error;

use crate::cli::daemon_path::to_daemon_path;


/// Returns all the running processes that aren't this process. Realistically there should only one
/// element, but to account for errors it returns an iterator
pub fn find_servers<'a>(
    system: &'a System,
    name: &'a Path,
) -> impl Iterator<Item = (&'a sysinfo::Pid, &'a sysinfo::Process)> {
    let current_id = get_current_pid().unwrap();
    system.processes().iter().filter(move |(pid, process)| {
        if **pid == current_id {
            return false;
        }
        if matches!(process.parent(), Some(p) if p == current_id) {
            return false;
        }
        process.exe().filter(|v| name == *v).is_some()
    })
}

pub fn kill_previous_daemons(name: &Path) -> Result<()> {
    let system = System::new_all();
    for (pid, process) in find_servers(&system, name) {
        println!("Killing process {pid}");
        if process.kill_with(Signal::Term).is_none() {
            // Windows doesn't support Signals, so forced termination is the only simple option.
            if !process.kill() {
                return Err(anyhow!("Failed killing process {pid}"));
            }
        }
        process.wait();
    }
    Ok(())
}

/// Intended for shutting down previous daemon and starting new one. Currently for simplicity sake
/// it operates using a detached process. This is not great but it's not as hard to configure.
pub fn restart_daemon() -> Result<()> {
    // The program use executable passed into the process. It's not the best option but it will do
    // the job in most cases.
    let daemon_name =
        to_daemon_path(env::current_exe().expect("Can't operate without an executable"));
    kill_previous_daemons(&daemon_name).inspect_err(|e| error!("Failed killing daemons {e}"))?;

    let mut v = Command::new(daemon_name).spawn().inspect_err(|e| error!("Failed creating a daemon {e}"))?;
    v.wait()?;
    Ok(())
}
