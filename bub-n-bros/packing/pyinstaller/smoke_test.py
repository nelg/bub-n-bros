#! /usr/bin/env python3
"""Start the single-file server, check that it answers like a game
server, and stop it.

    python3 smoke_test.py path/to/bubnbros-server
    python3 smoke_test.py [--level=FILE] COMMAND [ARGS...]

e.g. smoke_test.py bubbros-native --server, or, for the Python server
(which needs a level file, given last), smoke_test.py
--level=/usr/share/bubbros/bubbob/levels/Arena.bin bubnbros-server"""

import socket, subprocess, sys, time

def free_port():
    s = socket.socket()
    s.bind(('127.0.0.1', 0))
    port = s.getsockname()[1]
    s.close()
    return port

def main(command, level=None):
    port, http = free_port(), free_port()
    p = subprocess.Popen(command + ['--port', 'LISTEN=%d' % port,
                                    '--port', 'HTTP=%d' % http] +
                         ([level] if level else []),
                         stdin=subprocess.DEVNULL)
    try:
        deadline = time.time() + 120
        while time.time() < deadline:
            try:
                s = socket.create_connection(('127.0.0.1', port), timeout=2)
            except OSError:
                if p.poll() is not None:
                    sys.exit('server stopped with code %s' % p.returncode)
                time.sleep(0.5)
                continue
            s.settimeout(20)
            data = b''
            while data.count(b'\n') < 2:
                chunk = s.recv(100)
                if not chunk:
                    break
                data += chunk
            s.close()
            print(data.decode('utf-8', 'replace'))
            if not data.startswith(b'Welcome to gamesrv.py'):
                sys.exit('not a game server: %r' % data)
            print('OK')
            return
        sys.exit('the server did not start')
    finally:
        p.terminate()
        try:
            p.wait(10)
        except subprocess.TimeoutExpired:
            p.kill()

if __name__ == '__main__':
    args = sys.argv[1:]
    level = None
    if args and args[0].startswith('--level='):
        level = args.pop(0)[len('--level='):]
    main(args, level)
