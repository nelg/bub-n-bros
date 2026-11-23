#!/bin/sh
# Build what the Linux distribution packages install.
# Run from the bub-n-bros/ folder:
#     sh packing/linux/build.sh [native] [server-embedded] [play-alone]
# (default: all three).  The three packages, which conflict with each other:
#   native           the game on the system's Python, with all the clients
#                    (Python client and native client without a built-in
#                    server: it plays alone with the Python server)
#   server-embedded  the game server as one PyInstaller executable (its own
#                    Python inside), and the native client
#   play-alone       the native client with that server built in
# Results go to packing/linux/out/.  Used by debian/rules,
# packing/rpm/bubbros.spec and packing/arch/PKGBUILD.txt.
#
# PyInstaller: set PYINSTALLER_PYTHON to a Python that has it; otherwise
# it is installed with pip into packing/linux/out/venv (needs network).
set -e
PYTHON=${PYTHON:-python3}
VARIANTS=${*:-native server-embedded play-alone}
OUT=packing/linux/out
mkdir -p "$OUT"
want() { case " $VARIANTS " in *" $1 "*) return 0;; esac; return 1; }

# the man pages and the player-colour images (all variants)
make -C doc compress
(cd bubbob/images && "$PYTHON" buildcolors.py)

if want native; then
    # the xshm extension (X display driver)
    (cd display && "$PYTHON" setup.py build_ext -i)
    # the documentation of the bonuses (needs ImageMagick; optional)
    if command -v convert >/dev/null 2>&1 || command -v magick >/dev/null 2>&1; then
        (cd bubbob/doc && "$PYTHON" bonus-doc.py) || echo "bonus-doc.py failed, continuing"
    fi
fi

# the game server as one executable
if want server-embedded || want play-alone; then
    if [ -z "$PYINSTALLER_PYTHON" ]; then
        if [ ! -x "$OUT/venv/bin/python" ]; then
            "$PYTHON" -m venv "$OUT/venv"
            "$OUT/venv/bin/python" -m pip install --quiet pyinstaller
        fi
        PYINSTALLER_PYTHON=$OUT/venv/bin/python
    fi
    "$PYINSTALLER_PYTHON" packing/pyinstaller/build_server.py --native-client-only
    "$PYTHON" packing/pyinstaller/smoke_test.py packing/pyinstaller/dist/bubnbros-server
    cp packing/pyinstaller/dist/bubnbros-server "$OUT/bubnbros-server"
fi

# the native client: without a built-in server, then with it (the second
# build only recompiles the client itself)
if want native || want server-embedded; then
    (cd rust-client && env -u BUBNBROS_SERVER_BINARY cargo build --release ${CARGO_FLAGS})
    cp rust-client/target/release/bubnbros-client "$OUT/bubbros-native"
fi
if want play-alone; then
    (cd rust-client && BUBNBROS_SERVER_BINARY="$PWD/../$OUT/bubnbros-server" \
        cargo build --release ${CARGO_FLAGS})
    cp rust-client/target/release/bubnbros-client "$OUT/bubbros-play-alone"
fi
