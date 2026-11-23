#!/bin/sh
# Install the game built by build.sh into DESTDIR (as a distribution
# package does).  Run from the bub-n-bros/ folder:
#     sh packing/linux/install.sh "$DESTDIR"
# Layout:  /usr/share/bubbros/  the game (Python), /usr/lib/bubbros/  the
# compiled xshm extension, /usr/bin/  bubbros (web launcher),
# bubnbros-server, bubnbros-client (Python client), bubbros-native.
set -e
DESTDIR=${1:?usage: install.sh DESTDIR}
PREFIX=${PREFIX:-/usr}
PYTHON=${PYTHON:-python3}
DATA=$PREFIX/share/bubbros
LIB=$PREFIX/lib/bubbros

install -d "$DESTDIR$PREFIX/bin" "$DESTDIR$DATA" "$DESTDIR$LIB" \
           "$DESTDIR$PREFIX/share/man/man1" "$DESTDIR$PREFIX/share/doc/bubbros" \
           "$DESTDIR$PREFIX/share/applications" "$DESTDIR$PREFIX/share/pixmaps"
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

ln -sf "$DATA/BubBob.py" "$DESTDIR$PREFIX/bin/bubbros"
ln -sf "$DATA/display/Client.py" "$DESTDIR$PREFIX/bin/bubnbros-client"
ln -sf "$DATA/bubbob/bb.py" "$DESTDIR$PREFIX/bin/bubnbros-server"
if [ -x rust-client/target/release/bubnbros-client ]; then
    install -m755 rust-client/target/release/bubnbros-client "$DESTDIR$PREFIX/bin/bubbros-native"
    install -m644 packing/linux/bubbros-play-alone.desktop "$DESTDIR$PREFIX/share/applications/"
    install -m644 rust-client/README.md "$DESTDIR$PREFIX/share/doc/bubbros/native-client.md"
fi

install -m644 doc/bb.py.1.gz "$DESTDIR$PREFIX/share/man/man1/bubnbros-server.1.gz"
install -m644 doc/Client.py.1.gz "$DESTDIR$PREFIX/share/man/man1/bubnbros-client.1.gz"
install -m644 doc/BubBob.py.1.gz "$DESTDIR$PREFIX/share/man/man1/bubbros.1.gz"
install -m644 packing/linux/bubbros.desktop "$DESTDIR$PREFIX/share/applications/"
install -m644 http2/data/bob.png "$DESTDIR$PREFIX/share/pixmaps/bubbros.png"
install -m644 CHANGELOG.md INSTALL.txt "$DESTDIR$PREFIX/share/doc/bubbros/"

# remove what is not needed at run time
cd "$DESTDIR$DATA"
find . -type d \( -name build -o -name CVS -o -name __pycache__ \) -prune -exec rm -r {} +
find . -type f \( -name .cvsignore -o -name Makefile -o -name '*.c' \) -delete
rm -rf display/windows display/dpy_windows.py display/snd_windows.py display/setup.py
rm -f bubbob/images/buildcolors.py bubbob/doc/bonus-doc.py bubbob/setup.py \
      bubbob/statesaver.py bubbob/test_statesaver.py
# byte-compile (NO_COMPILE=1: the package does it at installation)
[ -n "$NO_COMPILE" ] || "$PYTHON" -m compileall -q -d "$DATA" "$DESTDIR$DATA" >/dev/null
