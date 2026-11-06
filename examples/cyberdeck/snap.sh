#!/usr/bin/env bash
# Native headless snapshots of the deck, Rune's agent-checking tooling:
# renders the running app to PNG with a virtual clock, plus the element tree.
# Uses software Vulkan (lavapipe) when no GPU adapter is present.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH" RUSTUP_TOOLCHAIN=stable
icd=$(ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -1 || true)
[[ -n "${icd:-}" ]] && export VK_ICD_FILENAMES="$icd"
cargo build --release
out=${1:-/tmp/cyberdeck-snaps}
mkdir -p "$out"
# target-dir is shared with the Futhark example (same Rune rev, same cache)
bin=../futhark/target/release/cyberdeck
CYBER_HERO=1 "$bin" --snap "$out/hero-1280x800.png" --size 1280x800 --scale 1 --at 2.6
"$bin" --snap "$out/boot-1280x800.png" --size 1280x800 --scale 1 --at 1.05
CYBER_DEMO=1 "$bin" --snap "$out/calc-1280x800.png" --size 1280x800 --scale 1 --at 2.2
CYBER_HERO=1 "$bin" --snap "$out/hero-390x844.png" --size 390x844 --scale 2 --at 2.6
CYBER_DEMO=1 "$bin" --snap "$out/calc-390x844.png" --size 390x844 --scale 2 --at 2.2
CYBER_DEMO=1 "$bin" --snap "$out/calc-landscape-844x390.png" --size 844x390 --scale 2 --at 2.2
CYBER_ICE=1 "$bin" --snap "$out/ice-1280x800.png" --size 1280x800 --scale 1 --at 2.45
CYBER_ICE=1 "$bin" --snap "$out/ice-390x844.png" --size 390x844 --scale 2 --at 2.45
"$bin" --tree "$out/tree.json" --size 390x844 --at 1.0
echo "snaps + tree.json in $out"
