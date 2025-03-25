
use anyhow::{Result, anyhow};
use tracing::error;
use windows::{
    Win32::{
        Foundation::{BOOL, CloseHandle, GetLastError, HANDLE, HWND},
        System::{
            Diagnostics::Debug::{
                FORMAT_MESSAGE_FROM_SYSTEM,
                FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageW,
            },
            SystemInformation::GetTickCount64,
            SystemServices::{LANG_ENGLISH, SUBLANG_ENGLISH_US},
            Threading::{
                OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
                QueryFullProcessImageNameW,
            },
        },
        UI::{
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
        },
    },
    core::PWSTR,
};

use super::{ActiveWindowData, WindowManager};

#[tracing::instrument]
pub fn get_active() -> Result<ActiveWindowData> {
    let window = unsafe { GetForegroundWindow() };

    if window.is_invalid() {
        return Err(anyhow!("Failed to get foreground window"));
    }

    let mut id = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut id)) };
    if id == 0 {
        let err = unsafe { GetLastError() };
        let mut message_buffer = [0u16; 2048];
        let size = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                None,
                err.0,
                LANG_ENGLISH | (SUBLANG_ENGLISH_US << 10),
                // 0x1033_0400,
                PWSTR::from_raw(message_buffer.as_mut_ptr()),
                2048,
                None,
            )
        };
        if size == 0 {
            return Err(anyhow!("Failed to get active window"));
        } else {
            let data =
                String::from_utf16(&message_buffer[0..size as usize]).expect("Failed to unwrap");
            return Err(anyhow!("Failed to get active window {data}"));
        }
    }
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            BOOL::from(false),
            id,
        )
    }
    .inspect_err(|e| error!("Failed to open process {e:?}"))?;

    let mut text: [u16; 4096] = [0; 4096];
    let process_name = unsafe { get_window_process_path(process_handle, &mut text) }
        .inspect_err(|e| error!("Failed to get window process path {e:?}"))?;
    let title = unsafe { get_window_title(window, &mut text) };

    unsafe { CloseHandle(process_handle) }
        .inspect_err(|e| error!("Failed to close handle {e:?}"))?;


    Ok(ActiveWindowData {
        process_name: process_name.into(),
        window_title: title.into(),
    })
}

unsafe fn get_window_process_path(window_handle: HANDLE, text: &mut [u16]) -> Result<String> {
    unsafe {
        let mut length = text.len() as u32;
        QueryFullProcessImageNameW(
            window_handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(text.as_mut_ptr()),
            &mut length,
        )?;
        Ok(String::from_utf16_lossy(&text[..length as usize]))
    }
}

unsafe fn get_window_title(window_handle: HWND, text: &mut [u16]) -> String {
    let len = unsafe { GetWindowTextW(window_handle, text) };
    String::from_utf16_lossy(&text[..len as usize])
}

pub fn get_idle_time() -> Result<u32> {
    let mut last: LASTINPUTINFO = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    let is_success = unsafe { GetLastInputInfo(&mut last) };
    if !is_success.as_bool() {
        error!("Failed to retrieve user idle time");
        return Err(anyhow!("Failed to retrieve user idle time"));
    }

    let tick_count = unsafe { GetTickCount64() };
    let duration = tick_count - last.dwTime as u64;
    if duration > u32::MAX as u64 {
        Ok(u32::MAX)
    } else {
        Ok(duration as u32)
    }
}

pub struct WindowsWindowManager {}

impl WindowsWindowManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WindowsWindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager for WindowsWindowManager {
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData> {
        get_active().inspect_err(|e| error!("Failed to get active window {e:?}"))
    }

    fn get_idle_time(&mut self) -> Result<u32> {
        get_idle_time().inspect_err(|e| error!("Failed to get idle time {e:?}"))
    }
}
