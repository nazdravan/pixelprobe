#!/usr/bin/env bash
# Package for Ubuntu / Debian / Mint: dist/release/pixelprobe-<version>-linux-x86_64.deb
# Install with `sudo apt install ./pixelprobe-*.deb`; it appears in the menu. Needs dpkg-deb.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked -p pixelprobe

name=$APP-$VERSION-linux-x86_64
pkgroot=$ROOT/dist/stage/deb
rm -rf "$pkgroot" "$OUT/$name.deb"
mkdir -p "$pkgroot/DEBIAN" "$pkgroot/usr/bin" "$pkgroot/usr/share/applications" \
    "$pkgroot/usr/share/icons/hicolor/256x256/apps" "$pkgroot/usr/share/doc/pixelprobe" "$OUT"
install -m 755 target/release/pixelprobe "$pkgroot/usr/bin/pixelprobe"
install -m 644 packaging/linux/pixelprobe.desktop "$pkgroot/usr/share/applications/pixelprobe.desktop"
install -m 644 assets/icon.png "$pkgroot/usr/share/icons/hicolor/256x256/apps/pixelprobe.png"
install -m 644 README.md "$pkgroot/usr/share/doc/pixelprobe/README.md"
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: pixelprobe"
    echo "Source: https://github.com/nazdravan/pixelprobe"
    echo
    echo "Files: *"
    echo "License: MIT or Apache-2.0"
    echo " On Debian systems, the Apache 2.0 text is in /usr/share/common-licenses/Apache-2.0."
    echo " The MIT text follows."
    echo " ."
    sed 's/^/ /; s/^ $/ ./' LICENSE-MIT
} >"$pkgroot/usr/share/doc/pixelprobe/copyright"

# Debian versions use ~ for pre-releases: 0.2.0-rc.1 -> 0.2.0~rc.1.
debver=${VERSION/-/\~}
size=$(du -sk "$pkgroot/usr" | cut -f1)
cat >"$pkgroot/DEBIAN/control" <<CONTROL
Package: pixelprobe
Version: $debver
Architecture: amd64
Maintainer: nazdravan <nazdravan@users.noreply.github.com>
Installed-Size: $size
Depends: libc6 (>= 2.35), libgl1, libxkbcommon0
Recommends: libxkbcommon-x11-0, libwayland-client0, libx11-6, libxcursor1, libxrandr2, libxi6
Section: utils
Priority: optional
Homepage: https://github.com/nazdravan/pixelprobe
Description: monitor test utility
 Shows what each monitor reports (size, pixels per inch, refresh rate, display
 modes), checks it with full-screen test patterns (SMPTE colour bars, solid
 colours, gradients, level patches, grid, checkerboards) and a motion test, and
 runs pixel refresh noise over stuck pixels.
CONTROL

dpkg-deb --root-owner-group -Zxz --build "$pkgroot" "$OUT/$name.deb"
echo "done: $OUT/$name.deb"
