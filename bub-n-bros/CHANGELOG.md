# Changelog

## 2.4.0 (2026-11-24)

### Added

- Play alone: the native client, started without arguments when no game
  server runs on the computer (or with `--solo`), starts a server just
  for itself and shows "Play alone mode".  A start screen chooses the
  level set and unlimited lives (otherwise 5 lives, one more every 50000
  points); `--level=NAME` and `--unlimited-lives` skip it.  The server is the Python
  server packaged as one executable with PyInstaller
  (`packing/pyinstaller/`); the GitHub workflow builds it on Linux,
  Windows and macOS, embeds it in the client and also publishes it on
  its own.  A server started this way stops when the client ends, even
  if the client is killed.
- Three forms of the game, as GitHub release downloads and as Linux
  packages that conflict with each other: **native** (the game on the
  system's Python, with all the clients), **server-embedded** (the
  server as one PyInstaller program with its own Python;
  `bubnbros-server --web` is the web launcher) and **play-alone** (the
  native client with that server built in).  Packaging: `debian/` for
  Debian and Ubuntu, `packing/rpm/bubbros.spec` for Fedora and the
  Mandriva family, and the Arch PKGBUILD (now a split package), sharing
  `packing/linux/`; a GitHub workflow builds and tests them.  The menu
  entries are "Bub-n-Bros server" (the web launcher) and "Bub-n-Bros
  play alone" (`bubbros-native --solo`).
- The packaged server also contains the web launcher pages.  Built with
  `build_server.py --native-client-only` (as for the server-embedded and
  play-alone downloads and packages), it leaves out the Python clients
  and the Java applet, and its web launcher only offers the native
  client.  The native form is unchanged.
- Play alone removes the built-in server it unpacked into the cache
  when the client quits.

### Fixed

- The Linux server-embedded and play-alone downloads crashed ("illegal
  instruction") on processors without BMI2 (before Intel Haswell / AMD
  Excavator): the Python bundled into the server is now a generic
  x86-64 build.

### Changed

- The web launcher's settings are saved in `~/.bubnbros` (they were in
  `http2/config.txt`, which is still read if there is no `~/.bubnbros`
  yet), and the colored images are only generated when
  `bubbob/images/buildcolors.py` is there.  So an installed, read-only
  copy of the game works without changes, and the Arch package no
  longer needs its patch; the Debian and RPM packages have none.
- Native client: the one-client-per-screen lock no longer needs Rust
  1.89, so the client builds with the Rust versions of current
  distributions (1.75 or newer).

## 2.3.4 (2026-11-23)

### Changed

- The one-client-per-screen lock expires: a client started from the web
  launcher more than 30 seconds after the running one stops it (if it
  runs as the same user, e.g. a stuck client) and takes its place.  The
  lock file now holds the running client's process id and start time.

## 2.3.3 (2026-11-23)

### Changed

- Clients started from the web launcher ("Join") run only once per
  screen: if a Bub-n-Bros client (Python or native) is already running on
  the same display, the new one logs "already running" and exits.  This
  is the new `--single-instance` option of both clients, which the
  launcher always passes; clients started by hand are not limited.

## 2.3.2 (2026-11-23)

### Changed

- The web launcher uses the native client by default when it is
  installed (a client chosen in the settings is kept).  `Client.py`
  itself is unchanged: without `-d` it still picks an installed Python
  display driver.

## 2.3.1 (2026-11-23)

### Fixed

- pygame client: on systems where PulseAudio is the sound server and
  PipeWire also runs, SDL played through PipeWire (often on HDMI
  speakers) instead of PulseAudio; it now uses PulseAudio when it is
  running (display/sdlaudio.py), and shows up as "The Bub's Brothers"
  in the sound settings.

## 2.3.0 (2026-11-23)

### Fixed

- `make docs` did nothing, because of the new `docs/` directory.
- pygame client: the sounds and music were very quiet (the game's 8-bit
  recordings only use a third to a half of the available range); they
  are now played amplified.
- pygame client: no music when the client did not have its own copy of
  the game files; it now fetches the music from the server quickly and
  starts it once complete.

### Added

- The web launcher offers the native client in its settings ("native");
  "Join" then starts `bubbros-native` (or a build in
  `rust-client/target/release/`) instead of the Python client.
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
