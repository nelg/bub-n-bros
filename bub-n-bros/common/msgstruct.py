from struct import pack, unpack, calcsize

try:
    from localmsg import PORTS
except ImportError:
    PORTS = {}
try:
    from localmsg import HOSTNAME
except ImportError:
    from socket import gethostname
    HOSTNAME = gethostname()


MSG_WELCOME = b"Welcome to gamesrv.py(3) !\n"
MSG_BROADCAST_PORT= b"*"
MSG_DEF_PLAYFIELD = b"p"
MSG_DEF_KEY       = b"k"
MSG_DEF_ICON      = b"r"
MSG_DEF_BITMAP    = b"m"
MSG_DEF_SAMPLE    = b"w"
MSG_DEF_MUSIC     = b"z"
MSG_PLAY_MUSIC    = b"Z"
MSG_FADEOUT       = b"f"
MSG_PLAYER_JOIN   = b"+"
MSG_PLAYER_KILL   = b"-"
MSG_PLAYER_ICON   = b"i"
MSG_PING          = b"g"
MSG_PONG          = b"G"
MSG_INLINE_FRAME  = b"\\"
MSG_PATCH_FILE    = MSG_DEF_MUSIC
MSG_ZPATCH_FILE   = b"P"
MSG_MD5_FILE      = b"M"
MSG_RECORDED      = b"\x00"

CMSG_PROTO_VERSION= b"v"
CMSG_KEY          = b"k"
CMSG_ADD_PLAYER   = b"+"
CMSG_REMOVE_PLAYER= b"-"
CMSG_UDP_PORT     = b"<"
CMSG_ENABLE_SOUND = b"s"
CMSG_ENABLE_MUSIC = b"m"
CMSG_PING         = b"g"
CMSG_PONG         = b"G"
CMSG_DATA_REQUEST = b"M"
CMSG_PLAYER_NAME  = b"n"

BROADCAST_MESSAGE = b"game!"   # less than 6 bytes


def _tobytes(v):
    if isinstance(v, str):
        # 'surrogateescape' makes this the exact inverse of tostr() below
        return v.encode('utf-8', 'surrogateescape')
    if isinstance(v, float):
        return int(v)     # Python 2's struct silently truncated floats
    return v

def tostr(v):
    """Turn a byte string field of a decoded message into text."""
    if isinstance(v, bytes):
        return v.decode('utf-8', 'surrogateescape')
    return v

def message(tp, *values):
    values = [_tobytes(v) for v in values]
    typecodes = ['']
    for v in values:
        if isinstance(v, (bytes, bytearray)):
            typecodes.append('%ds' % len(v))
        elif 0 <= v < 256:
            typecodes.append('B')
        else:
            typecodes.append('l')
    typecodes = ''.join(typecodes).encode('ascii')
    assert len(typecodes) < 256
    return pack(("!B%dsc" % len(typecodes)) + typecodes.decode('ascii'),
                len(typecodes), typecodes, _tobytes(tp), *values)

def decodemessage(data):
    if data:
        limit = data[0] + 1
        if len(data) >= limit:
            typecodes = b"!c" + data[1:limit]
            end = limit + calcsize(typecodes)
            if len(data) >= end:
                return unpack(typecodes, data[limit:end]), data[end:]
            elif end > 1000000:
                raise OverflowError
    return None, data
