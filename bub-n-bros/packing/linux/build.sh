#!/bin/sh
# Build everything that a Linux distribution package installs.
# Run from the bub-n-bros/ folder:   sh packing/linux/build.sh
# Used by the Debian (debian/rules) and RPM (packing/rpm/bubbros.spec)
# packages; the Arch PKGBUILD does the same steps itself.
set -e
PYTHON=${PYTHON:-python3}
# the xshm extension (X display driver)
(cd display && "$PYTHON" setup.py build_ext -i)
# the man pages
make -C doc compress
# the player-colour images
(cd bubbob/images && "$PYTHON" buildcolors.py)
# the documentation of the bonuses (needs ImageMagick; optional)
if command -v convert >/dev/null 2>&1 || command -v magick >/dev/null 2>&1; then
    (cd bubbob/doc && "$PYTHON" bonus-doc.py) || echo "bonus-doc.py failed, continuing"
fi
# the native client (without a built-in server: play alone uses the
# installed bubnbros-server)
if [ "${NATIVE_CLIENT:-yes}" = yes ]; then
    (cd rust-client && cargo build --release ${CARGO_FLAGS})
fi
