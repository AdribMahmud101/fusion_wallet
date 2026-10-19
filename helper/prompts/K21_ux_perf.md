# Card K21 — Fast first paint + product-grade UX (Phase 4, judge-facing)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Innovation & interaction is 30% of the score; judges open this app on a phone. The supervisor tested the merged build on the demo phone (Tecno, 1080×2436) and found:

1. **Cold start > 40 s** on Home ("Loading your accounts…"). Unacceptable.
2. **Safety tab broken**: the three stat tiles render truncated garbage ("0 / Attacks that", "not / Good", "0 / Blocked"); "Blocked Attack Attempts" says "No revert events observed" — reverted transactions emit NO events, so this can never work from logs. Emergency Unwind shows a button that is not wired.
3. **Developer copy instead of product copy**: feed titles are raw enums ("PAY_OBLIGATION 2.000000 USDC"); vendors are "Payee 0xd54c2d42"; amounts show 6 decimals everywhere ("2153.000000 / 4000.000000 USDC"); unexplained chips ("A", lone blue dots).
4. **Bills tab "Bills (0)"** while bills exist on-chain under a newer commitment.

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K21` (branch `card/K21`); Android submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside it `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K21 FETCH_HEAD`. Commit inside the submodule.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh`; `./gradlew testDebugUnitTest assembleDebug` in `fusion-core-android`.
- Data sources: `deployments/84532.json`, `evidence/base-sepolia.json` (E16 INTENTIONAL_REVERT rows = attacks blocked on-chain, with tx hashes), `evidence/results/latest.json` (arena: 24 scenarios, stopping layer each), `evidence/agent/golden.json`, `evidence/app/guardian_pause.json`, payee labels in K2's scripts / `evidence/obligations-84532.json` (ids are keccak of `fusion.k2.payee.acme|payroll|newbank` per `scripts/evidence_actions.py`) and K9's `evidence/agent/` obligations. These are public on GitHub raw (`https://raw.githubusercontent.com/AdribMahmud101/fusion_wallet/main/<path>`), which the app already uses for bills.
- Spec §17 (UX: every screen, states, copy, accessibility).

## Owned paths (inside the submodule)
`.../wallet/ui/**` EXCEPT the AI Agent screen's new "Run agent now" button and `runAgentTick` (card K17 adds those — do not touch `wallet/inference/**`, `wallet/agent/**`), `.../wallet/data/**`, `.../wallet/chain/**`, `res/**`, `assets/**`.

## Fix (commit after each; JVM tests for logic)
1. **Fast first paint (target: real numbers ≤ 3 s after login on Wi-Fi, cached numbers instantly).**
   - Persist the last good `WalletState` to app storage; on launch render it immediately with an "Updated 2 min ago · refreshing" chip, then swap in the live snapshot.
   - Split the snapshot: phase 1 = balances + module state + policy + queue (batch the `eth_call`s into ONE JSON-RPC batch request at one pinned block); render. Phase 2 = event history (incremental `eth_getLogs` from the last cached block, not from deployment each launch) + GitHub evidence/bills; merge when ready. Parallelize with coroutines; per-request timeouts; never block the UI on phase 2.
   - Measure: log `firstPaintMs` and `liveSnapshotMs`; write numbers from the Tecno into `ui/PERF_K21.md` (supervisor will test too).
2. **Safety tab, real and honest**: the blocked-attacks list comes from evidence (E16 revert tx hashes with BaseScan links + what each attack tried) and the arena summary (scenarios stopped / total, with the stopping layer per scenario, and the escaped ones listed honestly as "Known limits"). Fix the three tiles: e.g. "21 / 24 attacks stopped", "0 / 30 attacks succeeded", "Stop: armed" — no truncation at 1080 px (one short label + one number each). Emergency Unwind: if not wired to a signer, replace the button with "Owner action — run from Safe (2 of 2)" explanatory text (no dead buttons).
3. **Product copy & formatting**: action titles in plain language ("Paid bill · Acme Cloud", "Queued payment", "Moved surplus to Aave vault", "Withdrew to cash", "Held — needs a human"); vendor display names from the label map (fallback "Vendor 0x61dd…7f79"); money as `2,153.00 USDC` in lists/tiles (6-dp exact value on a detail view or long-press); every chip has a text label (e.g. "Agent", "Script", "App" provenance; "Instant / Queued / Needs approval" tier). Remove unlabeled dots. EN + 中文 strings.
4. **Bills**: resolve the current on-chain commitment root against every bundled/GitHub obligations file (K2 + K9 + K19 demo files); show bills with status Due / Paid (tx link) / Queued.
5. Sweep all tabs at 1080×2436 for truncation, contrast (≥ 4.5:1), touch targets (≥ 48 dp), and empty/loading/error states.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K21/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && ! grep -rnE '"(PAY_OBLIGATION|DEPOSIT|WITHDRAW)"' app/src/main/res
```
exits 0, and `ui/UX_K21.md` lists every defect above with the change made, plus `ui/PERF_K21.md` with measured first-paint / live-snapshot times.

## Acceptance
- No fabricated numbers; every count comes from chain or evidence files. No Float/Double money. No test weakened.
- Commit inside the submodule on `card/K21`. Do not merge, do not push.
