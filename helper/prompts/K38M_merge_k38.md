# K38M — Merge K38 (AI asset management, verified on-chain) onto current main

Main now has K43, K44 and (once K45R lands) K45. K38 branched before them. K38's work: root branch `card/K38` and submodule commit
in `/root/projects/fusion_wallet-wt/K38/src/Fintech_Wallet_Agent` (HEAD af00bf5-era). Its two on-chain txs are verified:
deposit 0x7acca495…aaaf and withdraw 0xe6f0105b…709a (from session key, to KavachiModule, status 1), plus OWNER ALLOCATION_MANDATE 0xb4896d5b….

1. Submodule: fetch K38's HEAD and merge. Conflicts expected: AgentNetwork.kt, AgentRuntime.kt, SessionSignerService.kt,
   inference/core/fusion_android_core.kt (UniFFI generated: regenerate from the merged Rust, do not hand-merge).
2. Root: `git merge card/K38`. Conflicts expected: scripts/android_planner/build_core.sh, src/fusion-core/src/candidates.rs,
   src/fusion-core/tests/candidates_prop.rs, src/fusion-planner/android/core/src/lib.rs.
3. Keep ALL behaviour: K44 quant guard (utilization, break-even, conservative APY) stays authoritative; K38's live-rate wiring and
   candidate fixes; K45 router/screen; K43 proof screen; K35 signer allowlist. If K38 and K44 disagree on a deposit rule, K44 wins.
4. Re-run `scripts/build_tx_index.py`; the proof screen must count the new APP deposit/withdraw under Asset management.
5. `cargo test` (src/fusion-core), arena still 24/24 with ASR 0 (regenerate via the existing generator only if core changed behaviour),
   Android `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`; first run `scripts/android_planner/build_core.sh` from worktree root), `python3 scripts/judge/verify_all.py`.
6. Commit submodule merge, then root merge. Message lists conflicts and how each was resolved.

## Also merge K37 (custody split, allowance labels, cached gas)
Root branch `card/K37`; submodule commit 64181aa in `/root/projects/fusion_wallet-wt/K37/src/Fintech_Wallet_Agent`.
It overlaps K45's shared allowance formatter: K45's formatter wins (one source of truth for Home/Agent/Safety); keep K37's custody
split card + Home pill and the "(cached)" gas suffix. Build/test as above, commit.
