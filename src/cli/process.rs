use std::{env, path::PathBuf};

use sysinfo::{get_current_pid, System};

pub fn kill_previous_servers(name: &PathBuf) {
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
            println!("It happened");
            // gracefuly_terminate(pid.as_u32());
            println!("Waiting to die");

            // TODO Gracefuly kill
            // process.kill_with(Signal::Term);
            // process.kill();
            process.wait();
        }
    }
}

pub fn restart_server() {
    let process_name = env::current_exe().unwrap();
    // kill_previous_servers(&process_name);
    let mut command = std::process::Command::new(process_name);
    command.args(["serve", "hello"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        
        command.creation_flags(CREATE_NEW_CONSOLE.0);

    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        
    }

    println!("Spawning");
    #[allow(clippy::zombie_processes)]
    let ch = command.spawn().unwrap();

    
}
