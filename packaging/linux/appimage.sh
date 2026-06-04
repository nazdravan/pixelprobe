#!/usr/bin/env bash
# Linux x86_64 AppImage: dist/release/pixelprobe-<version>-linux-x86_64.AppImage
# One file: make it executable (chmod +x) and start it, nothing to install.
# appimagetool is downloaded into .cache/ on the first run, at a fixed release, and checked
# against its SHA-256 below.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

TOOL_VERSION=1.9.1
TOOL_SHA256=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
TOOL_URL=https://github.com/AppImage/appimagetool/releases/download/$TOOL_VERSION/appimagetool-x86_64.AppImage
tool=$ROOT/.cache/appimagetool-$TOOL_VERSION-x86_64.AppImage
if [ ! -x "$tool" ]; then
    mkdir -p "$ROOT/.cache"
    curl -fsSL -o "$tool.part" "$TOOL_URL"
    echo "$TOOL_SHA256  $tool.part" | sha256sum -c -
    chmod +x "$tool.part"
    mv "$tool.part" "$tool"
fi

cargo build --release --locked -p pixelprobe

name=$APP-$VERSION-linux-x86_64
appdir=$ROOT/dist/stage/pixelprobe.AppDir
rm -rf "$appdir" "$OUT/$name.AppImage"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" "$appdir/usr/share/doc/pixelprobe" "$OUT"
install -m 755 target/release/pixelprobe "$appdir/usr/bin/pixelprobe"
cp packaging/linux/pixelprobe.desktop "$appdir/usr/share/applications/"
cp assets/icon.png "$appdir/usr/share/icons/hicolor/256x256/apps/pixelprobe.png"
cp README.md LICENSE-MIT LICENSE-APACHE "$appdir/usr/share/doc/pixelprobe/"
# appimagetool wants the .desktop file and the icon in the AppDir root too.
ln -s usr/share/applications/pixelprobe.desktop "$appdir/pixelprobe.desktop"
ln -s usr/share/icons/hicolor/256x256/apps/pixelprobe.png "$appdir/pixelprobe.png"
ln -s pixelprobe.png "$appdir/.DirIcon"
ln -s usr/bin/pixelprobe "$appdir/AppRun"

# No FUSE (containers, CI): appimagetool unpacks itself and runs from a folder.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION=$VERSION \
    "$tool" --no-appstream "$appdir" "$OUT/$name.AppImage"
echo "done: $OUT/$name.AppImage"
