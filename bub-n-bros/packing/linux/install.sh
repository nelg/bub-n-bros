#!/bin/sh
# Install one of the packages built by build.sh into DESTDIR.
# Run from the bub-n-bros/ folder:
#     sh packing/linux/install.sh native|server-embedded|play-alone DESTDIR
#
# native:           /usr/share/bubbros/ the game, /usr/lib/bubbros/ the xshm
#                   extension; bubbros (web launcher), bubnbros-server,
#                   bubnbros-client (Python client), bubbros-native
# server-embedded:  bubnbros-server (PyInstaller), bubbros (its web
#                   launcher), bubbros-native
# play-alone:       bubbros-native, with the server built in
# Menu entries: "Bub-n-Bros server" (not in play-alone) and
# "Bub-n-Bros play alone".
set -e
VARIANT=${1:?usage: install.sh native|server-embedded|play-alone DESTDIR}
DESTDIR=${2:?usage: install.sh native|server-embedded|play-alone DESTDIR}
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
native)
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
    ln -sf "$DATA/display/Client.py" "$BIN/bubnbros-client"
    ln -sf "$DATA/bubbob/bb.py" "$BIN/bubnbros-server"
    install -m755 "$OUT/bubbros-native" "$BIN/bubbros-native"
    install -m644 doc/Client.py.1.gz "$MAN/bubnbros-client.1.gz"
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
server-embedded)
    install -m755 "$OUT/bubnbros-server" "$BIN/bubnbros-server"
    printf '#!/bin/sh\n# the web launcher of the packaged game server\nexec %s/bin/bubnbros-server --web "$@"\n' \
        "$PREFIX" > "$BIN/bubbros"
    chmod 755 "$BIN/bubbros"
    install -m755 "$OUT/bubbros-native" "$BIN/bubbros-native"
    server_files
    ;;
play-alone)
    install -m755 "$OUT/bubbros-play-alone" "$BIN/bubbros-native"
    ;;
*)
    echo "install.sh: unknown package $VARIANT" >&2
    exit 1
    ;;
esac
install -m644 rust-client/README.md "$DOC/native-client.md"
