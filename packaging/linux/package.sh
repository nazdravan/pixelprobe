#!/usr/bin/env bash
# Linux x86_64 archive: dist/release/pixelprobe-<version>-linux-x86_64.tar.gz
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked -p pixelprobe

name=$APP-$VERSION-linux-x86_64
stage=$ROOT/dist/stage/$name
rm -rf "$stage" "$OUT/$name.tar.gz"
mkdir -p "$stage" "$OUT"
install -m 755 target/release/pixelprobe "$stage/pixelprobe"
install -m 644 packaging/linux/pixelprobe.desktop assets/icon.png README.md LICENSE-MIT LICENSE-APACHE "$stage/"
tar -C "$ROOT/dist/stage" -czf "$OUT/$name.tar.gz" "$name"
echo "done: $OUT/$name.tar.gz"
