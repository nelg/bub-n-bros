//! "Play alone": start a game server on this computer when none is found.
//!
//! The server is the Python game server packaged as one executable with
//! PyInstaller (packing/pyinstaller/).  Release builds of the client carry
//! it inside (see build.rs); on first use it is unpacked into the cache
//! folder.  Otherwise a `bubnbros-server` program next to the client or on
//! the PATH is used (the Arch Linux package installs the Python server
//! under that name).

use md5::{Digest, Md5};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const EXE: &str = if cfg!(windows) { ".exe" } else { "" };

/// Runs bubbob/bb.py (argv[1]) and exits when BUBNBROS_PARENT_PID is gone.
const WATCHDOG: &str = "\
import os, sys, threading, time, runpy
pid = int(os.environ.get('BUBNBROS_PARENT_PID', '0'))
def alive():
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except OSError:
        return True
def watch():
    while alive():
        time.sleep(2.0)
    os._exit(0)
if pid and sys.platform != 'win32':
    threading.Thread(target=watch, daemon=True).start()
sys.argv = sys.argv[1:]
runpy.run_path(sys.argv[0], run_name='__main__')
";

#[cfg(embedded_server)]
static EMBEDDED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/embedded-server.bin"));
#[cfg(not(embedded_server))]
static EMBEDDED: &[u8] = &[];

/// What to play, chosen on the play alone screen.
#[derive(Clone, Debug)]
pub struct Settings {
    /// level file name, as in bubbob/levels/ (e.g. "Levels.bin")
    pub level: String,
    /// no limit on lives; otherwise 5 lives, one more every 50000 points
    pub unlimited_lives: bool,
}

/// The level sets offered on the play alone screen.
pub const LEVELS: [(&str, &str); 7] = [
    ("Levels.bin", "Levels - the original game"),
    ("LostLevels.bin", "Lost Levels"),
    ("HouseOfFun.bin", "House of Fun"),
    ("Arena.bin", "Arena"),
    ("CompactLevels.py", "Compact Levels"),
    ("RandomLevels.py", "Random Levels - new levels every time"),
    ("scratch.py", "Scratch - experimental levels"),
];

/// "Arena", "arena.bin"... -> "Arena.bin"
pub fn level_file(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    LEVELS.iter().map(|(f, _)| *f).find(|f| {
        let fl = f.to_ascii_lowercase();
        fl == lower || fl.rsplit_once('.').map_or(false, |(stem, _)| stem == lower)
    }).map(|f| f.to_string())
}

/// The running server, stopped when the client ends.
static SERVER: Mutex<Option<Child>> = Mutex::new(None);

pub fn has_embedded_server() -> bool {
    !EMBEDDED.is_empty()
}

/// Unpack the embedded server into `dir` (once per version of it).
fn unpack(dir: &Path) -> std::io::Result<PathBuf> {
    let hash = Md5::digest(EMBEDDED);
    let tag: String = hash[..6].iter().map(|b| format!("{:02x}", b)).collect();
    let path = dir.join(format!("bubnbros-server-{}{}", tag, EXE));
    if path.metadata().map(|m| m.len() == EMBEDDED.len() as u64).unwrap_or(false) {
        return Ok(path);
    }
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("bubnbros-server-{}.tmp{}", std::process::id(), EXE));
    std::fs::write(&tmp, EMBEDDED)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(&tmp, &path)?;
    // remove the servers unpacked by older versions of the client
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("bubnbros-server-") && e.path() != path {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    Ok(path)
}

/// Which server program to run, the arguments to put first, and the
/// folder to run it in.
fn find_server_program(cache: &Path) -> Option<(PathBuf, Vec<String>, Option<PathBuf>, bool)> {
    if has_embedded_server() {
        match unpack(cache) {
            Ok(p) => return Some((p, vec![], None, false)),
            Err(e) => eprintln!("cannot unpack the embedded server: {}", e),
        }
    }
    let mut candidates = Vec::new();
    if let Ok(me) = std::env::current_exe() {
        if let Some(dir) = me.parent() {
            candidates.push(dir.join(format!("bubnbros-server{}", EXE)));
        }
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            candidates.push(dir.join(format!("bubnbros-server{}", EXE)));
        }
    }
    for c in candidates {
        if !c.is_file() {
            continue;
        }
        // the Python server itself (bubbob/bb.py, e.g. /usr/bin/bubnbros-server
        // in the Arch package): run it from its folder, with a level file
        let real = std::fs::canonicalize(&c).unwrap_or(c.clone());
        if real.file_name().map_or(false, |n| n == "bb.py") {
            // run it through a few lines of Python that stop it if this
            // client disappears (like the packaged server does)
            let dir = real.parent().map(|d| d.to_path_buf());
            let python = if cfg!(windows) { "python" } else { "python3" };
            return Some((PathBuf::from(python),
                         vec!["-c".into(), WATCHDOG.into(),
                              real.to_string_lossy().into_owned()],
                         dir, true));
        }
        return Some((c, vec![], None, false));
    }
    None
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(0)
}

/// Start a server and wait until it accepts connections.
pub fn start(cache: &Path, settings: &Settings) -> Result<SocketAddr, String> {
    let (program, prefix_args, cwd, python_script) = find_server_program(cache).ok_or_else(|| {
        "no server found, and this client has no built-in server \
         (see rust-client/README.md, \"Play alone\")".to_string()
    })?;
    let port = free_port();
    let http = free_port();
    let log_path = cache.join("play-alone-server.log");
    let _ = std::fs::create_dir_all(cache);
    let log = std::fs::File::create(&log_path).ok();
    let mut cmd = Command::new(&program);
    cmd.args(&prefix_args)
        .arg("--port").arg(format!("LISTEN={}", port))
        .arg("--port").arg(format!("HTTP={}", http))
        .args(if settings.unlimited_lives {
            vec![]
        } else {
            vec!["--lives", "5", "--extralife", "50000"]
        })
        // the packaged server finds a bare level file name in its own
        // levels folder; bb.py runs from the bubbob folder
        .arg(if python_script {
            format!("levels/{}", settings.level)
        } else {
            settings.level.clone()
        })
        // the packaged server stops by itself if this client disappears
        .env("BUBNBROS_PARENT_PID", std::process::id().to_string())
        .stdin(Stdio::null());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    match log {
        Some(f) => {
            let f2 = f.try_clone().map(Stdio::from).unwrap_or(Stdio::null());
            cmd.stdout(Stdio::from(f)).stderr(f2);
        }
        None => {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    eprintln!("play alone: starting {}", program.display());
    let child = cmd.spawn().map_err(|e| format!("cannot start {}: {}", program.display(), e))?;
    *SERVER.lock().unwrap() = Some(child);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
            return Ok(addr);
        }
        if let Some(c) = SERVER.lock().unwrap().as_mut() {
            if let Ok(Some(status)) = c.try_wait() {
                return Err(format!("the server stopped ({}); see {}", status, log_path.display()));
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    stop();
    Err(format!("the server did not start; see {}", log_path.display()))
}

/// Stop the server started by start(), if any.
pub fn stop() {
    let mut child = match SERVER.lock().unwrap().take() {
        Some(c) => c,
        None => return,
    };
    let pid = child.id().to_string();
    // A PyInstaller program runs the game in a child process of its own:
    // ask politely (it then stops its child), or stop the whole tree.
    if cfg!(windows) {
        let _ = Command::new("taskkill").args(["/PID", &pid, "/T", "/F"]).output();
    } else {
        let _ = Command::new("kill").args(["-TERM", &pid]).output();
        for _ in 0..30 {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}
