# Code layout

The repository has two top-level directories: `bub-n-bros/` is the game,
`bub-n-more/` holds old packaging material and the old web site.  Paths
below are relative to `bub-n-bros/`.

The program does not use Python packages in the usual way: each entry
point puts the directories it needs (`common/`, `bubbob/`, `display/`...)
on `sys.path` and the modules import each other by bare name
(`import gamesrv`, `import boards`).  A few modules are also imported as
`common.xxx` or `display.xxx`; they use

```python
try:
    from . import modes
except ImportError:
    import modes
```

so that both ways work.

```
bub-n-bros/
├── BubBob.py            entry point: start the web launcher + open a browser
├── INSTALL.txt          installation notes, requirements
├── CHANGELOG.md
├── VERSION
├── Makefile             builds display/xshm and the man pages
├── bubbob/              the game (server side)
├── common/              generic game-server framework, shared by server and client
├── display/             the Python client and its display / sound drivers
├── http2/               the web launcher
├── java/                the Java applet client
├── metaserver/          Internet game directory and NAT traversal
├── doc/                 man pages
├── docs/                this documentation
└── packing/arch/        Arch Linux PKGBUILD
```

## `bubbob/` – the game

| File | Contents |
|---|---|
| `bb.py` | Entry point of the server.  `BubBobGame` (a `gamesrv.Game`) holds the game options, runs the frame loop (`FnFrame`) and defines the key list.  Command-line parsing. |
| `boards.py` | `Board` base class, board life cycle (`next_board`, `normal_play`, `bonus_play`, `last_monster_killed`, `game_reset`), wall and wind helpers (`bget`, `wget`, `onground`), board-wide effects (earthquakes, repulsion...), and `loadmodules()` which (re)loads the level file. |
| `boarddef.py` | `Level`: a friendlier `Board` subclass used by the `.py` level files, and the monster placement helpers `LNasty`, `RNasty`, ... |
| `binboards.py`, `macbinary.py` | Loader for the original MacOS `.bin` level files (MacBinary resource forks with `LEVL` and `ppat` resources). |
| `player.py` | `BubPlayer` (a connected player: score, lives, keys, team) and `Dragon` (the sprite the player controls), the score board. |
| `monsters.py` | `Monster` and its subclasses (`Nasty`, `Monky`, `Ghosty`, `Flappy`, `Springy`, `Orcy`, `Gramy`, `Blitzy`), their shots, angry/ghost behaviour. |
| `bubbles.py` | Bubbles blown by the dragons, and the special bubbles: letters, fire, water, lightning, ... |
| `bonuses.py` | Every bonus item (fruit, potions, shoes, bombs, ...), each a class with a `taken()` method.  `Classes` is the list the random bonuses are drawn from. |
| `ranking.py` | End-of-game statistics and ranking screens. |
| `images.py` | `ActiveSprite` (a sprite driven by generators), the generator dispatch loop `action()`, collision lookup `touching()`, sprite and sound tables (`sprget`, `Snd`), music. |
| `sprmap.py`, `mnstrmap.py`, `patmap.py` | Tables mapping sprite numbers to rectangles in the `images/*.ppm` files, monster animation frames, and wall patterns per level number. |
| `ext1/` … `ext7/` | The seven "potion 4" mini-games: Arkanoid, Pac-Man, Galaga, Tetris, Lemmings, Tron and Camel. Each package has `run()` and `min_players`. |
| `levels/` | Level sets: `*.bin` (original game), `*.py` (hand-written), `RandomLevels.py` + `rnglevel` (random generator). See [Creating levels](creating-levels.md). |
| `images/` | Sprites as PPM files.  `*_0.ppm`/`*_1.ppm` are the originals; `buildcolors.py` generates the other player colours (`_2` … `_9`) from `palettes.dat`. |
| `tmp/` | Wall pattern images (`pat*.ppm`) used by the `.py` levels. |
| `sounds/`, `music/` | WAV files. |
| `doc/bonus-doc.py` | Generates `doc/bonuses.html` with an image of every bonus. |
| `statesaver.c`, `statesaver.py` | Deep copy of running generators, used by the clock bonus.  Python 2 only; without it the clock bonus is simply disabled. |
| `test_rnglevel.py`, `save_rnglevel.py` | Check / dump the random level generator. |

## `common/` – the network game framework

| File | Contents |
|---|---|
| `gamesrv.py` | Sockets and the main `select()` loop (`mainloop`, `recursiveloop`), `Game` and `Client` classes, `Bitmap`/`Icon`/`Sample`/`Music` data objects, the global sprite list and `Sprite` class, the per-frame `sendudpdata()`, frame compression, optional game recording. |
| `msgstruct.py` | Message type constants and `message()` / `decodemessage()` encoding (see [Architecture](architecture.md#the-network-protocol)). |
| `hostchooser.py` | LAN discovery over UDP (port 8056) for both sides. |
| `httpserver.py` | Minimal HTTP request handler with page "loaders" and the `%(...)s` template translator. |
| `javaserver.py` | Serves the Java applet and its HTML page, converts sounds to `.au`. |
| `pixmap.py` | Pure-Python PPM image helpers (crop, flip, rotate, darken, zoom). |
| `udpovertcp.py` | Sends frames inline over TCP when UDP does not work. |
| `stdlog.py` | Size-limited log files for background servers. |
| `py2sort.py` | `py2key`: sort key reproducing Python 2's ordering of mixed tuples. |

## `display/` – the Python client

| File | Contents |
|---|---|
| `Client.py` | Entry point: command line, server discovery, then `pclient.run()`. |
| `pclient.py` | `Playfield`: the client side of the protocol, sprite drawing, the player taskbar and key definition, sound and music triggering. |
| `caching.py` | Local file lookup by MD5 and the download cache. |
| `modes.py` | List of display and sound drivers and their options; `open_dpy()` / `open_snd()`. |
| `dpy_pygame.py`, `dpy_gtk.py`, `dpy_x.py` (+ `xshm.c`), `dpy_windows.py` | Display drivers. |
| `snd_pygame.py`, `snd_off.py`, `snd_linux.py`, `snd_windows.py`, `puremixer.py`, `music1.py` | Sound drivers and a pure-Python mixer. |
| `playback.py` | Replays a recorded game file. |
| `pythonxlibintf.py` | Unused, very slow python-xlib display experiment. |

## `http2/` – web launcher

| File | Contents |
|---|---|
| `httppages.py` | `PageServer`: the pages under `http://127.0.0.1:8000/<seed>/`, game start/stop, client launching, options file. |
| `data/*.html` | Page templates (with embedded Python, see [Architecture](architecture.md#web-launcher)) and their images. |
| `sf/bb12.py` | Old SourceForge CGI server list (historical). |

## `metaserver/`

| File | Contents |
|---|---|
| `metaserver.py` | The directory server (TCP/UDP 8055, HTTP 8050). |
| `metaclient.py` | Registration for servers, listing and connection set-up for clients (direct, back-connect, SYN, UDP tunnel). |
| `metastruct.py` | Message types and `MessageSocket`. |
| `pipelayer.py`, `socketoverudp.py` | Reliable, ordered stream over UDP, used for NAT traversal. |
