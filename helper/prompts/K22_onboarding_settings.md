# Card K22 — First-run onboarding stepper + Settings (functional, no dummy UI)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Implement `docs/ux/UX_PLAN.md` (READ IT FULLY FIRST) §3 onboarding and the Settings screen of §4. Every step does real work.

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K22` (branch `card/K22`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside it `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K22 FETCH_HEAD`. Commit inside the submodule.
- **Shared design tokens come from card K23** (`ui/theme/FusionTokens.kt`, components `BentoTile`, `StatusChip`, `AmountText`, `TxLink`, icons). Do the non-UI work first (steps A–C below); then, before building screens, merge K23's token commit: `git fetch /root/projects/fusion_wallet-wt/K23/src/Fintech_Wallet_Agent card/K23 && git merge FETCH_HEAD` (retry every 15 min until `ui/theme/FusionTokens.kt` exists there). Use those names; do not redefine colors.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh`; `./gradlew testDebugUnitTest assembleDebug`. Existing: K17 in-app planner (`inference/`, model provisioning from app files dir), `signer/` (Keystore import activities for guardian + session keys), `data/` (manifest from assets, chain reads).

## Owned paths (inside the submodule)
NEW `ui/onboarding/**`, NEW `ui/settings/**`, NEW `onboarding/**` (non-UI: model downloader, checks), `res/values*/strings*.xml` entries you add, the start-destination logic (first run → onboarding; persisted "completed" flag). Call (do not edit) `inference/**`, `signer/**`, `data/**`.

## Build — non-UI first
A. `ModelDownloader`: download LFM2.5-230M MNN files (Hugging Face `taobao-mnn/LFM2.5-230M-MNN` — check the exact file list; or a URL list in `assets/model_manifest.json` you create with file names, sizes, SHA-256 pinned from the files currently on the device in `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m`), resumable (HTTP Range), Wi-Fi-only option, storage check, progress Flow, SHA-256 verification, into the same app files dir the K17 provisioning uses. JVM tests for hashing/resume logic.
B. `TreasuryChecks`: live checks against the manifest at one block: Safe code exists, threshold == 2, module enabled, Sourcify link present; each → pass/fail + block number.
C. `PolicyReader`: tiers, daily/session limits, reserve buffer, verified vendors (reuse `data/` decoders).

## Then the UI
1. Stepper (6 steps, "Step n of 6", Skip + Back on every step, state survives rotation): Welcome & role (Owner / Finance approver / Auditor view-only) → Network & currency (Base Sepolia TEST; currency from manifest with USDC icon; fallback profile only if the manifest defines it; language EN/中文) → Connect treasury (live checks with ticks) → On-device AI model (download + verify + warm-up self-test decision via K17's planner showing letter + ms; "Skip — monitor only") → Keys (authority table; launch the existing guardian/session import; or View-only) → Policy review (read-only, "changes need both owners") → Finish.
2. **Settings**: Network & deployment (addresses + verification links), On-device model (hash, size, last latency, re-download/verify), Keys (imported or not; remove), Language, Judge mode (every evidence row + the two verify commands), About; each row works.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K22/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug
```
exits 0 and `ui/onboarding/K22_FLOW.md` lists each step → what it really does → its test. Real device run of the model download + verify on the OnePlus (`adb` to `192.168.0.102:5555`, or the local device) recorded in that file.

## Acceptance
- No keys in git/APK/logs; model files not in git/APK. No dummy states. No test weakened. Commit inside the submodule on `card/K22`. Do not merge, do not push.
