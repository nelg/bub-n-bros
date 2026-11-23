#!/bin/sh
# Install one of the packages built by build.sh into DESTDIR.
# Run from the bub-n-bros/ folder:
#     sh packing/linux/install.sh python|server-portable|portable DESTDIR
#
# python:           /usr/share/bubbros/ the game, /usr/lib/bubbros/ the xshm
#                   extension; bubbros (web launcher), bubnbros-server,
#                   bubbros-client-python (Python client), bubbros-client
#                   (Rust client)
# server-portable:  bubnbros-server (PyInstaller), bubbros (its web
#                   launcher), bubbros-client
# portable:         bubbros-client, with the server built in
# Menu entries: "Bub-n-Bros server" (not in portable) and
# "Bub-n-Bros play alone".
set -e
VARIANT=${1:?usage: install.sh python|server-portable|portable DESTDIR}
DESTDIR=${2:?usage: install.sh python|server-portable|portable DESTDIR}
PREFIX=${PREFIX:-/usr}
PYTHON=${PYTHON:-python3}
OUT=packing/linux/out
BIN=$DESTDIR$PREFIX/bin
DOC=$DESTDIR$PREFIX/share/doc/bubbros
MAN=$DESTDIR$PREFIX/share/man/man1
APPS=$DESTDIR$PREFIX/share/applications

install -d "$BIN" "$DOC" "$MAN" "$APPS" "$DESTDIR$PREFIX/share/pixmaps"
install -m644 http2/data/bob.png "$DESTDIR$PREFIX/share/pixmaps/bubbros.png"
install -m644 CHANGELOG.md INSTALL.txt "$DOC/"
install -m644 packing/linux/bubbros-play-alone.desktop "$APPS/"

server_files() {
    install -m644 packing/linux/bubbros.desktop "$APPS/"
    install -m644 doc/bb.py.1.gz "$MAN/bubnbros-server.1.gz"
    install -m644 doc/BubBob.py.1.gz "$MAN/bubbros.1.gz"
}

case $VARIANT in
python)
    DATA=$PREFIX/share/bubbros
    LIB=$PREFIX/lib/bubbros
    install -d "$DESTDIR$DATA" "$DESTDIR$LIB"
    install -m755 BubBob.py "$DESTDIR$DATA/BubBob.py"
    cp -r bubbob common display java http2 metaserver "$DESTDIR$DATA/"
    # the compiled extension, named after the Python ABI
    for so in display/xshm*.so; do
        [ -e "$so" ] || continue
        name=$(basename "$so")
        rm -f "$DESTDIR$DATA/display/$name"
        install -m755 "$so" "$DESTDIR$LIB/$name"
        ln -s "$LIB/$name" "$DESTDIR$DATA/display/$name"
    done
    chmod 755 "$DESTDIR$DATA/display/Client.py" "$DESTDIR$DATA/bubbob/bb.py"
    ln -sf "$DATA/BubBob.py" "$BIN/bubbros"
    ln -sf "$DATA/display/Client.py" "$BIN/bubbros-client-python"
    ln -sf "$DATA/bubbob/bb.py" "$BIN/bubnbros-server"
    install -m755 "$OUT/bubbros-client" "$BIN/bubbros-client"
    install -m644 doc/Client.py.1.gz "$MAN/bubbros-client-python.1.gz"
    server_files
    # remove what is not needed at run time
    (
        cd "$DESTDIR$DATA"
        find . -type d \( -name build -o -name CVS -o -name __pycache__ \) -prune -exec rm -r {} +
        find . -type f \( -name .cvsignore -o -name Makefile -o -name '*.c' \) -delete
        rm -rf display/windows display/dpy_windows.py display/snd_windows.py display/setup.py
        rm -f bubbob/images/buildcolors.py bubbob/doc/bonus-doc.py bubbob/setup.py \
              bubbob/statesaver.py bubbob/test_statesaver.py
    )
    # byte-compile (NO_COMPILE=1: the package does it at installation)
    [ -n "$NO_COMPILE" ] || "$PYTHON" -m compileall -q -d "$DATA" "$DESTDIR$DATA" >/dev/null
    ;;
server-portable)
    install -m755 "$OUT/bubnbros-server" "$BIN/bubnbros-server"
    printf '#!/bin/sh\n# the web launcher of the packaged game server\nexec %s/bin/bubnbros-server --web "$@"\n' \
        "$PREFIX" > "$BIN/bubbros"
    chmod 755 "$BIN/bubbros"
    install -m755 "$OUT/bubbros-client" "$BIN/bubbros-client"
    server_files
    ;;
portable)
    install -m755 "$OUT/bubbros-client-portable" "$BIN/bubbros-client"
    ;;
*)
    echo "install.sh: unknown package $VARIANT" >&2
    exit 1
    ;;
esac
install -m644 rust-client/README.md "$DOC/rust-client.md"
