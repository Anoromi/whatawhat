use core::panic;
use std::{thread::sleep, time::Duration};

use windows::Win32::{
    Foundation::{CloseHandle, BOOL, HWND}, System::{ProcessStatus::GetModuleBaseNameW, Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ}}, UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowInfo, GetWindowModuleFileNameA, GetWindowModuleFileNameW, GetWindowTextW, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, WINDOWINFO, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_VISIBLE
        },
    }
};

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

    println!("Hello there");
    // let hook = unsafe {
    //     SetWinEventHook(
    //         EVENT_SYSTEM_FOREGROUND,
    //         EVENT_SYSTEM_MINIMIZEEND,
    //         None,
    //         Some(process_active),
    //         id,
    //         0,
    //         WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    //     )
    // };
    //
    // dbg!(hook);

    // sleep(Duration::from_secs(60 * 60 * 24 * 356 * 1000));


    let mut text: [u16; 1024] = [0; 1024];

    let len = unsafe { GetModuleBaseNameW(process_handle, None, &mut text) };

    let process_name = String::from_utf16_lossy(&text[..len as usize]);

    let mut length = text.len() as u32;
    unsafe { QueryFullProcessImageNameW(process_handle, PROCESS_NAME_WIN32, windows::core::PWSTR(text.as_mut_ptr()), &mut length) }.unwrap();

    let full_path = String::from_utf16_lossy(&text[..length as usize]);


    let len = unsafe {GetWindowTextW(window, &mut text) };
    let window_title = String::from_utf16_lossy(&text[..len as usize]);

    unsafe { CloseHandle(process_handle).unwrap() };



    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowInfo(window, &mut info) }.unwrap();

    if !process_name.is_empty() && info.dwStyle.contains(WS_VISIBLE) && info.dwStyle.contains(WS_VISIBLE) {
        // dbg!(&info);
        println!("{} {} {} ({}, {})", process_name, full_path, window_title, info.rcWindow.left, info.rcWindow.top);
    }

    true.into()
}

extern "system" fn process_active(
    hwineventhook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    idchild: i32,
    ideventthread: u32,
    dwmseventtime: u32,
) {
    panic!();
    println!("Hello there")
}
