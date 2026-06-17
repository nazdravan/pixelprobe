# shellcheck shell=bash
# Shared variables for the packaging scripts (sourced with `. packaging/env.sh`).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP=pixelprobe
# The one place of the version: `version` in [workspace.package] of the root Cargo.toml.
VERSION="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p;}' "$ROOT/Cargo.toml" | head -1)"
OUT="$ROOT/dist/release"
export ROOT APP VERSION OUT

# The cargo-about version packaging/licences.sh generates dist/THIRD-PARTY.txt with (also in CI).
export CARGO_ABOUT_VERSION=0.9.3

# copy_docs DEST [txt]: the licences that go into every package, into the folder DEST. The
# packaging scripts copy them only through this function (packaging/windows/package.ps1 has its
# twin, Copy-Docs). With `txt`, the licence files and NOTICE get a .txt extension (Windows).
copy_docs() {
    local dest=$1 ext=${2:+.txt}
    if [ ! -f "$ROOT/dist/THIRD-PARTY.txt" ]; then
        echo "dist/THIRD-PARTY.txt is missing: run packaging/licences.sh" >&2
        return 1
    fi
    mkdir -p "$dest"
    local f
    for f in LICENSE-MIT LICENSE-APACHE NOTICE; do
        cp "$ROOT/$f" "$dest/$f$ext"
    done
    cp "$ROOT/dist/THIRD-PARTY.txt" "$dest/"
}
