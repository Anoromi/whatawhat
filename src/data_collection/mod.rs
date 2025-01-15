#[cfg(windows)]
pub mod windows;

use std::rc::Rc;

use anyhow::Result;



pub struct ActiveWindowData {
    pub title: Rc<str>,
    pub process_name: Rc<str>
}

pub fn get_active() -> Result<ActiveWindowData> {
    #[cfg(windows)]
    {
        windows::get_active()
    }
}
