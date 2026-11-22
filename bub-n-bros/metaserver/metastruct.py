import sys, os
LOCALDIR = __file__
LOCALDIR = os.path.abspath(os.path.dirname(LOCALDIR))
sys.path.insert(0, os.path.dirname(LOCALDIR))

from common.msgstruct import *
from socket import error

MMSG_INFO     = 'I'
MMSG_START    = '+'
MMSG_STOP     = '-'
MMSG_LIST     = 'L'
MMSG_ROUTE    = 'R'
MMSG_TRACEBACK= 'T'
MMSG_UDP_ADDR = 'U'

RMSG_WAKEUP   = 'w'
RMSG_PING     = 'p'
RMSG_PONG     = 'o'
RMSG_SYNC     = 'y'
RMSG_CONNECT  = 'c'
RMSG_LIST     = 'l'
RMSG_UDP_ADDR = 'u'
RMSG_UDP_CONN = 'd'
RMSG_NO_HOST  = '?'


# The meta-server protocol is handled as text: all byte string fields of
# incoming messages are turned into str with tostr(), which round-trips
# exactly with the encoding done by message().

def textmessage(msg):
    if msg is None:
        return None
    return tuple([tostr(v) for v in msg])

def encodedict(dict):
    data = []
    for key, value in list(dict.items()):
        data.append(message('#', key, value))
    return b''.join(data)

def encodelist(list):
    return message('[', *list)

def decodedict(buffer):
    if isinstance(buffer, str):
        buffer = buffer.encode('utf-8', 'surrogateescape')
    result = {}
    while 1:
        msg, buffer = decodemessage(buffer)
        msg = textmessage(msg)
        if msg is None or len(msg) < 3 or msg[0] != '#':
            break
        result[msg[1]] = msg[2]
    return result

def decodelist(buffer):
    if isinstance(buffer, str):
        buffer = buffer.encode('utf-8', 'surrogateescape')
    msg, buffer = decodemessage(buffer)
    msg = textmessage(msg)
    assert msg[0] == '['
    return list(msg[1:])


class MessageSocket:
    
    def __init__(self, s):
        self.s = s
        self.buffer = b""

    def receive(self):
        try:
            data = self.s.recv(2048)
        except error:
            data = b''
        if not data:
            self.disconnect()
            return
        self.buffer += data
        while 1:
            msg, self.buffer = decodemessage(self.buffer)
            if msg is None:
                break
            msg = textmessage(msg)
            if msg[0] not in self.MESSAGES:
                print('unknown message %r' % (msg[0],), file=sys.stderr)
            else:
                fn = self.MESSAGES[msg[0]]
                fn(self, *msg[1:])
