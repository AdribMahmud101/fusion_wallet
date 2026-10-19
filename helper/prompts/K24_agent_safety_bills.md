# Card K24 — Agent, Safety and Bills screens redesign (functional, no dummy UI)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Implement `docs/ux/UX_PLAN.md` (READ IT FULLY) §4 **Agent**, **Safety**, **Bills** using the merged design system from K23 (`ui/theme/FusionTokens.kt`, `ui/components/` — `BentoTile`, `StatusChip`, `AmountText`, `ProvenanceChip`, `TxLink`, `SkeletonTile`; crypto icons in `res/drawable`). Use them; do not redefine colors or components.

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K24` (branch `card/K24`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside it `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K24 FETCH_HEAD` (includes K17 in-app agent, K21 UX/perf, K23 design system, and the external-link crash fix: ALWAYS open links with `context.startActivitySafe(intent)` from `ui/common/SafeIntents.kt`, never `startActivity`). Commit inside the submodule.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh`; `./gradlew testDebugUnitTest assembleDebug` (shared Gradle daemon).
- Data: `data/` WalletState (module state, policy, queue, payees, bills, agent actions, evidence), K17 `runAgentTick(dryRun)` + decision/passport data, K12 `requestGuardianPause()`, evidence on GitHub raw (`evidence/results/latest.json` arena: 23/24 stopped, stopping layer per scenario, A12 escaped; held-out ASR 0/20; `evidence/base-sepolia.json` E16 revert txs).

## Owned paths (inside the submodule)
`ui/agent/**`, `ui/safety/**`, `ui/bills/**`, their strings in `res/values*/`. Do not touch `ui/theme/**`, `ui/components/**` (request additions in `ui/NEEDS_FROM_K23.md` and build a local private composable meanwhile), `ui/home/**`, `ui/activity/**`, `ui/onboarding/**`, `ui/settings/**`, `inference/**`, `agent/**`, `signer/**`, `data/**`.

## Agent screen
1. "How it decides" strip (collapsible): Facts → candidates A–G + H(old) → on-device model picks a letter (AHEM-G) → policy preview → signer → contract — a real flow diagram (Compose Canvas/Row with connectors), each step labelled.
2. Latest decision card: list the candidates with the chosen one highlighted; each policy check with pass/fail; planner kind (`mnn+ahem-g@android` or edge), load/decode ms; passport hash (copy) + TxLink.
3. Controls: **Run agent now** (dry run default; live behind a confirm dialog) wired to `runAgentTick`; **Stop** wired to `requestGuardianPause`; success → snackbar with tx link; failure → readable error.
4. Daily limit **ring gauge** (used / limit, remaining in text) and the queue with live countdowns.
5. Decision history list (human titles, ProvenanceChip, TxLink).

## Safety screen
1. Score tile: "23 / 24 attacks stopped" and "0 attacks succeeded (frozen 0/30 · held-out 0/20)" — numbers parsed from evidence, never typed.
2. **Five-layer diagram** (AHEM-G → validator → policy → signer → contract) with the count stopped at each layer from `latest.json`.
3. Blocked attacks list: what was tried, stopping layer, evidence link (Base Sepolia revert tx or test name).
4. Known limits: escaped scenarios (A12) stated plainly.
5. Stop + Emergency unwind (unwind = owner 2-of-2 action: explain + link; no dead button). Judge-mode shortcut.

## Bills screen
Segmented control Bills | Vendors. Bill card: vendor name + initial avatar, amount (AmountText), due date, status Due / Queued (countdown) / Paid (TxLink). Vendor card: name, short address, ACTIVE or COOLING with a **countdown ring**, sanctions check result, TxLink. "Add vendor / commit bills" = explains owner 2-of-2 and links to the Safe app; no dead form.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K24/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && ! grep -rn "context.startActivity(" app/src/main/java
```
exits 0, plus `ui/K24_SCREENS.md` mapping every tile → data source → empty/loading/error state. JVM tests for the parsers (arena counts per layer, countdown math, gauge math).

## Acceptance
- Every number from chain or evidence. No Float/Double money. Text on every chip/status (never color alone). Touch targets ≥ 48 dp. No test weakened. Commit inside the submodule on `card/K24`. Do not merge, do not push.
