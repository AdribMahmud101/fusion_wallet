# CONTINUATION
A previous worker started this merge in this worktree and stopped (quota). Inspect `git status` in the submodule:
finish resolving any conflicts (keep both K44 and K45 behaviour), then do steps 2-5. Do not stop before committing.

# K45R — Merge K45 onto main (which now has K44) and wire the seams

Android submodule `src/Fintech_Wallet_Agent` (branch `fusion-v3-integration`). This worktree is from main, which includes K44
(quant pre-processor). K45 (event router + AI Agent screen cleanup) was built before K44 merged. Its submodule commit is
`c39a178` in `/root/projects/fusion_wallet-wt/K45/src/Fintech_Wallet_Agent` (fetch it from there).

1. In the submodule: `git fetch /root/projects/fusion_wallet-wt/K45/src/Fintech_Wallet_Agent HEAD` and merge FETCH_HEAD.
   Conflicts expected in `agent/AgentRuntime.kt`, `agent/autopilot/AutopilotModels.kt`, `data/WalletState.kt`. Keep BOTH
   features: K44's quant report/economics and K45's wake paths, gas defer, display formatting, compact screen.
2. Wire the seams K45 left: the profitable-idle-cash cognitive trigger must use K44's measured ACTIONABLE break-even; HOLD reasons
   on the timeline and Home tile must show K44's economics when that is why it held (e.g. "10 USDC breaks even in 57 days").
3. Read `/root/projects/fusion_wallet-wt/K45` main-repo commit too (docs changes) and bring them over.
4. `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`; native libs via `scripts/android_planner/build.sh` + `build_core.sh` from worktree root);
   `cargo test` in `src/fusion-core`; `python3 scripts/judge/verify_all.py`. All pass.
5. Commit the merge in the submodule, then the pointer here.
