#!/usr/bin/env bash
# Build the Futhark example. Clones Rune at the pinned rev (depth 1) if the
# checkout is missing, then runs Rune's own web-build.sh into dist/.
#
# Needs: rustup toolchain with the wasm32-unknown-unknown target, python3,
# and wasm-bindgen on PATH matching the version in Cargo.lock
# (https://github.com/rustwasm/wasm-bindgen/releases).
#
#   ./build.sh          # release, wasm-opt -Oz if installed
#   ./build.sh --dev    # fast edit builds
set -euo pipefail
cd "$(dirname "$0")"
RUNE_REV="781e2b6d87cd5f32d6f4365956fad4baaf24ee50"
if [[ ! -d rune ]]; then
    git init -q rune
    git -C rune remote add origin https://github.com/anomalyco/rune
    git -C rune fetch --depth 1 origin "$RUNE_REV"
    git -C rune checkout -q FETCH_HEAD
fi
./rune/scripts/web-build.sh futhark --out dist "$@"
cp index.html dist/index.html
