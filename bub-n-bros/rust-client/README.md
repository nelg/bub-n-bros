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
| Esc | leave full screen; in a window, quit |
| F7 / F8 | volume down / up (20% steps, up to 200%) |
| F9 | mute / unmute |

These keys are shown at the top of the window until somebody joins the
game.  The game image is always scaled to fill the window, keeping its
proportions, so you can maximise or resize the window freely.

Options (`bubnbros-client --help`):

| Option | Meaning |
|---|---|
| `-f`, `--fullscreen` | start in full screen; the game is scaled to fit, keeping its proportions |
| `-z N`, `--zoom=N` | initial window size in percent of the game size (default 100) |
| `-t`, `--tcp` | receive the game frames over TCP (for slow, firewalled or proxied links) |
| `-u`, `--udp` | always use UDP (default: UDP, switching to TCP if too many packets get lost) |
| `--no-sound`, `--no-music` | no sound at all / no background music |
| `--volume=N` | sound volume in percent (default 100, up to 200) |
| `--music-volume=N` | music volume in percent of the sound volume (default 100) |
| `--name=NAME`, `--player=N=NAME` | player names shown on the score board |
| `--single-instance` | do not start if a client was started on this screen less than 30 seconds ago; an older one is stopped and replaced (the web launcher uses this) |
| `--cache=DIR`, `--no-cache` | where downloaded game data is kept (default: the user cache directory, e.g. `~/.cache/bubnbros`) |

**Sound on Linux:** the client plays through the ALSA library.  When a
PulseAudio server is running (or PipeWire's PulseAudio replacement), it
uses ALSA's `pulse` device, so it shows up as a normal application in
the sound settings; otherwise it uses ALSA's `default` device.  Set
`QUAD_SND_ALSA_DEVICE` to force another ALSA device.

Set `BUBNBROS_VERBOSE=1` to print frame and download statistics every
10 seconds.

The game server itself is still the Python program (`bubbob/bb.py`, or
`BubBob.py` for the web launcher).

## Building

The client is an ordinary Cargo project with no build script of its own:
on every platform it is

```bash
cd bub-n-bros/rust-client
cargo build --release
```

and the result is a single file, `target/release/bubnbros-client`
(`bubnbros-client.exe` on Windows), that can be copied anywhere.  What
differs between platforms is how to install Rust and the system
libraries.  See [docs/building-native-client.md](../docs/building-native-client.md)
for step-by-step instructions for Linux, Windows and macOS, cross-compiling,
and the GitHub workflow that builds all three.

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
