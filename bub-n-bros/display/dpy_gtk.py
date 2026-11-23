 ################################################
##   GTK 3 (PyGObject) implementation of xshm  ##
################################################

# The original driver used PyGTK 2, which does not exist for Python 3.
# This version uses GTK 3 through PyGObject, drawing with cairo: all the
# game graphics go to an offscreen cairo surface at the game's native
# size, which is scaled to the window when it is painted.

import sys
try:
    from .modes import KeyPressed, KeyReleased
except ImportError:
    from modes import KeyPressed, KeyReleased

def import_trickery():
    global Gtk, Gdk, GdkPixbuf, GLib, cairo
    argv = sys.argv[:]
    del sys.argv[1:]
    try:
        import gi
        gi.require_version('Gtk', '3.0')
        gi.require_version('Gdk', '3.0')
        gi.require_version('GdkPixbuf', '2.0')
        from gi.repository import Gtk, Gdk, GdkPixbuf, GLib
        import cairo
    finally:
        sys.argv[:] = argv
import_trickery()


class Display:

    def __init__(self, width, height, title, zoom="100", fullscreen="no"):
        if zoom.endswith('%'):
            zoom = zoom[:-1]
        scale = float(zoom) / 100.0
        iscale = int(scale+0.001)
        if abs(scale - iscale) < 0.002:
            scale = iscale
        self.scale = scale

        self.gamewidth  = width
        self.gameheight = height
        self.width  = int(width * scale)
        self.height = int(height * scale)

        # the offscreen buffer, at the game's native resolution
        self.offscreen = cairo.ImageSurface(cairo.FORMAT_RGB24, width, height)
        self.front = cairo.ImageSurface(cairo.FORMAT_RGB24, width, height)
        self.ctx = cairo.Context(self.offscreen)

        # create a top level window
        w = self.window = Gtk.Window(type=Gtk.WindowType.TOPLEVEL)
        w.connect("destroy", lambda w: sys.exit())
        w.connect("key-press-event", self.key_press_event)
        w.connect("key-release-event", self.key_release_event)
        w.connect("motion-notify-event", self.motion_notify_event)
        w.connect("button-press-event", self.button_press_event)
        w.add_events(Gdk.EventMask.KEY_PRESS_MASK |
                     Gdk.EventMask.KEY_RELEASE_MASK |
                     Gdk.EventMask.POINTER_MOTION_MASK |
                     Gdk.EventMask.BUTTON_PRESS_MASK)
        self.area = Gtk.DrawingArea()
        self.area.set_size_request(self.width, self.height)
        self.area.connect("draw", self.draw_event)
        w.add(self.area)
        w.set_title(title)
        w.show_all()
        self.is_fullscreen = False
        self.quit_requested = False
        self.swallowed = {}
        if fullscreen.startswith('y'):
            self.toggle_fullscreen()

        self.events_key = []
        self.events_mouse = []
        self.event_motion = None

        pixel = b"\x00\x00\x80"
        hole  = b"\x01\x01\x01"
        self.taskbkgnd = self.pixmap(32, 32,
                                     ((pixel+hole)*16 + (hole+pixel)*16) * 16,
                                     0x010101)
        self.clear()
        self.events_poll()

    def pixmap(self, w, h, data, colorkey=-1):
        pb = GdkPixbuf.Pixbuf.new_from_bytes(GLib.Bytes.new(data),
                                             GdkPixbuf.Colorspace.RGB,
                                             False, 8, w, h, w*3)
        if colorkey >= 0:
            pb = pb.add_alpha(True, colorkey >> 16,
                              (colorkey >> 8) & 0xFF,
                              colorkey & 0xFF)
        surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, w, h)
        ctx = cairo.Context(surface)
        Gdk.cairo_set_source_pixbuf(ctx, pb, 0, 0)
        ctx.paint()
        return surface

    def taskbar(self, rect):
        (x, y, w, h) = rect
        ctx = self.ctx
        ctx.save()
        pattern = cairo.SurfacePattern(self.taskbkgnd)
        pattern.set_extend(cairo.EXTEND_REPEAT)
        pattern.set_matrix(cairo.Matrix(x0=-x, y0=-y))
        ctx.set_source(pattern)
        ctx.rectangle(x, y, w, h)
        ctx.fill()
        ctx.restore()

    def getppm(self, rect):
        (x, y, w, h) = rect
        bkgnd = cairo.ImageSurface(cairo.FORMAT_RGB24, max(w, 1), max(h, 1))
        ctx = cairo.Context(bkgnd)
        ctx.set_source_surface(self.offscreen, -x, -y)
        ctx.paint()
        return bkgnd

    def putppm(self, x, y, surface, rect=None):
        if surface is None:
            return
        ctx = self.ctx
        if rect is None:
            ctx.set_source_surface(surface, x, y)
            ctx.paint()
        else:
            srcx, srcy, w, h = rect
            ctx.set_source_surface(surface, x-srcx, y-srcy)
            ctx.rectangle(x, y, w, h)
            ctx.fill()

    def flip(self):
        # GTK paints later, when the window asks for it, and by then the
        # client may have erased the taskbar and the help text from the
        # offscreen buffer: paint from a copy of the finished frame
        ctx = cairo.Context(self.front)
        ctx.set_source_surface(self.offscreen, 0, 0)
        ctx.paint()
        self.area.queue_draw()
        self.events_poll()

    def viewport(self):
        # where the game image goes in the window: (x, y, scale)
        aw = self.area.get_allocated_width()
        ah = self.area.get_allocated_height()
        # the game fills the window or the screen, keeping its proportions
        # (--zoom only sets the initial window size)
        scale = min(float(aw) / self.gamewidth, float(ah) / self.gameheight)
        if scale <= 0:
            scale = self.scale
        x = (aw - self.gamewidth * scale) / 2.0
        y = (ah - self.gameheight * scale) / 2.0
        return max(x, 0), max(y, 0), scale

    def draw_event(self, widget, ctx):
        ctx.set_source_rgb(0, 0, 0)
        ctx.paint()
        x, y, scale = self.viewport()
        ctx.translate(x, y)
        if scale != 1:
            ctx.scale(scale, scale)
        ctx.set_source_surface(self.front, 0, 0)
        if scale != 1:
            ctx.get_source().set_filter(cairo.FILTER_GOOD)
        ctx.paint()
        return True

    def toggle_fullscreen(self):
        self.is_fullscreen = not self.is_fullscreen
        if self.is_fullscreen:
            self.window.fullscreen()
        else:
            self.window.unfullscreen()

    def controlkey(self, keyval):
        # the client's own keys (see pclient.Playfield.control())
        return {Gdk.KEY_F7: 'volume-down',
                Gdk.KEY_F8: 'volume-up',
                Gdk.KEY_F9: 'mute'}.get(keyval)

    def is_toggle_key(self, event):
        alt = event.state & Gdk.ModifierType.MOD1_MASK
        return (event.keyval == Gdk.KEY_F11 or
                (event.keyval == Gdk.KEY_Return and alt) or
                (event.keyval == Gdk.KEY_Escape and self.is_fullscreen))

    def gamepos(self, event):
        x, y, scale = self.viewport()
        return (int((event.x - x) / scale), int((event.y - y) / scale))

    def close(self):
        self.window.destroy()
        self.events_poll()

    def clear(self):
        ctx = self.ctx
        ctx.set_source_rgb(0, 0, 0)
        ctx.paint()

    def events_poll(self):
        while Gtk.events_pending():
            Gtk.main_iteration_do(False)

    def key_press_event(self, window, event):
        # F11 or Alt+Enter toggle full screen; Esc leaves full screen,
        # and quits when in a window
        if event.keyval == Gdk.KEY_Escape and not self.is_fullscreen:
            self.quit_requested = True
            return True
        if self.is_toggle_key(event):
            self.swallowed[event.keyval] = 1
            self.toggle_fullscreen()
            return True
        self.events_key.append((event.keyval, KeyPressed))

    def key_release_event(self, window, event):
        if self.swallowed.pop(event.keyval, None):
            return True
        self.events_key.append((event.keyval, KeyReleased))

    def motion_notify_event(self, window, event):
        self.event_motion = self.gamepos(event)

    def button_press_event(self, window, event):
        self.events_mouse.append(self.gamepos(event))

    def keyevents(self):
        self.events_poll()
        if self.quit_requested:
            raise SystemExit
        result = self.events_key
        self.events_key = []
        return result

    def pointermotion(self):
        result = self.event_motion
        self.event_motion = None
        return result

    def mouseevents(self):
        self.events_poll()
        result = self.events_mouse
        self.events_mouse = []
        return result

    def selectlist(self):
        return []


def htmloptionstext(nameval):
    return '''Scale image by <%s size=5>%%<br>
<%s> Full Screen (F11 or Alt+Enter to switch, Esc to leave)</input><%s><br>
''' % (nameval('text', 'zoom', default='100'),
       nameval("checkbox", "fullscreen", "yes", default="no"),
       nameval("hidden", "fullscreen", "no"))
