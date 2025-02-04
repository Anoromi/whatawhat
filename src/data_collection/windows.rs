use std::{
    fmt::{Debug, Display},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use windows::Win32::{
    Foundation::{CloseHandle, BOOL, HANDLE, HWND},
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION,
        PROCESS_VM_READ,
    },
    UI::{
        Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
        WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    },
};

use super::ActiveWindowData;

#[derive(Debug)]
struct ActiveWindowError {}

impl ActiveWindowError {
    fn new() -> Self {
        Self {}
    }
}

impl Display for ActiveWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl std::error::Error for ActiveWindowError {}

pub fn get_active() -> Result<ActiveWindowData> {
    let window = unsafe { GetForegroundWindow() };

    let mut id = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut id)) };
    if id == 0 {
        return Err(ActiveWindowError::new().into());
    }
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            BOOL::from(false),
            id,
        )
    }?;

    let mut text: [u16; 1024] = [0; 1024];
    let process_name = unsafe { get_window_process_path(process_handle, &mut text)? };
    let title = unsafe { get_window_title(window, &mut text) };

    unsafe { CloseHandle(process_handle).unwrap() };

    Ok(ActiveWindowData {
        process_name: process_name.into(),
        title: title.into(),
    })
}

unsafe fn get_window_process_path(window_handle: HANDLE, text: &mut [u16]) -> Result<String> {
    let mut length = text.len() as u32;
    QueryFullProcessImageNameW(
        window_handle,
        PROCESS_NAME_WIN32,
        windows::core::PWSTR(text.as_mut_ptr()),
        &mut length,
    )?;
    Ok(String::from_utf16_lossy(&text[..length as usize]))
}

unsafe fn get_window_title(window_handle: HWND, text: &mut [u16]) -> String {
    let len = unsafe { GetWindowTextW(window_handle, text) };
    String::from_utf16_lossy(&text[..len as usize])
}

pub fn is_afk() {
    let mut last: LASTINPUTINFO = LASTINPUTINFO::default();
    let is_success = unsafe { GetLastInputInfo(&mut last) };
    loop {
        println!("{} {}", is_success.0 != 0, last.dwTime);
        sleep(Duration::from_secs(5));
    }
}
