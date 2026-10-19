#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Some host images expose rustup without a selected toolchain. Use an installed
# stable compiler without changing the user's rustup default.
if ! cargo --version >/dev/null 2>&1; then
  for compiler in /root/.auth_pool/*/.rustup/toolchains/stable-*/bin; do
    if [[ -x "$compiler/cargo" ]]; then export PATH="$compiler:$PATH"; break; fi
  done
fi
export FUSION_MODEL_DIR="${FUSION_MODEL_DIR:-/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m}"
if [[ -z "${MNN_LIB_DIR:-}" ]]; then
  for native in "$PWD/src/fusion-planner/target/mnn-host" /root/projects/fusion_wallet-wt/K8/src/fusion-planner/target/mnn-host; do
    if [[ -f "$native/libllm.so" ]]; then export MNN_LIB_DIR="$native"; break; fi
  done
fi
if [[ -z "${MNN_LIB_DIR:-}" ]]; then
  bash evidence/planner/build_native.sh
  export MNN_LIB_DIR="$PWD/src/fusion-planner/target/mnn-host"
fi
if [[ ! -f src/ahem/Cargo.toml ]]; then git submodule update --init src/ahem; fi
exec python3 arena/run.py
