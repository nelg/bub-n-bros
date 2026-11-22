import sys, os
from io import StringIO, BytesIO
try:
    from . import httpserver
except ImportError:
    import httpserver

PLAYERNAMES = ['Bub', 'Bob', 'Boob', 'Beb',
               'Biob', 'Bab', 'Bib',
               'Baub', 'Beab', 'Biab']
LOCALDIR = os.path.abspath(os.path.dirname(__file__))
DATADIR  = os.path.join(LOCALDIR, os.pardir, 'http2', 'data')

EMPTY_PAGE = '''<html>
<head><title>No server is running</title></head>
<body><h1>No server is running at the moment.</h1>
</body>
</html>
'''

INDEX_PAGE = '''<html>
<head><title>%(title)s</title></head>
<body><h1>%(title)s</h1>
 <applet code=pclient.class width=%(width)s height=%(height)s>
  <param name="gameport" value="%(gameport)d">
  %(names1)s
 </applet>
<br>
<p align="center"><a href="name.html?%(names2)s">Player Names &amp; Teams</a></p>
</body>
</html>
'''

NAME_LINE1 = '<param name="%s" value="%s">'
NAME_SEP1  = '\n'
NAME_LINE2 = '%s=%s'
NAME_SEP2  = '&'

def playernames(options):
    NUM_PLAYERS = len(PLAYERNAMES)
    result = {}
    anyname = None
    for id in range(NUM_PLAYERS):
        keyid = 'player%d' % id
        if keyid in options:
            value = options[keyid][0]
            anyname = anyname or value
            teamid = 'team%d' % id
            if teamid in options:
                team = options[teamid][0]
                if len(team) == 1:
                    value = '%s (%s)' % (value, team)
            result[keyid] = value
    if 'c' in options:
        for id in range(NUM_PLAYERS):
            keyid = 'player%d' % id
            try:
                del result[keyid]
            except KeyError:
                pass
    if 'f' in options:
        for id in range(NUM_PLAYERS):
            keyid = 'player%d' % id
            if not result.get(keyid):
                result[keyid] = anyname or PLAYERNAMES[id]
            else:
                anyname = result[keyid]
    return result

def indexloader(**options):
    if 'cheat' in options:
        for opt in options.pop('cheat'):
            __cheat(opt)
    try:
        from . import gamesrv
    except ImportError:
        import gamesrv
    if gamesrv.game is None:
        indexdata = EMPTY_PAGE
    else:
        names = list(playernames(options).items())
        indexdata = INDEX_PAGE % {
            'title':    gamesrv.game.FnDesc,
            'width':    gamesrv.game.width,
            'height':   gamesrv.game.height,
            'gameport': gamesrv.game.address[1],
            'names1':   NAME_SEP1.join([NAME_LINE1 % kv for kv in names]),
            'names2':   NAME_SEP2.join([NAME_LINE2 % kv for kv in names]),
            }
    return StringIO(indexdata), 'text/html'

def nameloader(**options):
    if 's' in options:
        return indexloader(**options)
    locals = {
        'options': playernames(options),
        }
    return httpserver.load(os.path.join(DATADIR, 'name.html'),
                           'text/html', locals=locals)


wave_cache = {}

def _lin2ulaw(sample):
    # 16-bit signed linear sample -> 8-bit mu-law (G.711)
    BIAS, CLIP = 0x84, 32635
    sign = 0x80 if sample < 0 else 0
    if sample < 0:
        sample = -sample
    if sample > CLIP:
        sample = CLIP
    sample += BIAS
    exponent = 7
    mask = 0x4000
    while exponent > 0 and not (sample & mask):
        exponent -= 1
        mask >>= 1
    mantissa = (sample >> (exponent + 3)) & 0x0F
    return ~(sign | (exponent << 4) | mantissa) & 0xFF

def wav2au(data):
    # Very limited! Assumes a standard 8-bit mono .wav as input
    # (the 'audioop' module is gone since Python 3.13, so this is done by hand)
    import struct
    freq, = struct.unpack("<i", data[24:28])
    data = data[44:]
    count = len(data) * 8000 // freq if freq else 0
    out = bytearray(count)
    for i in range(count):
        s = data[i * freq // 8000] - 128     # unsigned 8-bit -> signed
        out[i] = _lin2ulaw(s << 8)
    data = bytes(out)
    data = struct.pack('>4siiiii8s',
                       b'.snd',                        # header
                       struct.calcsize('>4siiiii8s'),  # header size
                       len(data),                      # data size
                       1,                              # encoding
                       8000,                           # sample rate
                       1,                              # channels
                       b'magic.au') + data
    return data

def sampleloader(code=[], **options):
    try:
        from . import gamesrv
    except ImportError:
        import gamesrv
    try:
        data = wave_cache[code[0]]
    except KeyError:
        for key, snd in list(gamesrv.samples.items()):
            if str(getattr(snd, 'code', '')) == code[0]:
                data = wave_cache[code[0]] = wav2au(snd.read())
                break
        else:
            raise KeyError(code[0])
    return BytesIO(data), 'audio/wav'


def setup():
    dir = os.path.abspath(os.path.join(os.path.dirname(__file__),
                                       os.pardir,
                                       'java'))
    if not os.path.isdir(dir):
        return

    # register all '.class' files
    for name in os.listdir(dir):
        if name.endswith('.class'):
            httpserver.register(name, httpserver.fileloader(os.path.join(dir, name)))

    # register a '', an 'index.html', and a 'name.html' file
    httpserver.register('', indexloader)
    httpserver.register('index.html', indexloader)
    httpserver.register('name.html',  nameloader)

    # 'name.html' has a few images, list the .png files in DATADIR
    for fn in os.listdir(DATADIR):
        fn = fn.lower()
        if fn.endswith('.png'):
            httpserver.register(fn, httpserver.fileloader(
                os.path.join(DATADIR, fn)))

    # register the sample loader
    httpserver.register('sample.wav', sampleloader)

setup()
