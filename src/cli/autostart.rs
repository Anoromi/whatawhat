use std::{env, fs, io, path::PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::cli::daemon_path::to_daemon_path;

pub fn configure_autostart() -> Result<()> {
    let daemon_path = to_daemon_path(env::current_exe().context("Failed to get current executable")?);

    #[cfg(target_os = "linux")]
    {
        configure_autostart_linux(&daemon_path)?;
        println!("Autostart configured: created ~/.config/autostart/whatawhat-daemon.desktop -> {:?}", daemon_path);
        return Ok(());
    }

    #[cfg(windows)]
    {
        configure_autostart_windows(&daemon_path)?;
        println!("Autostart configured: added HKCU Run entry for {:?}", daemon_path);
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(anyhow!("Autostart is not supported on this OS"))
}

#[cfg(target_os = "linux")]
fn configure_autostart_linux(daemon_path: &PathBuf) -> Result<()> {
    let home = env::var("HOME").context("HOME not set")?;
    let autostart_dir = PathBuf::from(home).join(".config").join("autostart");
    fs::create_dir_all(&autostart_dir).with_context(|| format!("Failed to create {:?}", autostart_dir))?;

    let desktop_file_path = autostart_dir.join("whatawhat-daemon.desktop");
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName=Whatawhat Daemon\nExec={}\nX-GNOME-Autostart-enabled=true\nHidden=false\n",
        daemon_path.display()
    );
    fs::write(&desktop_file_path, contents)
        .with_context(|| format!("Failed writing {:?}", desktop_file_path))?;
    Ok(())
}

#[cfg(windows)]
fn configure_autostart_windows(daemon_path: &PathBuf) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _disp) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    // Quote the path in case of spaces
    let value: String = format!("\"{}\"", daemon_path.display());
    run.set_value("Whatawhat", &value)?;
    Ok(())
}

