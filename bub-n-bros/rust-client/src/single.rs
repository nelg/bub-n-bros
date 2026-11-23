//! One client per screen when the web launcher starts it (--single-instance).
//! Uses the same lock file as display/singleinstance.py, so the Python and
//! the native clients also exclude each other.

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

fn clean(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

pub fn lock_path() -> PathBuf {
    let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let display = var("WAYLAND_DISPLAY").or_else(|| var("DISPLAY"))
        .unwrap_or_else(|| "console".to_string());
    let (folder, user) = match var("XDG_RUNTIME_DIR") {
        Some(d) => (PathBuf::from(d), String::new()),
        None => {
            let name = var("USER").or_else(|| var("USERNAME")).unwrap_or_default();
            (std::env::temp_dir(), format!("-{}", clean(&name)))
        }
    };
    folder.join(format!("bubnbros-client{}-{}.lock", user, clean(&display)))
}

/// Returns the locked file (keep it for the life of the program), or None
/// if another client holds the lock.  If the lock cannot be taken for any
/// other reason, playing is not prevented.
pub fn acquire() -> Option<Option<File>> {
    let path = lock_path();
    let file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(_) => return Some(None),
    };
    match file.try_lock() {
        Ok(()) => Some(Some(file)),
        Err(std::fs::TryLockError::WouldBlock) => None,
        Err(_) => Some(None),
    }
}
