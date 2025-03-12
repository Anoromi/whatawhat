#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(windows)]
pub mod windows;

use std::sync::Arc;

use anyhow::Result;

#[derive(Debug)]
pub struct ActiveWindowData {
    pub window_title: Arc<str>,
    pub process_name: Arc<str>,
}

/// Intended to serve as a contract windows and linux systems must implement. 
pub trait WindowManager {
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData>;

    /// Retreive amount of time user has been inactive in milliseconds
    fn get_idle_time(&mut self) -> Result<u32>;
}

// Serves as a cross-compatible implementation.
pub struct GenericWindowManager {
    inner: Box<dyn WindowManager>,
}

impl GenericWindowManager {
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        {
            use windows::WindowsWindowManager;
            Ok(Self {
                inner: Box::new(WindowsWindowManager::new()),
            })
        }
        #[cfg(target_os = "linux")]
        {
            use linux::LinuxWindowManager;
            Ok(Self {
                inner: Box::new(LinuxWindowManager::new()?),
            })
        }
    }
}

impl WindowManager for GenericWindowManager {
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData> {
        self.inner.get_active_window_data()
    }

    fn get_idle_time(&mut self) -> Result<u32> {
        self.inner.get_idle_time()
    }
}
