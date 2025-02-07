// use std::u32;
//
// use windows::Win32::System::Console::{
//     FreeConsole, GenerateConsoleCtrlEvent,
// };
//
// pub fn gracefuly_terminate(uid: u32) -> Option<bool> {
//     #[cfg(windows)]
//     {
//         // SetWindowsHookExW(WH_CALLWNDPROC, Some(test_process), None, thread_id).unwrap();
//
//         use windows::Win32::System::Console::AttachConsole;
//         // let consoleWindow = unsafe {GetConsoleWindow() };
//         unsafe { FreeConsole().unwrap() };
//         println!("Freedom");
//         // println!("Returned");
//
//         let result = unsafe { AttachConsole(uid) };
//         if result.is_ok() {
//             
//             unsafe { GenerateConsoleCtrlEvent(0, uid) };
//
//             // unsafe { .unwrap() };
//             // unsafe { SetConsoleCtrlHandler(None, true).unwrap() };
//             //
//             // unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0).unwrap() };
//             //
//             // unsafe { SetConsoleCtrlHandler(None, false).unwrap() };
//         }
//
//         unsafe { AttachConsole(u32::MAX).unwrap() };
//
//         result.unwrap();
//
//         Some(true)
//
//         // let mut kill = process::Command::new("taskkill.exe");
//         // kill.arg("/PID").arg(uid.to_string());
//         // kill.creation_flags(CREATE_NO_WINDOW.0);
//         // match kill.output() {
//         //     Ok(o) => {
//         //         sleep(Duration::from_millis(500));
//         //         let out = o.stdout;
//         //         println!("{:?}", String::from_utf8(out));
//         //         // println!("{}", String::from_utf8(*o.stdout).unwrap());
//         //         Some(o.status.success())
//         //     },
//         //     Err(_) => Some(false),
//         // }
//     }
//     #[cfg(unix)]
//     {}
// }
