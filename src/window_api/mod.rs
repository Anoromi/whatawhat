//! Contains logic for extracting records from different environments.
//! [GenericWindowManager] is the main artifact of this module that abstracts
//! the operations.

#[cfg(feature = "win")]
pub mod win;
#[cfg(feature = "x11")]
pub mod x11;

#[cfg(feature = "win")]
extern crate windows;

#[cfg(feature = "x11")]
extern crate xcb;

use std::sync::Arc;

use anyhow::Result;

#[derive(Debug)]
pub struct ActiveWindowData {
    /// Name of the window. For example 'bash in hello' or 'Document 1' or 'Vibing in YouTube -
    /// Chrome'
    pub window_title: Arc<str>,
    /// Full path to an executable. For example /home/etc/nvim
    pub process_name: Arc<str>,
}

/// Intended to serve as a contract windows and linux systems must implement.
pub trait WindowManager {
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData>;

    /// Retrieve amount of time user has been inactive in milliseconds
    fn get_idle_time(&mut self) -> Result<u32>;
}

/// Serves as a cross-compatible WindowManager implementation.
pub struct GenericWindowManager {
    inner: Box<dyn WindowManager>,
}

impl GenericWindowManager {
    pub fn new() -> Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "win")] {
                use win::WindowsWindowManager;
                Ok(Self {
                    inner: Box::new(WindowsWindowManager::new()),
                })
            }
            else if #[cfg(feature = "x11")] {
                use x11::LinuxWindowManager;
                Ok(Self {
                    inner: Box::new(LinuxWindowManager::new()?),
                })
            }
            else {
                // This runtime error is needed to allow the project to be compiled for during testing.
                unimplemented!("No window manager was specified")
            }
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
