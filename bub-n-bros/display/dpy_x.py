import sys
import xshm
try:
    from .modes import BaseDisplay, KeyPressed, KeyReleased
except ImportError:
    from modes import BaseDisplay, KeyPressed, KeyReleased

XK_F7     = 0xffc4
XK_F8     = 0xffc5
XK_F9     = 0xffc6
XK_F11    = 0xffc8
XK_Escape = 0xff1b
from io import StringIO


class Display(BaseDisplay):
    
    def __init__(self, width, height, title, shm='yes', fullscreen='no'):
        use_shm = not shm.startswith('n')
        self.xdpy = xdpy = xshm.Display(width, height, use_shm)
        xdpy.settitle(title)
        self.pixmap = xdpy.pixmap
        self.getppm = xdpy.getppm
        self.putppm = xdpy.putppm
        self.overlayppm = xdpy.overlayppm
        self.close  = xdpy.close
        self.clear  = xdpy.clear
        self.flip   = xdpy.flip
        self.mouseevents = xdpy.mouseevents
        self.pointermotion = xdpy.pointermotion
        self.is_fullscreen = False
        self.swallowed = {}
        if fullscreen.startswith('y'):
            self.toggle_fullscreen()
        if use_shm and not xdpy.shmmode():
            print("Note: cannot use SHM extension (%dx%d), display will be slow." % \
                  (width, height), file=sys.stderr)

    def toggle_fullscreen(self):
        self.is_fullscreen = not self.is_fullscreen
        self.xdpy.setfullscreen(self.is_fullscreen)

    def keyevents(self):
        # F11 toggles full screen; Esc leaves full screen, and quits when
        # in a window
        result = []
        for keysym, event in self.xdpy.keyevents():
            if (event == KeyPressed and keysym == XK_Escape and
                not self.is_fullscreen):
                raise SystemExit
            if event == KeyPressed and (keysym == XK_F11 or
                         (keysym == XK_Escape and self.is_fullscreen)):
                self.swallowed[keysym] = 1
                self.toggle_fullscreen()
            elif event == KeyReleased and self.swallowed.pop(keysym, None):
                pass
            else:
                result.append((keysym, event))
        return result

    def controlkey(self, keysym):
        # the client's own keys (see pclient.Playfield.control())
        return {XK_F7: 'volume-down', XK_F8: 'volume-up',
                XK_F9: 'mute'}.get(keysym)

    def selectlist(self):
        if hasattr(self.xdpy, 'fd'):
            from socket import fromfd, AF_INET, SOCK_STREAM
            return [fromfd(self.xdpy.fd(), AF_INET, SOCK_STREAM)]
        else:
            return []


def htmloptionstext(nameval):
    return '''
<%s> Use the shared memory extension</input><%s><br>
<font size=-1>Note: Disable it for remote connections or old X servers</font><br>
<%s> Full Screen (F11 to switch, Esc to leave)</input><%s><br>
''' % (nameval("checkbox", "shm", "yes", default="yes"),
       nameval("hidden", "shm", "no"),
       nameval("checkbox", "fullscreen", "yes", default="no"),
       nameval("hidden", "fullscreen", "no"))
