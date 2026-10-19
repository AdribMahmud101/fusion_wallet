# Card K4 — Android data layer on REAL Base Sepolia state (Phase 3)

You are the Android data engineer for Fusion Wallet (FinTechathon 2026, Topic F; hard deadline 2026-10-20 15:59 UTC).
Judges must see the app show **real** on-chain state, not fixtures. The app is a real-state monitor + approval/kill-switch UI (spec SL4); agent txs are produced by `agent-runner` (card K9).

## Where you work
- Umbrella worktree: `/root/projects/fusion_wallet-wt/K4` (branch `card/K4`). Android app = submodule `src/Fintech_Wallet_Agent` (own git repo, base branch `fusion-v3-integration`). In your worktree: `git submodule update --init src/Fintech_Wallet_Agent`, then inside the submodule `git checkout -b card/K4`. **Commit Android code inside the submodule on `card/K4`.** Commit non-Android files (none expected) on the umbrella `card/K4`.
- App module: `src/Fintech_Wallet_Agent/fusion-core-android/app`, package `app.fusion.wallet`.
- Build on this machine (aarch64, verified): `source /root/projects/fusion_wallet/scripts/edge_android_env.sh` then `./gradlew assembleDebug` in `fusion-core-android`. See `docs/EDGE_ANDROID_BUILD.md`.
- Live deployment: `deployments/84532.json` (merged). Evidence: `evidence/base-sepolia.json` (30 verified rows). Rust core v4: `src/fusion-core` (UniFFI surface, Appendix B §25 of the spec).
- Spec: `system architecture/fusion_wallet_architecture.md` §5, §7.9 buckets, §10.5 snapshot, §11 Android, §17 UX screens (data each screen needs).

## Owned paths (exclusive; inside the submodule)
`fusion-core-android/app/src/main/java/app/fusion/wallet/data/**`, `.../wallet/chain/**`, `.../wallet/ui/MainViewModel.kt`, `fusion-core-android/app/src/main/assets/deployments/**`, the app's Gradle dependency lines, and the UniFFI Kotlin binding + `jniLibs` for `fusion_core`. Do NOT edit other `ui/**` files (card K5 owns them).

## Do (commit inside the submodule after each)
1. **FIRST, within 20 minutes:** commit `data/WalletState.kt` — immutable Kotlin data classes the UI will render: treasury (asset balance raw units + 6 dp string), buckets (reserve/owed/buffer/surplus, §7.9), vault positions (A waUSDC, B demo), module state (paused?, session active?, policy tiers/limits), queue (id, kind, amount, readyAt, status), payees (address, label, status ACTIVE/COOLING with activeAt), recent agent actions (kind, amount, payee, txHash, decisionHash, block time), and an `EvidenceRow` list. Plus `interface WalletRepository { val state: StateFlow<WalletState>; suspend fun refresh() }`. Money = `BigInteger`/raw strings only, never Float/Double (D12). K5 starts coding against this file the moment it lands — keep it stable afterwards (additive changes only).
2. `chain/` — read-only JSON-RPC client (OkHttp or HttpURLConnection, `https://sepolia.base.org`, configurable) using `eth_call` / `eth_getLogs` / `eth_getBlockByNumber` at ONE block for a consistent snapshot. Prefer the fusion-core v4 UniFFI snapshot/decoder functions if they build for `aarch64-linux-android` (regenerate bindings from `src/fusion-core`); otherwise ABI-encode the few view calls in Kotlin and say so in `data/README.md`.
3. `data/ChainWalletRepository` implementing `WalletRepository` from `assets/deployments/84532.json` (copy of the merged manifest) + live reads; agent actions from module events. `MainViewModel` uses it in release builds; `DemoData` only in debug builds (gate by `BuildConfig.DEBUG`), and every screen fed by demo data must be labelled DEMO.
4. Unit tests (JVM) for ABI decoding + bucket math against known values from `evidence/base-sepolia.json` / `deployments/84532.json`.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K4/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug
```
exits 0, and `data/README.md` lists every on-chain read with the contract + function, and one sample snapshot captured live (block number + values) proving the numbers match `cast call` output.

## Acceptance
- No private keys in the app. Read-only RPC only in this card. No Float/Double money. No test weakened.
- Commit inside the submodule on `card/K4`. Do not merge, do not push.
