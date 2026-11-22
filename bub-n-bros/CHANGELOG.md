# Changelog

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
