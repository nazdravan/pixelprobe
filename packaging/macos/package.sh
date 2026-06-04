#!/usr/bin/env bash
# macOS app: dist/release/pixelprobe-<version>-macos-universal.dmg
# pixelprobe.app, universal (Apple Silicon + Intel). Runs on a Mac (or the GitHub Actions
# macOS runner) with the Xcode Command Line Tools.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

# Oldest supported macOS (Rust needs 10.12 on Intel and 11 on Apple Silicon).
export MACOSX_DEPLOYMENT_TARGET=10.13
targets=(aarch64-apple-darwin x86_64-apple-darwin)
if command -v rustup >/dev/null; then
    rustup target add "${targets[@]}"
fi
for t in "${targets[@]}"; do
    cargo build --release --locked -p pixelprobe --target "$t"
done

name=$APP-$VERSION-macos-universal
stage=$ROOT/dist/stage/$name
app=$stage/pixelprobe.app
rm -rf "$stage" "$OUT/$name.dmg"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$OUT"
lipo -create -output "$app/Contents/MacOS/pixelprobe" \
    target/aarch64-apple-darwin/release/pixelprobe target/x86_64-apple-darwin/release/pixelprobe
sed -e "s/@VERSION@/$VERSION/g" -e "s/@MIN_MACOS@/$MACOSX_DEPLOYMENT_TARGET/g" \
    packaging/macos/Info.plist >"$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
cp assets/pixelprobe.icns "$app/Contents/Resources/pixelprobe.icns"
cp LICENSE-MIT LICENSE-APACHE "$app/Contents/Resources/"

# Ad-hoc signature: without one the app does not even start on Apple Silicon. It is not a
# Developer ID signature, so Gatekeeper still asks on first start (see the README).
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"
lipo -info "$app/Contents/MacOS/pixelprobe"

# Disk image: the app next to a link to Applications, so installing is one drag.
ln -s /Applications "$stage/Applications"
hdiutil create -volname "pixelprobe $VERSION" -srcfolder "$stage" -fs HFS+ \
    -format UDZO -imagekey zlib-level=9 -ov "$OUT/$name.dmg"
hdiutil verify "$OUT/$name.dmg"
echo "done: $OUT/$name.dmg"
