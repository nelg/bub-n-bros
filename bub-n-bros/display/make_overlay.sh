#!/bin/sh
# Builds overlay.ppm: the client's on-screen help and volume messages, one
# 440x22 row each, in the order of pclient.Overlay.LABELS.
# Needs ImageMagick 7 ("magick").
set -e
cd "$(dirname "$0")"
row() {
    magick -size 440x22 xc:black -font DejaVu-Sans-Mono -pointsize 14 \
        -fill '#ffffa0' -gravity center -annotate +0+0 "$1" "row-$2.png"
}
n=0
for label in \
    "F11: full screen     Esc: leave full screen / quit" \
    "F7 / F8: volume down / up     F9: mute" \
    "Volume 0%" "Volume 20%" "Volume 40%" "Volume 60%" "Volume 80%" \
    "Volume 100%" "Sound muted (F9)" "No sound"
do
    row "$label" $(printf %02d $n)
    n=$((n+1))
done
magick row-*.png -append -depth 8 overlay.ppm
rm -f row-*.png
