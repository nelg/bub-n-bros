"""Make the game's quiet sound files louder.

The sounds and music of the game are 8-bit recordings that use only a
third to a half of the available range (peaks around 30-50%), so they
play quietly.  boost_wav() returns the same sound as a 16-bit WAV file,
amplified so that its loudest point reaches about 95% of full scale (with
the gain limited to MAX_GAIN).  It only uses the standard library and is
fast enough for the 10 MB music files (the conversion uses
bytes.translate(), which runs in C).
"""

import io, wave, array, sys

MAX_GAIN = 3.0
TARGET_PEAK = 0.95


def boost_wav(data, max_gain=MAX_GAIN):
    """data: bytes of a WAV file.  Returns the bytes of a louder 16-bit
    WAV file, or the original data if it cannot be handled."""
    try:
        w = wave.open(io.BytesIO(data), 'rb')
        channels = w.getnchannels()
        width = w.getsampwidth()
        rate = w.getframerate()
        frames = w.readframes(w.getnframes())
        w.close()
    except (wave.Error, EOFError, ValueError):
        return data
    if not frames:
        return data
    if width == 1:
        # unsigned 8-bit: centre is 128
        peak = max(max(frames) - 128, 128 - min(frames), 1)
        gain = min(max_gain, TARGET_PEAK * 128.0 / peak)
        lo = bytearray(256)
        hi = bytearray(256)
        for b in range(256):
            v = int(round((b - 128) * 256 * gain))
            v = max(-32768, min(32767, v)) & 0xFFFF
            lo[b] = v & 0xFF
            hi[b] = v >> 8
        out = bytearray(2 * len(frames))
        out[0::2] = frames.translate(bytes(lo))   # little-endian 16-bit
        out[1::2] = frames.translate(bytes(hi))
        samples = bytes(out)
    elif width == 2:
        a = array.array('h', frames)
        if sys.byteorder == 'big':
            a.byteswap()
        peak = max(max(a), -min(a), 1)
        gain = min(max_gain, TARGET_PEAK * 32768.0 / peak)
        if gain <= 1.01:
            return data
        a = array.array('h', [max(-32768, min(32767, int(x * gain))) for x in a])
        if sys.byteorder == 'big':
            a.byteswap()
        samples = a.tobytes()
    else:
        return data
    f = io.BytesIO()
    w = wave.open(f, 'wb')
    w.setnchannels(channels)
    w.setsampwidth(2)
    w.setframerate(rate)
    w.writeframes(samples)
    w.close()
    return f.getvalue()
