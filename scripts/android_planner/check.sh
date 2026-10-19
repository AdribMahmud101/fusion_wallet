#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source /root/projects/fusion_wallet/scripts/edge_android_env.sh
cd "$ROOT/src/Fintech_Wallet_Agent/fusion-core-android"
./gradlew testDebugUnitTest assembleDebug --no-configuration-cache --max-workers=2 "$@"
