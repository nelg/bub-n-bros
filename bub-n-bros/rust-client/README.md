# bubnbros-client — native client in Rust

A single-file client for The Bub's Brothers, for Linux, Windows and macOS.
It needs no Python and no game files: everything (graphics, sounds,
music, levels) comes from the server, and is cached locally so that the
next game starts at once.  It speaks the same protocol as
`display/Client.py` and the Java applet, so it can join any Bub-n-Bros
server, including the old Python 2 ones.

## Playing

```bash
bubnbros-client                    # find a server on this machine or the LAN
bubnbros-client myhost             # find a server on that machine
bubnbros-client myhost:12345       # connect to that game port
bubnbros-client --fullscreen --name=Glen myhost:12345
```

Then click on one of the dragons at the bottom of the window and press
the four keys you want to use for **right, left, jump and fire**.  Click
on another dragon to add a second player on the same keyboard; click on
your dragon again to leave the game.

| Key | Action |
|---|---|
| F11 or Alt+Enter | switch full screen on and off |
| Esc | leave full screen |
| F7 / F8 | volume down / up (10% steps, up to 300%) |

Options (`bubnbros-client --help`):

| Option | Meaning |
|---|---|
| `-f`, `--fullscreen` | start in full screen; the game is scaled to fit, keeping its proportions |
| `-z N`, `--zoom=N` | window size in percent of the game size (default 100) |
| `-t`, `--tcp` | receive the game frames over TCP (for slow, firewalled or proxied links) |
| `-u`, `--udp` | always use UDP (default: UDP, switching to TCP if too many packets get lost) |
| `--no-sound`, `--no-music` | no sound at all / no background music |
| `--volume=N` | sound volume in percent (default 100, up to 300) |
| `--music-volume=N` | music volume in percent of the sound volume (default 100) |
| `--name=NAME`, `--player=N=NAME` | player names shown on the score board |
| `--cache=DIR`, `--no-cache` | where downloaded game data is kept (default: the user cache directory, e.g. `~/.cache/bubnbros`) |

Set `BUBNBROS_VERBOSE=1` to print frame and download statistics every
10 seconds.

The game server itself is still the Python program (`bubbob/bb.py`, or
`BubBob.py` for the web launcher).

## Building

Install Rust (https://rustup.rs), then:

```bash
cd bub-n-bros/rust-client
cargo build --release
# the binary is target/release/bubnbros-client (.exe on Windows)
```

On Linux the build needs the ALSA, X11 and OpenGL development files
(Debian/Ubuntu: `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`;
Arch: `alsa-lib libx11 libxi mesa`).  Windows and macOS need nothing
extra.

The GitHub workflow `.github/workflows/rust-client.yml` builds the
binaries for Linux (x86_64), Windows (x86_64) and macOS (Intel and Apple
Silicon) on every change, and attaches them to the release when a `v*`
tag is pushed.  The macOS binaries are not signed: the first time, open
them with right-click → Open.

## How it works

| File | Contents |
|---|---|
| `src/main.rs` | command line, finding the server, window, input, scaling the game image to the window or screen |
| `src/game.rs` | the protocol client (port of `display/pclient.py`): definitions, frames (UDP with protocol-3 compression, or inline over TCP), sprites, the player taskbar and key definition |
| `src/proto.rs` | message encoding (`common/msgstruct.py`) |
| `src/files.rs` | downloads checked by MD5, and the local cache (`display/caching.py`) |
| `src/audio.rs` | sound effects and background music |
| `src/discovery.rs` | finding servers with UDP pings on port 8056 (`common/hostchooser.py`) |

It uses [macroquad](https://macroquad.rs) for the window, OpenGL drawing,
input and sound, `flate2` (pure Rust zlib) and `md-5`.  See
`docs/architecture.md` for the protocol.

Not supported (yet): joining Internet games through the meta-server
(`Client.py -m`), the LAN broadcast of frames to several clients (each
client gets its own UDP stream instead, which works the same), and
left/right stereo panning of sounds.
