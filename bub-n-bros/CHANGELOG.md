# Changelog

## Unreleased

### Fixed

- `make docs` did nothing, because of the new `docs/` directory.

### Added

- Native client: build instructions for Linux, Windows and macOS
  (`docs/building-native-client.md`) and an INSTALL.txt section; the Arch
  Linux package builds it too and installs it as `bubbros-native`.

- Python clients (pygame, GTK, X): F7 / F8 turn the volume down / up in
  20% steps and F9 mutes (pygame sound driver); Esc quits when in a
  window; the window can be resized or maximised and the game is scaled
  to fill it; the client's keys are shown at the top of the screen until
  somebody joins the game.

## 2.2.0 (2026-11-23)

### Added

- `rust-client/`: a native client written in Rust, built as a single
  binary for Linux, Windows and macOS.  It needs neither Python nor the
  game files (everything is downloaded from the server and cached), and
  supports full screen, window resizing, sound and music (started within
  seconds by downloading it ahead of the server's stream), volume keys
  (F7 / F8, F9 to mute), several players per client, UDP or TCP frames
  and LAN server discovery.  On Linux it plays through PulseAudio when it
  is running.  A GitHub workflow builds it for all three systems.

## 2.1.0 (2026-11-23)

### Added

- Full screen in all Python clients (pygame, GTK and X): start with
  `--fullscreen=yes`, or press F11 (or Alt+Enter) at any time; Esc leaves
  full screen.  The game is scaled to fit the screen, keeping its
  proportions (the X driver uses the XRender extension for this).

## 2.0.1 (2026-11-23)

### Added

- Developer documentation in `docs/`: architecture (with diagrams), code
  layout, how the game engine works, and a guide to creating levels with an
  example level set (`docs/examples/TutorialLevels.py`).

## 2.0.0 (2026-11-23)

Port of the whole game from Python 2 to Python 3 (tested with Python 3.14).
Python 2 is no longer supported.  The network protocol is unchanged, so
2.0.0 servers and clients still play with 1.x ones and with the Java applet.

### Changed

- All of the code (server, clients, web launcher, meta-server, tools) now
  runs on Python 3.  The network protocol, image and sound data are handled
  as bytes throughout.
- GTK display driver rewritten for GTK 3 with PyGObject and cairo (PyGTK 2
  does not exist for Python 3).  The `--zoom` option is kept.
- The `xshm` C extension used by the X display driver is ported to the
  Python 3 C API and builds with setuptools (`make`).
- The embedded Python snippets in the web launcher's HTML pages are
  converted to Python 3.
- `audioop`, `cgi`, `md5` and `sys.setcheckinterval` are no longer used,
  since they are gone from current Python versions.
- Arch Linux packaging (`packing/arch`) updated for Python 3 and pygame.

### Fixed

- The web launcher's error pages now show the server traceback, and the
  traceback is also logged.
- `test_rnglevel.py` and `save_rnglevel.py` run again outside a game.

### Removed / known limitations

- The clock bonus is disabled: it needs the `statesaver` extension, which
  copies running generators and cannot be built for Python 3.
- The `linux` sound driver needs `linuxaudiodev`, which does not exist in
  Python 3; use the pygame sound driver.
- The native Windows driver (`display/windows/wingame.c`) is not ported;
  use pygame on Windows.

## 1.6.2 and earlier

Python 2 releases by Armin Rigo and contributors, see
http://bub-n-bros.sourceforge.net and the git history.
