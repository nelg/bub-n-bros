#! /usr/bin/env python3
"""Start the single-file server, check that it answers like a game
server, and stop it.   python3 smoke_test.py path/to/bubnbros-server"""

import socket, subprocess, sys, time

def free_port():
    s = socket.socket()
    s.bind(('127.0.0.1', 0))
    port = s.getsockname()[1]
    s.close()
    return port

def main(exe):
    port, http = free_port(), free_port()
    p = subprocess.Popen([exe, '--port', 'LISTEN=%d' % port,
                          '--port', 'HTTP=%d' % http],
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
    main(sys.argv[1])
