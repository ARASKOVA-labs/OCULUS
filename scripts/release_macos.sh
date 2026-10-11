#!/usr/bin/env bash
# Build a self-contained OCULUS.app and an installer .dmg for the machine's architecture.
#
#   scripts/release_macos.sh
#
# Needs: Homebrew tesseract + leptonica, Rust, bun, and `bun install` done in araskova-doccraft/.
# Output: dist-release/OCULUS_<version>_<arch>.dmg (+ .sha256)
# Set APPLE_SIGNING_IDENTITY to sign with a Developer ID; the default is an ad-hoc signature.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(python3 -c "import json;print(json.load(open('src-tauri/tauri.conf.json'))['version'])")"
case "$(uname -m)" in
  arm64) ARCH=aarch64 ;;
  x86_64) ARCH=x64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

# Keep the build machine's folder names (including the user name) out of the shipped binary.
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$ROOT=oculus --remap-path-prefix=$HOME=~"
export CFLAGS="${CFLAGS:-} -ffile-prefix-map=$ROOT=oculus"
export CXXFLAGS="${CXXFLAGS:-} -ffile-prefix-map=$ROOT=oculus"

echo "==> Building OCULUS $VERSION ($ARCH)"
(cd src-tauri && ../araskova-doccraft/node_modules/.bin/tauri build --bundles app)

APP="$ROOT/target/release/bundle/macos/OCULUS.app"
[ -d "$APP" ] || { echo "expected $APP was not built" >&2; exit 1; }

echo "==> Relocating Homebrew libraries into the bundle"
python3 scripts/bundle_macos.py --app "$APP"

echo "==> Creating the installer"
OUT="$ROOT/dist-release"
STAGE="$(mktemp -d)"
mkdir -p "$OUT"
cp -R "$APP" "$STAGE/OCULUS.app"
ln -s /Applications "$STAGE/Applications"
DMG="$OUT/OCULUS_${VERSION}_${ARCH}.dmg"
rm -f "$DMG"
hdiutil create -volname "OCULUS" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
(cd "$OUT" && shasum -a 256 "$(basename "$DMG")" > "$(basename "$DMG").sha256")

echo "==> Done: $DMG"
