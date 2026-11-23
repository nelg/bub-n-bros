import sys, io, os
try:
    from . import wavboost, caching, sdlaudio
except ImportError:
    import wavboost, caching, sdlaudio
sdlaudio.setup()     # before pygame initializes its sound
try:
    from .modes import musichtmloptiontext as htmloptionstext
except ImportError:
    from modes import musichtmloptiontext as htmloptionstext
from pygame.locals import *
import pygame.mixer

if pygame.mixer is None:
    raise ImportError


#ENDMUSICEVENT = USEREVENT


class Sound:
    has_sound = has_music = 0   # until initialized
    volume = 1.0                # 0.0 to 1.0, changed with F7 / F8
    muted = False               # F9

    def __init__(self):
        try:
            pygame.mixer.init()
        except pygame.error as e:
            print("sound disabled: %s" % str(e), file=sys.stderr)
        else:
            self.has_sound = 1
            try:
                from pygame.mixer import music
            except ImportError:
                pass
            else:
                self.has_music = music is not None
        self.cmusics = None

    def close(self):
        try:
            pygame.mixer.stop()
        except pygame.error:
            pass
        if self.has_music:
            try:
                pygame.mixer.music.stop()
            except pygame.error:
                pass

    def sound(self, f):
        # the game's sounds are recorded quietly: play a louder copy
        try:
            data = wavboost.boost_wav(f.read())
            return pygame.mixer.Sound(file=io.BytesIO(data))
        except Exception:
            return pygame.mixer.Sound(f.freezefilename())

    boosted_musics = {}

    def louder_music_file(self, filename):
        # a louder 16-bit copy of a music file, in a temporary file
        try:
            key = filename, os.path.getmtime(filename), os.path.getsize(filename)
            if key not in self.boosted_musics:
                f = open(filename, 'rb')
                data = f.read()
                f.close()
                name = caching.mktemp('.wav')
                f = open(name, 'wb')
                f.write(wavboost.boost_wav(data))
                f.close()
                self.boosted_musics[key] = name
            return self.boosted_musics[key]
        except Exception:
            return filename

    def flop(self):
        # the events are not processed if pygame is not also the display,
        # so ENDMUSICEVENT will not arrive -- poll for end of music
        if self.cmusics and not pygame.mixer.music.get_busy():
            self.next_music()

    def gain(self):
        # loudness is heard roughly logarithmically: square the setting so
        # that each step of F7 / F8 makes a clear difference
        if self.muted:
            return 0.0
        return self.volume * self.volume

    def set_volume(self, volume):
        self.volume = volume
        self.update_music_volume()

    def set_muted(self, muted):
        self.muted = muted
        self.update_music_volume()

    def update_music_volume(self):
        if self.has_music:
            try:
                pygame.mixer.music.set_volume(self.gain())
            except pygame.error:
                pass

    def play(self, sound, lvolume, rvolume):
        g = self.gain()
        if not g:
            return
        channel = pygame.mixer.find_channel(1)
        channel.stop()
        try:
            channel.set_volume(lvolume * g, rvolume * g)
        except TypeError:
            channel.set_volume(0.5 * (lvolume+rvolume) * g)
        channel.play(sound)

    def play_musics(self, musics, loop_from):
        #dpy_pygame.EVENT_HANDLERS[ENDMUSICEVENT] = self.next_music
        #pygame.mixer.music.set_endevent(ENDMUSICEVENT)
        self.cmusics = musics, loop_from, 0
        self.next_music()

    def next_music(self, e=None):
        if self.cmusics:
            musics, loop_from, c = self.cmusics
            if c >= len(musics):  # end
                c = loop_from
                if c >= len(musics):
                    pygame.mixer.music.stop()
                    self.cmusics = None
                    return
            pygame.mixer.music.load(self.louder_music_file(
                musics[c].freezefilename()))
            pygame.mixer.music.play()
            self.update_music_volume()
            self.cmusics = musics, loop_from, c+1

    def fadeout(self, millisec):
        #print "fadeout:", millisec
        pygame.mixer.music.fadeout(millisec)
        self.cmusics = None
