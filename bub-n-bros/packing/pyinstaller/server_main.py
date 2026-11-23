"""Entry point of the single-file game server built with PyInstaller.

PyInstaller unpacks the game (bubbob/, common/) next to this script in a
temporary folder (sys._MEIPASS).  The game loads its modules and data
files by path, so they are bundled as plain files and this script starts
bubbob/bb.py from there, exactly as "python3 bubbob/bb.py ..." would.

Usage:  bubnbros-server [bb.py options] [level file]
A level file given as a bare name (Levels.bin, CompactLevels.py...) is
looked up in the bundled bubbob/levels/; with no level file at all, the
server plays levels/Levels.bin.  "bubnbros-server --web" starts the web
launcher pages instead (as "python3 bubbob/bb.py" with no arguments).
"""

import os, sys, runpy

# Standard modules used by the game.  The game's own modules are bundled
# as data (they are loaded by path), so PyInstaller cannot see what they
# import: list it here so that it is included.
import socket, select, struct, zlib, hashlib, random, math, time, getopt
import gzip, io, errno, traceback, tempfile, types, collections, string
import colorsys, array, wave, copy, weakref, _thread, threading, pdb
if sys.platform == 'win32':
    import ctypes
import http.server, urllib.parse, html.entities



def watch_parent(pid):
    """Stop when the program that started us (the native client's play
    alone mode, which sets BUBNBROS_PARENT_PID) has gone away, even if it
    was killed without stopping us."""
    import threading

    def alive():
        if sys.platform == 'win32':
            import ctypes
            SYNCHRONIZE = 0x00100000
            k32 = ctypes.windll.kernel32
            h = k32.OpenProcess(SYNCHRONIZE, False, pid)
            if not h:
                return False
            try:
                return k32.WaitForSingleObject(h, 0) == 0x102   # WAIT_TIMEOUT
            finally:
                k32.CloseHandle(h)
        try:
            os.kill(pid, 0)
            return True
        except ProcessLookupError:
            return False
        except OSError:
            return True

    def loop():
        while alive():
            time.sleep(2.0)
        os._exit(0)

    t = threading.Thread(target=loop, daemon=True)
    t.start()


if os.environ.get('BUBNBROS_PARENT_PID', '').isdigit():
    watch_parent(int(os.environ['BUBNBROS_PARENT_PID']))

base = getattr(sys, '_MEIPASS', os.path.dirname(os.path.abspath(__file__)))
bubbob = os.path.join(base, 'bubbob')
script = os.path.join(bubbob, 'bb.py')

args = sys.argv[1:]
if args[:1] == ['--web']:
    args = args[1:]
else:
    names = [a for a in args if not a.startswith('-')]
    # options taking a value as a separate word
    takes_value = ('-b', '-s', '-l', '-M', '--start', '--begin', '--step',
                   '--lives', '--monsters', '--extralife', '--limitlives',
                   '--final', '--port', '--saveurlto')
    level = None
    for i, a in enumerate(args):
        if (not a.startswith('-') and
            not (i > 0 and args[i-1] in takes_value)):
            level = i
    if level is None:
        args.append(os.path.join(bubbob, 'levels', 'Levels.bin'))
    elif not os.path.exists(args[level]):
        candidate = os.path.join(bubbob, 'levels', os.path.basename(args[level]))
        if os.path.exists(candidate):
            args[level] = candidate

sys.argv = [script] + args
sys.path.insert(0, os.path.join(base, 'common'))
sys.path.insert(0, bubbob)
runpy.run_path(script, run_name='__main__')
