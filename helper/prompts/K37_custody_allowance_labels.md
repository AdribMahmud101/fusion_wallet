# K37 — Custody split card, honest allowance labels, cached gas label

Android app: `src/Fintech_Wallet_Agent/fusion-core-android` (submodule `fusion-v3-integration`; commit there, then the pointer).
Style: existing dark bento design (`docs/ux/UX_PLAN.md`, `docs/ux/DASHBOARD_LAYOUT.md`). Real data only, no dummy UI.

1. Custody split card (Safety screen, plus a compact one-line pill on Home near the hero): states that owner keys are not in
   this app; the Safe is 2-of-2; this app can only Receive, preview, and run the limited agent; sending and admin changes need
   the Safe owners. Tapping it opens a short sheet with the real Safe address and threshold read from chain (already in WalletState).
   No Send/Swap buttons.
2. Allowance wording: Home shows "Today's AI allowance: 4,000.00 of 4,000.00 USDC left · up to 10.00 per move". Read
   `src/contracts/src/KavachiModule.sol` and the policy reader to learn what each number really is (daily cap vs per-action
   auto-exec limit vs queue threshold: what happens above 10? queued for owner approval? rejected?). Then label both numbers
   accurately on Home and Safety, e.g. "Per move: up to 10 USDC (auto)" and "Daily cap: 4,000 USDC". Say what happens
   above the per-move limit only if the contract does exactly that. Explain in an info tooltip.
3. Agent gas chip: when the Home state comes from the persisted cache (not the live refresh), suffix "(cached)"; if the live
   gas read fails, keep the cached value with "(cached)" rather than hiding it. Never invent values.
4. Update `docs/ux/DASHBOARD_LAYOUT.md` for these changes.
5. `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle ./gradlew testDebugUnitTest assembleDebug` passes; add unit
   tests for label formatting. Native libs if missing: `scripts/android_planner/build.sh` + `build_core.sh` from worktree root.
