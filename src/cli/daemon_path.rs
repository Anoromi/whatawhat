use std::path::PathBuf;

pub fn to_daemon_path(mut path: PathBuf) -> PathBuf {
    path.set_file_name("whatawhat-daemon");
    #[cfg(windows)]
    {
        path.set_extension("exe");
    }
    path
}
