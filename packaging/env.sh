# shellcheck shell=bash
# Shared variables for the packaging scripts (sourced with `. packaging/env.sh`).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP=pixelprobe
# The one place of the version: `version` in [workspace.package] of the root Cargo.toml.
VERSION="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p;}' "$ROOT/Cargo.toml" | head -1)"
OUT="$ROOT/dist/release"
export ROOT APP VERSION OUT
