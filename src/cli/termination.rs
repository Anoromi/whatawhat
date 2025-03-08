
// use windows::Win32::{
//     System::Console::{FreeConsole, GenerateConsoleCtrlEvent, GetConsoleWindow, SetConsoleCtrlHandler},
//     UI::WindowsAndMessaging::GetWindowThreadProcessId,
// };

// pub fn terminate_app(uid: u32) -> Result<()> {
//     #[cfg(windows)]
//     {
//         // SetWindowsHookExW(WH_CALLWNDPROC, Some(test_process), None, thread_id).unwrap();
//
//         println!("ver sus");
//
//         use windows::Win32::System::Console::AttachConsole;
//         // let consoleWindow = unsafe {GetConsoleWindow() };
//         let past_window = unsafe { GetConsoleWindow() };
//         let mut process_id = 0;
//         let v = unsafe { GetWindowThreadProcessId(past_window, Some(&mut process_id)) };
//
//         println!("process_id {}", process_id);
//         unsafe { FreeConsole() }?;
//         println!("Freedom");
//         // println!("Returned");
//
//         let result = unsafe { AttachConsole(uid) };
//         // info!("{result:?}");
//         if result.is_ok() {
//             // info!("Steady");
//             // unsafe { GenerateConsoleCtrlEvent(0, 0) }?;
//             // info!("Steady 2");
//
//             // unsafe { .unwrap() };
//             unsafe { SetConsoleCtrlHandler(None, true) }?;
//             const CTRL_C_EVENT: u32 = 0;
//             unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) }?;
//
//             sleep(Duration::from_millis(2000));
//
//             unsafe { SetConsoleCtrlHandler(None, false) }?;
//         }
//         // info!("we returned 1");
//         result?;
//
//         unsafe { AttachConsole(process_id) }?;
//
//         info!("we returned 2");
//
//         Ok(())
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
