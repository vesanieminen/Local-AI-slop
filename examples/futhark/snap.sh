#!/usr/bin/env bash
# Native headless snapshots of the app, Rune's agent-checking tooling:
# renders the running app to PNG with a virtual clock, plus the element tree.
# Uses software Vulkan (lavapipe) when no GPU adapter is present.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH" RUSTUP_TOOLCHAIN=stable
icd=$(ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -1 || true)
[[ -n "${icd:-}" ]] && export VK_ICD_FILENAMES="$icd"
cargo build --release
out=${1:-/tmp/futhark-snaps}
mkdir -p "$out"
bin=./target/release/futhark
FUTHARK_CAST=1 FUTHARK_SEED=7 "$bin" --snap "$out/cast-1280x800.png" --size 1280x800 --scale 1 --at 3.2
FUTHARK_RUNE=9 "$bin" --snap "$out/alphabet-1280x800.png" --size 1280x800 --scale 1 --at 2.0
FUTHARK_TASKS=1 "$bin" --snap "$out/ledger-1280x800.png" --size 1280x800 --scale 1 --at 2.5
FUTHARK_RUNE=23 "$bin" --snap "$out/alphabet-390x844.png" --size 390x844 --scale 2 --at 2.0
FUTHARK_CAST=1 FUTHARK_SEED=3 "$bin" --snap "$out/cast-390x844.png" --size 390x844 --scale 2 --at 3.2
FUTHARK_TASKS=1 "$bin" --snap "$out/ledger-390x844.png" --size 390x844 --scale 2 --at 2.5
"$bin" --tree "$out/tree.json" --size 390x844 --at 1.0
echo "snaps + tree.json in $out"
