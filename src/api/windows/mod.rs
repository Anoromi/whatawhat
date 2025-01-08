pub mod active_process;

use core::panic;
use std::mem::transmute;

use windows::Win32::{
    Foundation::*,
    System::{
        ProcessStatus::{GetModuleBaseNameW, GetModuleFileNameExW},
        Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
            PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        },
    },
    UI::{Input::KeyboardAndMouse::GetActiveWindow, WindowsAndMessaging::*},
};

// #[cfg(windows)]
pub fn hello_there() -> windows::core::Result<()> {
    let mut data = (2, 4);
    let data_ptr: *mut _ = &mut data;
    let k = unsafe { transmute::<&mut _, isize>(&mut data) };
    println!("Start {}", k);
    unsafe { EnumWindows(Some(enum_window_processes), LPARAM(k))? }
    println!("End");
    Ok(())
}

extern "system" fn enum_window(window: HWND, params: LPARAM) -> BOOL {
    let mut text: [u16; 1024] = [0; 1024];
    let len = unsafe { GetWindowModuleFileNameW(window, &mut text) };

    let text = String::from_utf16_lossy(&text[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowInfo(window, &mut info) }.unwrap();

    if !text.is_empty()
    //&& info.dwStyle.contains(WS_VISIBLE)
    {
        // dbg!(&info);
        println!("{} ({}, {})", text, info.rcWindow.left, info.rcWindow.top);
        println!("{}", params.0);
    }

    true.into()
}

extern "system" fn enum_window_processes(window: HWND, params: LPARAM) -> BOOL {
    let mut id = 0u32;
    if window.is_invalid() {
        panic!()
    }
    unsafe { GetWindowThreadProcessId(window, Some(&mut id)) };
    if id == 0 {
        return true.into();
    }
    let process_handle = match unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            BOOL::from(false),
            id,
        )
    } {
        Ok(value) => value,
        Err(_) => return true.into(),
    };

    let mut text: [u16; 1024] = [0; 1024];

    let len = unsafe { GetModuleBaseNameW(process_handle, None, &mut text) };

    // let mut length = text.len() as u32;
    // let len2 = unsafe { QueryFullProcessImageNameW(process_handle, PROCESS_NAME_WIN32, windows::core::PWSTR(text.as_mut_ptr()), &mut length) };

    unsafe { CloseHandle(process_handle).unwrap() };

    let text = String::from_utf16_lossy(&text[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowInfo(window, &mut info) }.unwrap();

    if !text.is_empty() && info.dwStyle.contains(WS_VISIBLE)  {
        // dbg!(&info);
        println!("{} ({}, {})", text, info.rcWindow.left, info.rcWindow.top);
        println!("{}", params.0);
    }

    true.into()
}


pub fn get_active() -> BOOL {
    let window = unsafe { GetForegroundWindow() };

    let mut id = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut id)) };
    if id == 0 {
        return true.into();
    }
    let process_handle = match unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            BOOL::from(false),
            id,
        )
    } {
        Ok(value) => value,
        Err(_) => return true.into(),
    };

    let mut text: [u16; 1024] = [0; 1024];

    let len = unsafe { GetModuleBaseNameW(process_handle, None, &mut text) };

    // let mut length = text.len() as u32;
    // let len2 = unsafe { QueryFullProcessImageNameW(process_handle, PROCESS_NAME_WIN32, windows::core::PWSTR(text.as_mut_ptr()), &mut length) };

    unsafe { CloseHandle(process_handle).unwrap() };

    let text = String::from_utf16_lossy(&text[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowInfo(window, &mut info) }.unwrap();

    if !text.is_empty() && info.dwStyle.contains(WS_VISIBLE) && info.dwStyle.contains(WS_VISIBLE) {
        // dbg!(&info);
        println!("{} ({}, {})", text, info.rcWindow.left, info.rcWindow.top);
    }

    true.into()
}
