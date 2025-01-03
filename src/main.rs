// #[cfg(windows)]
pub mod windows;

fn main() {
    // if cfg!(windows) {
        windows::hello_there().unwrap();
    // }
}
