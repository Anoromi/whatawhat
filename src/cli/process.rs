use std::{env, path::PathBuf};

use sysinfo::{get_current_pid, System};
use windows::Win32::System::Threading::CREATE_BREAKAWAY_FROM_JOB;

pub fn kill_previous_servers(name: &PathBuf) {
    let system = System::new_all();
    for (pid, process) in system.processes().iter() {
        println!("Process {:?}", process.exe());
        let this_pid = get_current_pid().unwrap();
        println!("This id {:?}", this_pid);
        println!("This id {:?}", process);
        if *pid == this_pid {
            continue;
        }
        if matches!(process.parent(), Some(p) if p == this_pid) {
            continue;
        }

        if process
            .exe()
            .filter(|v| v.exists())
            .filter(|v| name == *v)
            .is_some()
        {
            println!("It happened");
            process.kill();
            process.wait();
        }
    }
}

pub fn restart_server() {
    let process_name = env::current_exe().unwrap();
    kill_previous_servers(&process_name);
    let mut command = std::process::Command::new(process_name);
    command.args(["run"]);

    #[cfg(windows)]
    {
        
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_BREAKAWAY_FROM_JOB.0);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    // .process_group(0)
    // .stdout(Stdio::null())
    // .spawn()
    // .unwrap()
    // .wait();
}
