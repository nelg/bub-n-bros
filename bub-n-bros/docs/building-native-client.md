# Building the native client (Linux, Windows, macOS)

The native client in `rust-client/` is written in Rust.  It compiles to
one self-contained executable per platform; the people who only want to
*play* can use a ready-made binary (see [Ready-made binaries](#ready-made-binaries))
and do not need any of this.

Whatever the platform, the build itself is always the same two commands:

```bash
cd bub-n-bros/rust-client
cargo build --release
```

The first build downloads the Rust libraries it needs (macroquad, flate2,
md-5 and their dependencies) from crates.io, so it needs Internet access
once; the audio library is included in the repository
(`rust-client/vendor/quad-snd`, see its README for why).  The result is
`target/release/bubnbros-client`, or `bubnbros-client.exe` on Windows.
`cargo test --release` runs the unit tests.

Use a current stable Rust (the client is developed with Rust 1.99).

## Linux

1. Install Rust, either from your distribution or with
   [rustup](https://rustup.rs):

   | Distribution | Command |
   |---|---|
   | Arch Linux | `sudo pacman -S rust` |
   | Debian / Ubuntu | `sudo apt install cargo` (or rustup for a newer version) |
   | Fedora | `sudo dnf install cargo` |
   | any | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |

2. Install the C libraries and headers used for the window, OpenGL and
   sound:

   | Distribution | Command |
   |---|---|
   | Arch Linux | `sudo pacman -S alsa-lib libx11 libxi mesa` |
   | Debian / Ubuntu | `sudo apt install libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev` |
   | Fedora | `sudo dnf install alsa-lib-devel libX11-devel libXi-devel mesa-libGL-devel` |

   For sound through PulseAudio (recommended on desktops running it), also
   install the ALSA PulseAudio plugin (`pulseaudio-alsa` on Arch,
   `libasound2-plugins` on Debian/Ubuntu, `alsa-plugins-pulseaudio` on
   Fedora).  PipeWire systems already provide it.

3. Build:

   ```bash
   cd bub-n-bros/rust-client
   cargo build --release
   ./target/release/bubnbros-client --help
   ```

The binary uses X11 (it also runs under Wayland through XWayland) and
loads `libGL`, `libX11`, `libXi` and `libasound` when it starts, so a
binary built on one Linux distribution runs on others of the same age or
newer.

**Arch Linux package:** `packing/arch/PKGBUILD.txt` builds the whole game,
including this client, which it installs as `/usr/bin/bubbros-native`
(with a desktop menu entry) next to the Python client
`/usr/bin/bubnbros-client`.  Copy it to a directory as `PKGBUILD`,
together with the other files in `packing/arch/`, and run `makepkg -si`.

## Windows

1. Install the **Visual Studio Build Tools** (free): from
   https://visualstudio.microsoft.com/downloads/ choose
   "Build Tools for Visual Studio", and in the installer tick
   **Desktop development with C++**.  Rust needs its linker.
2. Install Rust with **rustup-init.exe** from https://rustup.rs and accept
   the defaults (the `x86_64-pc-windows-msvc` toolchain).
3. Open a new *Command Prompt* or *PowerShell* (so that `cargo` is on the
   PATH), then:

   ```bat
   cd bub-n-bros\rust-client
   cargo build --release
   target\release\bubnbros-client.exe --help
   ```

No other library is needed: the client uses Windows' own OpenGL and
WASAPI sound.  The `.exe` can be copied to any 64-bit Windows 10 or 11
machine.  Windows may show a SmartScreen warning the first time an
unsigned program is started ("More info" → "Run anyway").

### Cross-compiling for Windows from Linux

With rustup and the MinGW-w64 compiler installed (`mingw-w64-gcc` on Arch,
`gcc-mingw-w64-x86-64` on Debian/Ubuntu):

```bash
rustup target add x86_64-pc-windows-gnu
cd bub-n-bros/rust-client
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/bubnbros-client.exe
```

(Rust installed from a distribution package usually lacks the Windows
target; use rustup for this.)

## macOS

1. Install Apple's command line tools (compiler and linker):

   ```bash
   xcode-select --install
   ```

2. Install Rust with rustup:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build:

   ```bash
   cd bub-n-bros/rust-client
   cargo build --release
   ./target/release/bubnbros-client --help
   ```

This builds for the Mac you are on (Apple Silicon or Intel).  To make one
binary that runs on both:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create -output bubnbros-client \
    target/aarch64-apple-darwin/release/bubnbros-client \
    target/x86_64-apple-darwin/release/bubnbros-client
```

The client uses macOS's OpenGL and CoreAudio, so nothing else is needed.
A binary that was downloaded rather than built on the Mac is blocked by
Gatekeeper because it is not signed: right-click it and choose **Open**
the first time, or run `xattr -d com.apple.quarantine bubnbros-client`.
Building macOS binaries on Linux is not practical (it needs Apple's SDK).

## The built-in server (play alone)

The release binaries carry the game server inside them, for the "play
alone" mode (see `rust-client/README.md`).  The server is the Python game
server packaged as a single executable with
[PyInstaller](https://pyinstaller.org), which bundles a Python interpreter
and all the game data.  PyInstaller only builds for the system it runs
on, so the server is built on each platform before the client:

```bash
cd bub-n-bros
python3 -m pip install pyinstaller
python3 packing/pyinstaller/build_server.py
python3 packing/pyinstaller/smoke_test.py packing/pyinstaller/dist/bubnbros-server
cd rust-client
BUBNBROS_SERVER_BINARY=$PWD/../packing/pyinstaller/dist/bubnbros-server \
    cargo build --release
```

On Windows, use `bubnbros-server.exe` and set the variable with
`set BUBNBROS_SERVER_BINARY=...` (cmd) or `$env:BUBNBROS_SERVER_BINARY="..."`
(PowerShell).  Without the variable, the client is built without a server
(about 1.5 MB instead of 25 MB) and play alone uses a `bubnbros-server`
program next to it or on the PATH.

The packaged server can also be used on its own: `bubnbros-server` plays
`Levels.bin`, `bubnbros-server CompactLevels.py` another level set, and
`bubnbros-server --web` starts the web launcher pages.

## Ready-made binaries

The GitHub workflow `.github/workflows/rust-client.yml` builds the client
on GitHub's Linux, Windows and macOS machines whenever `rust-client/`
changes:

| Target | Runs on |
|---|---|
| `x86_64-unknown-linux-gnu` | 64-bit Linux |
| `x86_64-pc-windows-msvc` | 64-bit Windows 10 / 11 |
| `aarch64-apple-darwin` | Macs with Apple Silicon |
| `x86_64-apple-darwin` | Intel Macs |

Each run also builds the stand-alone server (`bubnbros-server-<target>`)
and puts it inside the client.  Each run keeps the binaries as downloadable *artifacts* (on the run's page
under the repository's **Actions** tab), and when a tag starting with `v`
is pushed (for example `v2.2.0`), they are also attached to the GitHub
release for that tag as `.tar.gz` / `.zip` files.

## Troubleshooting

* **"Package alsa was not found" / "cannot find -lX11"** on Linux: the
  development packages from step 2 are missing.
* **"link.exe not found"** on Windows: install the Visual Studio Build
  Tools with "Desktop development with C++", then open a new terminal.
* **No sound on Linux, or on the wrong output:** see the sound notes in
  [`rust-client/README.md`](../rust-client/README.md); the ALSA device can
  be forced with `QUAD_SND_ALSA_DEVICE=pulse` (or `default`, `pipewire`...).
* **Black window or OpenGL errors:** the client needs OpenGL 2.1 or
  OpenGL ES 2.0; update the graphics drivers, or in a virtual machine
  enable 3D acceleration.
