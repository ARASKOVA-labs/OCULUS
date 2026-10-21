#!/usr/bin/env bash
# Build the OCULUS .deb and .AppImage for the machine's architecture.
#
#   scripts/release_linux.sh
#
# Needs (Ubuntu 22.04): libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
# libtesseract-dev libleptonica-dev pkg-config build-essential, plus Rust, node, bun and `bun install`
# done in araskova-doccraft/. Output: dist-release/OCULUS_<version>_<arch>.deb and .AppImage (+ .sha256).
# The .deb depends on the system Tesseract (libtesseract4); the AppImage carries its own libraries.
# Language data is installed next to the app (<prefix>/lib/OCULUS/tessdata).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(python3 -c "import json;print(json.load(open('src-tauri/tauri.conf.json'))['version'])")"
echo "==> Building OCULUS $VERSION ($(uname -m))"

for f in eng.traineddata mal.traineddata osd.traineddata pdf.ttf; do
  [ -f "tessdata/$f" ] || { echo "missing tessdata/$f" >&2; exit 1; }
done

# Keep the build machine's folder names (including the user name) out of the shipped binary.
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$ROOT=oculus --remap-path-prefix=$HOME=~"
export CFLAGS="${CFLAGS:-} -ffile-prefix-map=$ROOT=oculus"
export CXXFLAGS="${CXXFLAGS:-} -ffile-prefix-map=$ROOT=oculus"
# The AppImage tools run without FUSE (hosted runners and containers have none).
export APPIMAGE_EXTRACT_AND_RUN=1
export NO_STRIP=true

(cd src-tauri && ../araskova-doccraft/node_modules/.bin/tauri build --bundles deb,appimage)

BUNDLE="${CARGO_TARGET_DIR:-$ROOT/target}/release/bundle"
OUT="$ROOT/dist-release"
mkdir -p "$OUT"
rm -f "$OUT"/OCULUS_*.deb "$OUT"/OCULUS_*.AppImage

for f in "$BUNDLE"/deb/OCULUS_*.deb "$BUNDLE"/appimage/OCULUS_*.AppImage; do
  [ -f "$f" ] || { echo "expected package under $BUNDLE was not built: $f" >&2; exit 1; }
  cp "$f" "$OUT/"
  (cd "$OUT" && sha256sum "$(basename "$f")" > "$(basename "$f").sha256")
done

echo "==> Done:"
ls -l "$OUT"
