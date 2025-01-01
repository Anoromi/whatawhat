use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

fn main() -> windows::core::Result<()> {
    unsafe { EnumWindows(Some(enum_window), LPARAM(0)) }
}

extern "system" fn enum_window(window: HWND, _: LPARAM) -> BOOL {
    let mut text: [u16; 1024] = [0; 1024];
    let len = unsafe {
        GetWindowTextW(window, &mut text)
    };

    let text = String::from_utf16_lossy(&text[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowInfo(window, &mut info) }.unwrap();

    if !text.is_empty() && info.dwStyle.contains(WS_VISIBLE) {
        dbg!(&info);
        println!("{} ({}, {})", text, info.rcWindow.left, info.rcWindow.top);
    }

    true.into()
}
