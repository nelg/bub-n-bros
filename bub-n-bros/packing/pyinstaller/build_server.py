#! /usr/bin/env python3
"""Build the single-file game server (bubnbros-server, .exe on Windows)
with PyInstaller.

    pip install pyinstaller
    python3 packing/pyinstaller/build_server.py

The result is in packing/pyinstaller/dist/.  It contains a Python
interpreter, the server code and all the game data, so it runs on
machines without Python.  PyInstaller builds for the system it runs on:
build on Windows for Windows, on macOS for macOS (the GitHub workflow
does all three).
"""

import os, sys, subprocess, shutil

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))   # bub-n-bros/
SEP = os.pathsep


def main():
    # generate the coloured images first, so that the server does not
    # have to do it each time it starts
    subprocess.check_call([sys.executable,
                           os.path.join(ROOT, 'bubbob', 'images', 'buildcolors.py')],
                          cwd=os.path.join(ROOT, 'bubbob', 'images'))
    work = os.path.join(HERE, 'build')
    dist = os.path.join(HERE, 'dist')
    data = []
    # bubbob/ and common/: the game; http2/, display/, java/, metaserver/:
    # the web launcher pages (--web), which start the native client
    for d in ('bubbob', 'common', 'http2', 'display', 'java', 'metaserver'):
        data += ['--add-data', '%s%s%s' % (os.path.join(ROOT, d), SEP, d)]
    cmd = [sys.executable, '-m', 'PyInstaller', '--onefile', '--noconfirm',
           '--clean', '--name', 'bubnbros-server',
           '--workpath', work, '--distpath', dist, '--specpath', work,
           # not needed by the server, keep the file small
           '--exclude-module', 'tkinter', '--exclude-module', 'pygame',
           '--exclude-module', 'gi', '--exclude-module', 'cairo',
           ] + data + [os.path.join(HERE, 'server_main.py')]
    print(' '.join(cmd))
    subprocess.check_call(cmd)
    print('built', os.path.join(dist, 'bubnbros-server' +
                                (sys.platform == 'win32' and '.exe' or '')))

if __name__ == '__main__':
    main()
