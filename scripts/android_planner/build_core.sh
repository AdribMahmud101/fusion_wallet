#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source /root/projects/fusion_wallet/scripts/edge_android_env.sh
if ! rustc --version >/dev/null 2>&1 && [[ -d /root/.auth_pool/antigravity_1/.rustup/toolchains ]]; then
  export RUSTUP_HOME=/root/.auth_pool/antigravity_1/.rustup
fi
export CARGO_TARGET_DIR="$ROOT/src/fusion-core/target"
APP="$ROOT/src/Fintech_Wallet_Agent/fusion-core-android/app"
cargo build --manifest-path "$ROOT/src/fusion-planner/android/core/Cargo.toml" --target aarch64-linux-android --release --locked -j 4
mkdir -p "$APP/src/main/jniLibs/arm64-v8a"
cp "$CARGO_TARGET_DIR/aarch64-linux-android/release/libfusion_android_core.so" "$APP/src/main/jniLibs/arm64-v8a/"
# Bindgen reads ELF metadata, so the host executable can consume the Android .so.
BINDGEN=${UNIFFI_BINDGEN:-/root/projects/fusion_wallet-wt/K3/src/fusion-core/target/release/uniffi-bindgen}
cd "$ROOT/src/fusion-planner/android/core"
"$BINDGEN" generate "$CARGO_TARGET_DIR/aarch64-linux-android/release/libfusion_android_core.so" \
  --metadata-no-deps --crate fusion_android_core --language kotlin --out-dir "$CARGO_TARGET_DIR/generated-kotlin" \
  --config "$ROOT/scripts/android_planner/uniffi.toml" --no-format

mkdir -p "$APP/src/main/java/app/fusion/wallet/inference/core"
cp "$CARGO_TARGET_DIR/generated-kotlin/app/fusion/wallet/inference/core/fusion_android_core.kt" "$APP/src/main/java/app/fusion/wallet/inference/core/"

python3 - "$APP/src/main/java/app/fusion/wallet/inference/core/fusion_android_core.kt" <<'PYFMT'
import pathlib,sys
p=pathlib.Path(sys.argv[1]); p.write_text('\n'.join(line.rstrip() for line in p.read_text().splitlines())+'\n')
PYFMT
