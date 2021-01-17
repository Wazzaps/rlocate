use std::path::{Path, PathBuf};

pub fn get() -> PathBuf {
    let mut dir = PathBuf::from("/run");

    if let Ok(sock_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let sock_dir = Path::new(&sock_dir);
        if sock_dir.exists() {
            dir = sock_dir.into();
        }
    }

    dir.join("rlocate.sock")
}