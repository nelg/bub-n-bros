##########################################################
##   "native": the Rust client (rust-client/), which is  ##
##   a separate program, offered by the web launcher      ##
##########################################################

# This is not a display driver for Client.py: choosing "native" in the
# web launcher starts the native client program instead of Client.py.

import os, sys, shutil

LOCALDIR = os.path.dirname(os.path.abspath(__file__))
EXE = sys.platform == 'win32' and '.exe' or ''


def find_binary():
    """Where is the native client?  $BUBNBROS_NATIVE, then
    'bubbros-native' on the PATH (the Arch Linux package), then a build
    in rust-client/target/release/."""
    path = os.environ.get('BUBNBROS_NATIVE')
    if path and os.path.isfile(path):
        return path
    path = shutil.which('bubbros-native')
    if path:
        return path
    path = os.path.join(LOCALDIR, os.pardir, 'rust-client', 'target',
                        'release', 'bubnbros-client' + EXE)
    if os.path.isfile(path):
        return os.path.abspath(path)
    return None


def imperror():
    if find_binary() is None:
        return 'not built (see rust-client/README.md)'


def command(options, address, sound_mode=None):
    """The command line starting the native client with the web
    launcher's settings (a localoptions object of httppages)."""
    args = [find_binary(), '--single-instance']
    if (options.dpy_native_fullscreen or 'no').startswith('y'):
        args.append('--fullscreen')
    zoom = (options.dpy_native_zoom or '').strip().rstrip('%')
    if zoom.isdigit():
        args.append('--zoom=' + zoom)
    if options.datachannel == 'tcp':
        args.append('--tcp')
    elif options.datachannel == 'udp':
        args.append('--udp')
    if sound_mode == 'off':
        args.append('--no-sound')
    elif options.music == 'no':
        args.append('--no-music')
    for n in range(10):
        name = getattr(options, 'player%d' % n)
        if name:
            args.append('--player=%d=%s' % (n, name))
    args.append(address)
    return args


class Display:
    def __init__(self, *args, **kwds):
        raise SystemExit("the 'native' client is a separate program: run %s"
                         % (find_binary() or 'rust-client/target/release/bubnbros-client'))


def htmloptionstext(nameval):
    return '''
<font size=-1>A separate program, with its own sound and music
(the sound settings below are not used, except "off")</font><br>
<%s> Full Screen (F11 or Alt+Enter to switch, Esc to leave)</input><%s><br>
Window size <%s size=5>%% of the game size<br>
''' % (nameval("checkbox", "fullscreen", "yes", default="no"),
       nameval("hidden", "fullscreen", "no"),
       nameval("text", "zoom", default="100"))
