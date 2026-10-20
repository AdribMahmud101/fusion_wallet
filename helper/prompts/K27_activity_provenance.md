# Card K27 — Activity: correct provenance, amount and counterparty for every tx

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). On the demo phone the Activity tab shows the in-app agent payment `0x4d09be10…5c89` (the model decided INSIDE the Android app and the Keystore session key signed it — our strongest proof) as **"Script"**, with "Amount not recorded" and "Counterparty not recorded". Cause: `ui/activity/ActivityModels.kt` only reads provenance/amount/recipient from `evidence/base-sepolia.json` (SCRIPT rows); module events for agent/app/demo txs have no source and fall back to `Provenance.Script`.

## Where you work
Umbrella worktree `/root/projects/fusion_wallet-wt/K27` (branch `card/K27`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K27 FETCH_HEAD`. From your umbrella worktree root: `bash scripts/android_planner/build.sh && bash scripts/android_planner/build_core.sh` (installs the verified native libs into YOUR submodule), then `source /root/projects/fusion_wallet/scripts/edge_android_env.sh; ./gradlew testDebugUnitTest assembleDebug`.

## Build
1. `scripts/build_tx_index.py` (umbrella): derive ONE file `evidence/tx_index.json` from the evidence files — never typed by hand: `evidence/base-sepolia.json` (source as recorded, SCRIPT/OWNER), `evidence/agent/golden.json` (AGENT, planner kind), `evidence/demo/runs.json` (AGENT), `evidence/app/agent/golden_app.json` (APP, planner `mnn+ahem-g@android`), `evidence/app/guardian_pause.json` (APP pause, OWNER resume). Per tx: `{txHash, source, kind, amountRaw (string, if recorded), recipient (address, if recorded), payeeLabel (if resolvable from obligations/by-root or payee ids), planner (if any), evidenceFile}`. Where a row lacks amount/recipient, read them from the on-chain receipt logs (module Paid/Executed events) via public RPC once at index build time and record them. Add the file to `scripts/judge/verify_all.py`? NO — keep the verifier unchanged.
2. App: load `tx_index.json` from GitHub raw (`https://raw.githubusercontent.com/AdribMahmud101/fusion_wallet/main/evidence/tx_index.json`, cached like the other evidence) and use it in `ActivityModels.kt` for provenance, amount, counterparty name and planner chip, BEFORE falling back to `Script`. Unknown txs must show "Source not recorded" — never default to Script for a tx the index does not know. Same lookup for the Home agent tile's "last decision" and the Agent screen's history if they show provenance.
3. JVM tests: index parsing; provenance precedence; a fixture containing `0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89` resolves to App with its amount and payee.

## Owned paths
Umbrella: `scripts/build_tx_index.py`, `evidence/tx_index.json`. Submodule: `ui/activity/**`, `data/**` (index loader only), `ui/home/**` / `ui/treasury/**` and `ui/agent/**` only for the provenance lookup.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K27 && python3 scripts/build_tx_index.py && python3 -c "import json;d=json.load(open('evidence/tx_index.json'));r={x['txHash'].lower():x for x in d};x=r['0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89'];assert x['source']=='APP' and x.get('amountRaw');print(x)" && cd src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug
```
exits 0.

## Acceptance
Every label derived from evidence or chain; no relabeling beyond what the evidence files record. No test weakened. Commit on `card/K27` (both repos). Do not merge, do not push.
