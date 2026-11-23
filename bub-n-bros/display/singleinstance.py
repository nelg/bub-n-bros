"""One client per screen when the web launcher starts it.

Clicking "Join" several times in the web launcher would otherwise start
several clients on the same screen and sound card.  Clients started by
the launcher (with --single-instance) take a lock named after the
display; if another client holds it, they log a message and stop.  The
native client (rust-client/) uses the same lock file, so a Python client
and a native client also exclude each other.
"""

import os, sys, tempfile


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


_lockfile = None    # kept open (and locked) for the life of the process

def acquire():
    """Returns True if this is the only client on this screen."""
    global _lockfile
    path = lock_path()
    try:
        f = open(path, 'a+')
    except (IOError, OSError):
        return True      # cannot lock: do not prevent playing
    try:
        if sys.platform == 'win32':
            import msvcrt
            f.seek(0)
            msvcrt.locking(f.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(f.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except (IOError, OSError):
        f.close()
        return False
    _lockfile = f
    return True


def check_or_exit():
    if not acquire():
        print("A Bub-n-Bros client is already running on this screen "
              "(lock %s); not starting another one." % lock_path(),
              file=sys.stderr)
        sys.exit(0)
