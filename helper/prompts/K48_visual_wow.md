# K48 — Visual wow that judges notice (real data only)

Android: submodule `src/Fintech_Wallet_Agent/fusion-core-android` (commit there, then the pointer). Read `docs/ux/UX_PLAN.md`,
`docs/ux/DASHBOARD_LAYOUT.md` and the current Home / AI Agent / Proof screens first. Dark obsidian bento style, existing motion (K29).

Rules: every animated value is a real state or timing from the app. No fake progress, no invented numbers, no remote images.
Respect system "remove animations" (Settings.Global ANIMATOR_DURATION_SCALE == 0 => no motion). Cold start must not get slower.
Keep the AI Agent screen within about two screens.

Build:
1. Live decision pipeline (AI Agent hero + Preview sheet): a 6-step strip Snapshot -> Candidates -> On-device AI -> Policy checks ->
   Signer -> On-chain. Each step lights up when the runtime actually reaches it and shows its real duration (load/inference timings already
   recorded). Dry-run stops at Policy checks with "Preview · no broadcast".
2. Policy checks reveal: checks flip to PASS/FAIL one by one (staggered ~60 ms) and end with a "16/16 passed" summary chip.
3. Execution moment: when a tick's outcome is EXECUTED with a tx hash, the timeline row expands into a success card (pulse, tx hash,
   "Verified on BaseScan" opens the explorer). Same in Proof for AI-executed rows.
4. Live heartbeat: pulsing dot "Live · block N" in the top bar that updates when a newer block is read; amber when data is cached.
5. Security shield badge on Home: "24/24 attacks stopped · ASR 0" from the bundled/fetched arena summary (not hard-coded; read
   evidence/results/latest.json or the numbers the Proof screen already uses), opens Proof.
6. Number count-up for Home balances on refresh (from old to new real value, 400 ms) and an Autopilot countdown ring to next tick.
7. Proof: tapping a tx shows spinner -> checkmark morph with the real receipt block.
Tests: unit tests for pipeline state mapping and reduce-motion handling. `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`
(`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`). Update `docs/ux/DASHBOARD_LAYOUT.md`. Commit.
