
 ################################################
##     pygame-based implementation of xshm     ##
################################################

import os
try:
    from . import sdlaudio
except ImportError:
    import sdlaudio
sdlaudio.setup()     # before pygame initializes its sound
import pygame
from pygame.locals import *
try:
    from .modes import KeyPressed, KeyReleased
except ImportError:
    from modes import KeyPressed, KeyReleased


class Display:
    musthidemouse = 0
    mousevisible = 1

    def __init__(self, width, height, title, transparency='yes',
                 fullscreen='no', zoom='100%', smooth='yes',
                 smoothfast='yes'):
        self.use_transparency = not transparency.startswith('n')
        self.use_fullscreen = fullscreen.startswith('y')
        if zoom.endswith('%'):
            zoom = zoom[:-1]
        scale = float(zoom) / 100.0
        iscale = int(scale+0.001)
        if abs(scale - iscale) < 0.002:
            scale = iscale
        self.scale = scale
        self.smooth = smooth.startswith('y')
        self.smoothfast = smoothfast.startswith('y')

        # Initialize pygame
        pygame.init()

        # Set the display mode
        self.width = width
        self.height = height
        self.title = title
        self.set_display_mode()
        self.offscreen = pygame.Surface((width, height))
        #decorate the game window
        pygame.display.set_caption(title)
        #pygame.mouse.set_visible(0)
        self.tbcache = None, None
        self.events_key = []
        self.events_mouse = []
        self.prevposition = None
        self.swallowed = {}
        EVENT_HANDLERS[KEYDOWN] = self.keydown_handler
        EVENT_HANDLERS[KEYUP] = self.keyup_handler
        EVENT_HANDLERS[MOUSEBUTTONDOWN] = self.mousebuttondown_handler

    def set_display_mode(self):
        scaled = getattr(self, 'scaled_mode', None)
        if scaled is not None and scaled != self.use_fullscreen:
            # SDL cannot drop the SCALED flag of a window: start afresh
            pygame.display.quit()
            pygame.display.init()
            self.mousevisible = 1
        if self.use_fullscreen:
            # "desktop" full screen: SDL scales the game image to fit the
            # screen, keeping its proportions, and maps the mouse back
            self.screen = pygame.display.set_mode((self.width, self.height),
                                                  FULLSCREEN | SCALED)
        else:
            size = (int(self.width * self.scale),
                    int(self.height * self.scale))
            self.screen = pygame.display.set_mode(size, RESIZABLE)
        self.scaled_mode = self.use_fullscreen
        pygame.display.set_caption(self.title)
        self.screensize = self.screen.get_size()
        self.compute_viewport()

    def compute_viewport(self):
        # where the game image goes on the screen: (x, y, w, h, scale).
        # In a window, the game is scaled to fill it (keeping its
        # proportions), so the window can be resized or maximised.
        sw, sh = self.screen.get_size()
        if self.use_fullscreen:
            scale = 1      # SDL does the scaling
        else:
            scale = min(float(sw) / self.width, float(sh) / self.height)
        w = int(self.width * scale)
        h = int(self.height * scale)
        self.viewport = ((sw - w) // 2, (sh - h) // 2, w, h, scale)
        self.screen.fill((0, 0, 0))

    def toggle_fullscreen(self):
        self.use_fullscreen = not self.use_fullscreen
        self.set_display_mode()

    def controlkey(self, key):
        # the client's own keys (see pclient.Playfield.control())
        return CONTROL_KEYS.get(key)

    def keydown_handler(self, e):
        # F11 or Alt+Enter toggle full screen; Esc leaves full screen, and
        # quits when in a window
        if e.key == K_ESCAPE and not self.use_fullscreen:
            raise SystemExit
        if (e.key == K_F11 or (e.key == K_RETURN and e.mod & KMOD_ALT) or
            (e.key == K_ESCAPE and self.use_fullscreen)):
            self.swallowed[e.key] = 1     # don't send its release either
            self.toggle_fullscreen()
            return
        self.showmouse(not self.musthidemouse)
        self.events_key.append((e.key, KeyPressed))
        del self.events_key[:-16]

    def keyup_handler(self, e):
        if self.swallowed.pop(e.key, None):
            return
        self.events_key.append((e.key, KeyReleased))
        del self.events_key[:-16]

    def mousebuttondown_handler(self, e):
        self.showmouse(1)
        self.events_mouse.append(self.fixpos(e.pos))
        del self.events_mouse[:-8]

    def pixmap(self, w, h, data, colorkey=-1):
        img = pygame.image.frombytes(data, (w, h), "RGB")
        if colorkey >= 0:
            r = colorkey & 0xFF
            g = (colorkey >> 8) & 0xFF
            b = (colorkey >> 16) & 0xFF
            img.set_colorkey([r, g, b])
        return img   # not optimized -- must use getopticon()

    def getopticon(self, pixmap, rect, alpha=255):
        if not self.use_transparency:
            alpha = 255
        img = pixmap.subsurface(rect)
        colorkey = pixmap.get_colorkey()
        if alpha == 255 and not colorkey:
            return img.convert(self.offscreen)
        else:
            if colorkey:
                img.set_colorkey(colorkey, RLEACCEL)
            if alpha < 255:
                img.set_alpha(alpha, RLEACCEL)
            img = img.convert_alpha(self.offscreen)
            img.set_alpha(255, RLEACCEL)
            return img

##    def vflipppm(self, img):
##        w, h = img.get_size()
##        colorkey = img.get_colorkey()
##        data = pygame.image.tostring(img, "RGB", 1)
##        flipimg = pygame.image.fromstring(data, (w, h), "RGB")
##        flipimg.set_colorkey(colorkey, RLEACCEL)
##        return flipimg, h

    def getppm(self, rect):
        bkgnd = pygame.Surface(rect[2:])
        bkgnd.blit(self.offscreen, (0, 0), rect)
        return bkgnd

    def putppm(self, x, y, bitmap, rect=None):
        if rect:
            self.offscreen.blit(bitmap, (x, y), rect)
        else:
            self.offscreen.blit(bitmap, (x, y))

    def flip(self):
        offscreen = self.offscreen
        if self.screen.get_size() != self.screensize:
            self.screensize = self.screen.get_size()   # window resized
            self.compute_viewport()
        x, y, w, h, scale = self.viewport
        if (w, h) != offscreen.get_size():
            if (scale == 2 and self.smoothfast and
                (w, h) == (self.width*2, self.height*2)):
                offscreen = pygame.transform.scale2x(offscreen)
            elif self.smooth:
                offscreen = pygame.transform.smoothscale(offscreen, (w, h))
            else:
                offscreen = pygame.transform.scale(offscreen, (w, h))
        self.screen.blit(offscreen, (x, y))
        pygame.display.flip()
        events_dispatch()

    def close(self):
        self.showmouse(1)
        pygame.display.quit()

    def clear(self):
        self.offscreen.fill([0,0,0,])

    def fixpos(self, pos):
        # screen coordinates -> game coordinates
        (x, y) = pos
        x0, y0, w, h, scale = self.viewport
        return (int((x - x0) / scale), int((y - y0) / scale))

    def events_poll(self):
        while 1:
            e = pygame.event.poll()
            if e.type == NOEVENT:
                break
            elif e.type == KEYDOWN:
                self.events_key.append((e.key, KeyPressed))
                del self.events_key[:-16]
            elif e.type == KEYUP:
                self.events_key.append((e.key, KeyReleased))
                del self.events_key[:-16]
            elif e.type == MOUSEBUTTONDOWN:
                self.events_mouse.append(self.fixpos(e.pos))
                del self.events_mouse[:-8]
            elif e.type == ENDMUSICEVENT:
                self.next_music()
            elif e.type == QUIT:
                raise SystemExit

    def keyevents(self):
        events_dispatch()
        events = self.events_key
        self.events_key = []
        return events

    def pointermotion(self):
        position = pygame.mouse.get_pos()
        if position != self.prevposition:
            self.showmouse(1)
            self.prevposition = position
            return self.fixpos(position)
        else:
            return None

    def mouseevents(self):
        events_dispatch()
        events = self.events_mouse
        self.events_mouse = []
        return events

    def selectlist(self):
        return []

    def taskbar(self, rect):
        (x, y, w, h) = rect
        tbs, tbh = self.tbcache
        if tbh != h:
            tbs = pygame.Surface((32, h)).convert_alpha(self.offscreen)
            alpha_f = 256.0 / h
            for j in range(h):
                tbs.fill((128, 128, 255, int(j*alpha_f)),
                         (0, j, 32, 1))
            self.tbcache = tbs, h
        for i in range(x, x+w, 32):
            dw = x+w-i
            if dw < 32:
                self.offscreen.blit(tbs, (i, y), (0, 0, dw, h))
            else:
                self.offscreen.blit(tbs, (i, y))

    def settaskbar(self, tb_visible):
        self.showmouse(1)
        self.musthidemouse = not tb_visible # and self.use_fullscreen

    def showmouse(self, v):
        if v != self.mousevisible:
            self.mousevisible = v
            pygame.mouse.set_visible(v)


def quit_handler(e):
    raise SystemExit

CONTROL_KEYS = {
    K_F7: 'volume-down',
    K_F8: 'volume-up',
    K_F9: 'mute',
    }

EVENT_HANDLERS = {
    QUIT: quit_handler,
    }

def events_dispatch(handlers = EVENT_HANDLERS):
    while 1:
        e = pygame.event.poll()
        if e.type == NOEVENT:
            break
        elif e.type in handlers:
            handlers[e.type](e)


def htmloptionstext(nameval):
    return '''
<%s> Full Screen (F11 or Alt+Enter to switch, Esc to leave)</input><%s><br>
<%s> Draw slightly transparent bubbles</input><%s><br>
Scale image by <%s size=5>%%<br>
<%s> Smoothed scaled image</input><%s><br>
<%s> Semi-smoothed scaled image (for 200%% only)</input><%s><br>
''' % (nameval("checkbox", "fullscreen", "yes", default="no"),
       nameval("hidden", "fullscreen", "no"),
       nameval("checkbox", "transparency", "yes", default="yes"),
       nameval("hidden", "transparency", "no"),
       nameval("text", "zoom", default="100"),
       nameval("checkbox", "smooth", "yes", default="yes"),
       nameval("hidden", "smooth", "no"),
       nameval("checkbox", "smoothfast", "yes", default="no"),
       nameval("hidden", "smoothfast", "no"))
