//! One client per screen when the web launcher starts it (--single-instance).
//!
//! Same lock file and format as display/singleinstance.py ("pid start-time"),
//! so the Python and the native clients also exclude each other.  If the
//! client holding the lock started less than TAKEOVER_DELAY seconds ago,
//! the new one exits ("already running"); otherwise the old one is assumed
//! to be stuck or forgotten, and is stopped if it runs as the same user.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TAKEOVER_DELAY: f64 = 30.0;

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

fn now() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn read_owner(path: &PathBuf) -> Option<(u32, f64)> {
    let mut text = String::new();
    File::open(path).ok()?.read_to_string(&mut text).ok()?;
    let mut words = text.split_whitespace();
    let pid = words.next()?.parse().ok()?;
    let started = words.next()?.parse().ok()?;
    Some((pid, started))
}

#[cfg(target_os = "linux")]
fn same_user(pid: u32) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(format!("/proc/{}", pid)), std::fs::metadata("/proc/self")) {
        (Ok(theirs), Ok(mine)) => theirs.uid() == mine.uid(),
        _ => false,
    }
}

#[cfg(not(target_os = "linux"))]
fn same_user(_pid: u32) -> bool {
    true // the system refuses to stop other users' programs anyway
}

fn stop(pid: u32) {
    if cfg!(windows) {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"]).output();
    } else {
        let _ = std::process::Command::new("kill").args(["-TERM", &pid.to_string()]).output();
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(100));
            let alive = std::process::Command::new("kill").args(["-0", &pid.to_string()])
                .output().map(|o| o.status.success()).unwrap_or(false);
            if !alive {
                return;
            }
        }
        let _ = std::process::Command::new("kill").args(["-KILL", &pid.to_string()]).output();
    }
}

fn write_owner(file: &mut File) {
    let _ = file.set_len(0);
    let _ = file.seek(SeekFrom::Start(0));
    let _ = writeln!(file, "{} {:.6}", std::process::id(), now());
    let _ = file.flush();
}

/// Returns the locked file (keep it for the life of the program), or None
/// if another client is running.  If the lock cannot be taken for any
/// other reason, playing is not prevented.
pub fn acquire() -> Option<Option<File>> {
    let path = lock_path();
    let mut file = match OpenOptions::new().create(true).read(true).write(true)
        .truncate(false).open(&path) {
        Ok(f) => f,
        Err(_) => return Some(None),
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => {
            let (pid, started) = read_owner(&path)?;
            if now() - started < TAKEOVER_DELAY || pid == std::process::id() || !same_user(pid) {
                return None;
            }
            eprintln!("Stopping the Bub-n-Bros client already running on this screen \
                       (pid {}, started {:.0} seconds ago).", pid, now() - started);
            stop(pid);
            let mut locked = false;
            for _ in 0..30 {
                if file.try_lock().is_ok() {
                    locked = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if !locked {
                return None;
            }
        }
        Err(_) => return Some(None),
    }
    write_owner(&mut file);
    Some(Some(file))
}
