# K47C — Retry the on-device AI's own deposit on the newest app build

Context: K47B raised the per-move cap to 75 USDC on-chain (OWNER tx 0xea19c153…939f3, only microMax changed) and the module/session
are live. K47B's live AI tick HELD with an empty candidate set because that run used an older app build whose live refresh was too slow
(snapshot "stale", no K44 quant section). Main now has the fix (block-lag limit 15) and a Preview on a phone shows
"The AI would move 75.00 USDC into Aave waUSDC · ACTIONABLE · break-even ~11.7 days".

Do (read `evidence/app/agent/K47B_autopilot_hold.md` and the K38/K47B run helper first):
1. Build the CURRENT main app (this worktree is from current main): native libs via `scripts/android_planner/build.sh` + `build_core.sh`, then
   `flock /root/.gradle/fusion-build.lock ./gradlew assembleDebug` (`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`).
   Install it on the OnePlus edge device that holds the session key, the same way K38/K47B did. Do not change on-chain policy.
2. Run ONE live tick through the real app path (no allocation mandate; confirm the report has no `allocationMandate`). If the AI deposits,
   record the APP tx (receipt status 1, from the session key, to KavachiModule, `ActionExecuted`), index it (`scripts/build_tx_index.py`),
   add verify_all checks, and let the AI withdraw it back in a later tick (or document why not).
3. If it holds again, capture WHY (report reason, quant section present or missing, snapshot age and block lag) in
   `evidence/app/agent/K47C_hold_diagnosis.md`. Do not force a deposit and do not retry more than 3 times.
4. HARD STOP at 14:30 UTC (check `date -u`). Never leave the module paused or the session ended; if anything goes wrong, say so.
5. Checks: `python3 scripts/judge/verify_all.py` passes; commit (submodule first only if you changed it, then the pointer).
   Message: the report fields that prove it was the AI's own choice, every tx hash, break-even.
Never print private keys. Base Sepolia only.
