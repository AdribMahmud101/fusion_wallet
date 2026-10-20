# Card K30 — Home: consumer actions that really work + clearer tiles

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). An expert review proposed Home changes; the supervisor kept only what is real and honest. **No button may exist that cannot complete its action in this app.** Owner keys are NOT in the app by design (Safe 2-of-2); the app holds the guardian key (Stop, cancel queued actions) and the agent session key.

## Where you work
Umbrella worktree `/root/projects/fusion_wallet-wt/K30` (branch `card/K30`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K30 FETCH_HEAD` (must include merged K29 — check `git log --oneline | grep K29`; if K29 is not merged yet, also `git fetch /root/projects/fusion_wallet-wt/K29/src/Fintech_Wallet_Agent card/K29 && git merge FETCH_HEAD`). From the umbrella worktree root: `bash scripts/android_planner/build.sh && bash scripts/android_planner/build_core.sh`; then `source /root/projects/fusion_wallet/scripts/edge_android_env.sh; ./gradlew testDebugUnitTest assembleDebug`. Read `docs/ux/DASHBOARD_LAYOUT.md` first. Links: `context.startActivitySafe(...)` only.

## Build
1. **Hero action bar** (bottom of the Spare-cash hero), two buttons only:
   - `↓ Receive`: bottom sheet with the Safe address (copy button), a QR code of it (zxing or a small pure-Kotlin QR encoder; no network), "Send USDC on Base Sepolia (test network) only", BaseScan link.
   - `+ Earn`: runs the existing agent **dry run** (`runAgentTick(dryRun = true)`) and shows the decision in plain language ("The AI would move 2,000 USDC of spare cash into Aave · bills stay covered" or "The AI would hold: …"), with its policy checks; no broadcast. Never claim a yield number other than the live Aave APY.
2. **"Waiting for your approval" banner** under the hero, ONLY when the module queue has an item: description ("Pay Acme Hosting 2,500.00 USDC"), "Above the AI's auto-limit · Your bills stay covered" (only if true from buckets), live countdown to readyAt, buttons: `Cancel` (guardian `cancelQueuedAction(digest)` via the existing `:signer` guardian path, confirm dialog, success snackbar with tx link) and `Approve in Safe (2 of 2)` (sheet explaining owners co-sign in the Safe + link). If no guardian key is imported, Cancel explains how to import it.
3. **AI Agent tile on Home**: chip `On-device AI · Private` (keep `mnn+ahem-g@android` on the AI Agent tab); rename the button to `Preview next move`; add an allowance bar "Today's AI allowance: X of Y USDC left · up to Z per move" from live module state/policy (daily used/limit, instant-tier max).
4. **Vault tiles**: show "Available anytime: X USDC" (withdrawable); move shares, exposure cap and addresses into the tile's ⓘ sheet. No Add/Withdraw buttons.
5. **Where it sits** donut: slices `Spare cash` (green, surplus), `For bills` (amber, owed + buffer), `Earning` (sky, vault assets); legend rows "Spare cash  8.9K · 95%" (compact amount + %); total in the center. Footer chip: `Agent gas: X ETH (~N moves)` — session key ETH balance at the snapshot block ÷ median gas cost of its recent txs (from the tx index receipts); hide N if unknown.

## Owned paths
`ui/treasury/**` (Home), `ui/home/**` if present, new `ui/receive/**`, `data/**` (gas runway, queue read if missing), strings. Call (do not change) `signer/**` and `agent/**` public APIs.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K30/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && unzip -l app/build/outputs/apk/debug/app-debug.apk | grep -q libfusion_planner_android.so
```
exits 0; JVM tests for the QR payload, allowance math, gas-runway math, and banner visibility (queue empty → hidden). Update `docs/ux/DASHBOARD_LAYOUT.md` (umbrella, Home section) to match. No dummy buttons, no fabricated numbers, no test weakened. Commit on `card/K30` (both repos). Do not merge, do not push.
