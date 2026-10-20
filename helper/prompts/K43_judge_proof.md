# CONTINUATION (read first)
This worktree already holds your previous partial K43 edits (uncommitted, in the submodule): ProofScreen.kt rewritten,
new ProofModels.kt, FusionWalletApp.kt, TopHealthAppBar.kt, TreasuryScreen.kt. They do not compile yet. Finish them:
- Add every missing string resource used (`prf_title`, `prf_subtitle`, `prf_contracts`, `prf_contracts_hint`, `prf_receipt_note`,
  `prf_ai_count`, `prf_index_count`, `prf_attacks_stopped`, `prf_arena_link`, `prf_attacks_unavailable`, `prf_arena_refreshing`,
  `prf_index_unavailable`, `prf_index_hint`, `prf_op_asset`, `prf_op_transactions`, `prf_op_risk`, `prf_op_owner`,
  `prf_time_unavailable`, and any others) to `app/src/main/res/values/strings.xml`.
- FusionWalletApp.kt:180 passes `onOpenProof` to a composable that has no such parameter: add the parameter to that composable.
- ProofModels.kt:26 calls `remove` on a type without it: fix the expression.
- Then run the build below until it passes, run the tests, and COMMIT inside the submodule, then commit the pointer here.
  Do not stop before the commit. If you change your mind about the design, still leave a compiling, committed state.

# K43 — Judge proof screen and watch-only mode

Read `docs/WIN_PLAN.md`, `JUDGES.md`, `docs/ux/DASHBOARD_LAYOUT.md`. Android: submodule (commit there, then the pointer).
You own new `ui/proof/*` and the onboarding watch-only path. Do not touch `agent/autopilot/*`, `agent/risk/*`, `advisor/*`.

Judges install the public APK. It has no session key, so it cannot execute. Make that a feature, not a dead end:
1. Onboarding: "Watch the live demo wallet" path (default for judges) that loads the deployed Safe/module read-only, with a
   one-line explanation: "This phone has no keys. You are watching the real wallet the AI runs."
2. Proof screen (reachable from Settings and a Home chip "Verified on-chain"): grouped list of every indexed tx
   (`evidence/tx_index.json` by-root fetch already used by the app) by Topic F operation: Asset management, Transactions,
   Risk control, Owner/setup; each with source badge (AI agent / App / Owner / Script), time, amount, BaseScan link, and a live
   receipt check (status, block) done on tap with the existing pinned RPC client. Header counts: "N AI-executed txs".
   Plus deployed contract addresses with BaseScan links, and "24/24 attacks stopped" linking to ARENA.md on GitHub.
3. Real data only; failures show plainly. Unit tests; build with
   `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`; native libs via `scripts/android_planner/build.sh` + `build_core.sh`).
4. Update `docs/ux/DASHBOARD_LAYOUT.md` and `JUDGES.md` (2-minute path mentions the proof screen).
