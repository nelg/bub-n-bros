"""One client per screen when the web launcher starts it.

Clicking "Join" several times in the web launcher would otherwise start
several clients on the same screen and sound card.  Clients started by
the launcher (with --single-instance) take a lock named after the
display, and write their process id and start time in it.

When the lock is already taken:
  * if the other client started less than TAKEOVER_DELAY seconds ago,
    the new one logs "already running" and exits (a double click);
  * otherwise the other client is assumed to be stuck or forgotten: it
    is stopped (only if it runs as the same user) and the new one starts.

The native client (rust-client/src/single.rs) uses the same lock file and
format, so a Python client and a native client also exclude each other.
"""

import os, sys, time, tempfile, signal

TAKEOVER_DELAY = 30.0     # seconds


def lock_path():
    display = (os.environ.get('WAYLAND_DISPLAY') or
               os.environ.get('DISPLAY') or 'console')
    display = ''.join([c if c.isalnum() else '_' for c in display])
    folder = os.environ.get('XDG_RUNTIME_DIR') or tempfile.gettempdir()
    user = ''
    if not os.environ.get('XDG_RUNTIME_DIR'):
        # shared temporary folder: one lock per user
        name = os.environ.get('USER') or os.environ.get('USERNAME') or ''
        user = '-' + ''.join([c if c.isalnum() else '_' for c in name])
    return os.path.join(folder, 'bubnbros-client%s-%s.lock' % (user, display))


def _trylock(f):
    try:
        if sys.platform == 'win32':
            import msvcrt
            f.seek(0)
            msvcrt.locking(f.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(f.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        return True
    except (IOError, OSError):
        return False


def _read_owner(path):
    # "pid start-time" written by the client holding the lock
    try:
        f = open(path, 'r')
        words = f.read().split()
        f.close()
        return int(words[0]), float(words[1])
    except (IOError, OSError, ValueError, IndexError):
        return None, None


def _same_user(pid):
    if os.path.isdir('/proc/%d' % pid) and hasattr(os, 'getuid'):
        try:
            return os.stat('/proc/%d' % pid).st_uid == os.getuid()
        except OSError:
            return False
    return True    # elsewhere the system refuses to stop other users' programs


def _stop(pid):
    try:
        os.kill(pid, signal.SIGTERM)
    except (OSError, AttributeError):
        return
    for i in range(20):
        time.sleep(0.1)
        try:
            os.kill(pid, 0)
        except OSError:
            return             # gone
    if hasattr(signal, 'SIGKILL'):
        try:
            os.kill(pid, signal.SIGKILL)
        except OSError:
            pass


_lockfile = None    # kept open (and locked) for the life of the process

def acquire():
    """Returns True if this client may run (it then holds the lock)."""
    global _lockfile
    path = lock_path()
    try:
        f = open(path, 'a+')
    except (IOError, OSError):
        return True      # cannot lock: do not prevent playing
    if not _trylock(f):
        pid, started = _read_owner(path)
        if pid is None or time.time() - started < TAKEOVER_DELAY:
            f.close()
            return False
        if pid == os.getpid() or not _same_user(pid):
            f.close()
            return False
        print("Stopping the Bub-n-Bros client already running on this "
              "screen (pid %d, started %d seconds ago)." %
              (pid, time.time() - started), file=sys.stderr)
        _stop(pid)
        for i in range(30):
            if _trylock(f):
                break
            time.sleep(0.1)
        else:
            f.close()
            return False
    try:
        f.seek(0)
        f.truncate()
        f.write('%d %f\n' % (os.getpid(), time.time()))
        f.flush()
    except (IOError, OSError):
        pass
    _lockfile = f
    return True


def check_or_exit():
    if not acquire():
        print("A Bub-n-Bros client is already running on this screen "
              "(lock %s); not starting another one." % lock_path(),
              file=sys.stderr)
        sys.exit(0)
