# K36R — Port K36's false-block fix onto current main and regenerate the arena once

K36 (root branch `card/K36`, worktree /root/projects/fusion_wallet-wt/K36, commit a75d0ce) fixed real false blocks:
frozen FBR 9/30 -> 3/30, held-out 13/20 -> 9/20, ASR 0. Read its commit message and evidence/results/FBR_DIAGNOSIS.md there.
But K36 branched BEFORE K35 (session signer allowlist, A12 now stopped) and K44 (quant guard). Its regenerated ARENA.md shows A12
as escaped. Do not merge its generated evidence.

1. In this worktree (from current main), port only K36's source changes: src/fusion-core/src/candidates.rs, src/fusion-core/src/preview.rs
   (+ its tests) and any non-generated corpus/script fixes. Resolve against K44's candidates.rs (K44's guard wins on deposits).
2. Copy evidence/results/FBR_DIAGNOSIS.md, re-checked against the merged code.
3. Regenerate the arena ONCE with the existing generators (arena/run.py, --refresh-core-probes as K35 did). Required result:
   24/24 scenarios stopped including A12 (SignerRefused::NotModuleCall), ASR 0/30 and 0/20, and the new FBR numbers. Held-out run once.
4. `cargo test` (src/fusion-core), forge tests if the generator runs them, `python3 scripts/judge/verify_all.py` pass.
   Rebuild host native core (`scripts/android_planner/build_core.sh` from worktree root) and run Android unit tests
   (`flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest` with `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`)
   if core's UniFFI surface changed.
5. Commit with before/after numbers for both corpora and the A12 row.
