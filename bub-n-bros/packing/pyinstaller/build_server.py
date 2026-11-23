#! /usr/bin/env python3
"""Build the single-file game server (bubnbros-server, .exe on Windows)
with PyInstaller.

    pip install pyinstaller
    python3 packing/pyinstaller/build_server.py [--native-client-only]

The result is in packing/pyinstaller/dist/.  It contains a Python
interpreter, the server code and all the game data, so it runs on
machines without Python.  PyInstaller builds for the system it runs on:
build on Windows for Windows, on macOS for macOS (the GitHub workflow
does all three).

--native-client-only: leave out the Python clients (pygame, GTK, X) and
the Java applet, which cannot run from the packaged server anyway; its
web launcher then only offers the native client (bubbros-native).  Used
for the server-embedded and play-alone downloads and packages.
"""

import os, sys, subprocess, shutil

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))   # bub-n-bros/
SEP = os.pathsep


# display/ files kept with --native-client-only: what the web launcher
# needs to list and start the native client
NATIVE_ONLY_DISPLAY = ('__init__.py', 'modes.py', 'dpy_native.py',
                       'snd_native.py', 'snd_off.py')


def native_only_display(work):
    """A copy of display/ with only the native client."""
    dst = os.path.join(work, 'display-native-only')
    shutil.rmtree(dst, ignore_errors=True)
    os.makedirs(dst)
    for name in NATIVE_ONLY_DISPLAY:
        shutil.copy2(os.path.join(ROOT, 'display', name), dst)
    with open(os.path.join(dst, 'native-client-only'), 'w') as f:
        f.write('only the native client: see display/modes.py\n')
    return dst


def main(argv=sys.argv[1:]):
    native_only = '--native-client-only' in argv
    # generate the coloured images first, so that the server does not
    # have to do it each time it starts
    subprocess.check_call([sys.executable,
                           os.path.join(ROOT, 'bubbob', 'images', 'buildcolors.py')],
                          cwd=os.path.join(ROOT, 'bubbob', 'images'))
    work = os.path.join(HERE, 'build')
    dist = os.path.join(HERE, 'dist')
    os.makedirs(work, exist_ok=True)
    # bubbob/ and common/: the game; http2/, display/, java/, metaserver/:
    # the web launcher pages (--web), which start the clients
    folders = {d: os.path.join(ROOT, d) for d in
               ('bubbob', 'common', 'http2', 'display', 'java', 'metaserver')}
    if native_only:
        folders['display'] = native_only_display(work)
        del folders['java']
        print('only the native client')
    data = []
    for d, path in folders.items():
        data += ['--add-data', '%s%s%s' % (path, SEP, d)]
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
