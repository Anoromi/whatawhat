#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(windows)]
pub mod windows;

use std::rc::Rc;

use anyhow::Result;

#[derive(Debug)]
pub struct ActiveWindowData {
    pub title: Rc<str>,
    pub process_name: Rc<str>,
}

pub fn get_active() -> Result<ActiveWindowData> {
    #[cfg(windows)]
    {
        windows::get_active()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_active()
    }
}
