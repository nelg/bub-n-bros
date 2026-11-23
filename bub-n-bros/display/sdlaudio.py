"""Choose SDL's audio backend before pygame starts.

SDL (which pygame uses for sound) picks PipeWire first when it is
running.  On systems where PulseAudio is the sound server and PipeWire
also runs (for example for screen sharing), PipeWire then plays the
sound on an output of its own choosing (often HDMI speakers), and the
game does not appear in the PulseAudio mixer.  When a PulseAudio server
is reachable, ask SDL to use it.  pipewire-pulse systems are not
affected: there PulseAudio is PipeWire anyway.

An SDL_AUDIODRIVER set by the user always wins.
"""

import os


def pulseaudio_running():
    if os.environ.get('PULSE_SERVER'):
        return True
    runtime = os.environ.get('XDG_RUNTIME_DIR')
    if runtime and os.path.exists(os.path.join(runtime, 'pulse', 'native')):
        return True
    # PulseAudio in system mode
    return (os.path.exists('/run/pulse/native') or
            os.path.exists('/var/run/pulse/native'))


def setup():
    if not os.environ.get('SDL_AUDIODRIVER') and pulseaudio_running():
        os.environ['SDL_AUDIODRIVER'] = 'pulseaudio'
    # the name shown in the sound settings (SDL 2 and SDL 3 hints)
    os.environ.setdefault('SDL_AUDIO_DEVICE_APP_NAME', "The Bub's Brothers")
    os.environ.setdefault('SDL_APP_NAME', "The Bub's Brothers")
