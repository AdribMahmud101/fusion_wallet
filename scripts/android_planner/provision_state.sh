#!/usr/bin/env bash
# Public committed bill metadata, verified against chain by K4 and Rust.
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SERIAL=${ANDROID_SERIAL:-192.168.0.105:5555}
BILLS=${1:-"$ROOT/evidence/agent/obligations-84532.json"}
python3 - "$BILLS" <<'PY'
import json,sys
v=json.load(open(sys.argv[1])); assert len(v['bills'])>0 and v['root'].startswith('0x')
PY
adb -s "$SERIAL" shell run-as app.fusion.wallet mkdir -p files/agent
adb -s "$SERIAL" push "$BILLS" /data/local/tmp/fusion-obligations.json
adb -s "$SERIAL" shell "cat /data/local/tmp/fusion-obligations.json | run-as app.fusion.wallet sh -c 'cat > files/agent/obligations.json'"
adb -s "$SERIAL" shell rm /data/local/tmp/fusion-obligations.json
