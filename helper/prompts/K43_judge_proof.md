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
