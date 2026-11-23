// Optionally embed the single-file game server (built with PyInstaller,
// see packing/pyinstaller/) in the client, for the "play alone" mode:
//
//     BUBNBROS_SERVER_BINARY=/path/to/bubnbros-server cargo build --release
//
// Without the variable the client is built without an embedded server
// (play alone then looks for a bubnbros-server program instead).

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(embedded_server)");
    println!("cargo:rerun-if-env-changed=BUBNBROS_SERVER_BINARY");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded-server.bin");
    match env::var("BUBNBROS_SERVER_BINARY") {
        Ok(path) if !path.is_empty() => {
            println!("cargo:rerun-if-changed={}", path);
            std::fs::copy(&path, &out)
                .unwrap_or_else(|e| panic!("cannot read BUBNBROS_SERVER_BINARY {}: {}", path, e));
            println!("cargo:rustc-cfg=embedded_server");
        }
        _ => {
            let _ = std::fs::remove_file(&out);
        }
    }
}
