#!/usr/bin/env bash
# Push only the five pinned model assets into app-private storage after install -r.
set -euo pipefail
SERIAL=${ANDROID_SERIAL:-192.168.0.105:5555}
MODEL=${FUSION_MODEL_DIR:-/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m}
ADB=(adb -s "$SERIAL")
"${ADB[@]}" shell run-as app.fusion.wallet mkdir -p files/models/lfm_230m
for name in config.json llm_config.json llm.mnn llm.mnn.weight tokenizer.mtok; do
  "${ADB[@]}" push "$MODEL/$name" "/data/local/tmp/fusion-$name"
  "${ADB[@]}" shell "cat /data/local/tmp/fusion-$name | run-as app.fusion.wallet sh -c 'cat > files/models/lfm_230m/$name'"
  "${ADB[@]}" shell rm "/data/local/tmp/fusion-$name"
done
