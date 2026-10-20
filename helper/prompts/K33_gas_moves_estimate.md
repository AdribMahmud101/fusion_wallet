# K33 — Agent gas chip shows balance but never "(~N moves)"

Repo: this worktree. Android app: `src/Fintech_Wallet_Agent/fusion-core-android` (submodule, branch `fusion-v3-integration`; commit inside it, then commit the bumped pointer here).

## Observed on a real device (K32 build, live Base Sepolia)
Home shows `Agent gas: 0.0176411 ETH` with no `(~N moves)`. So `AgentGasReader.read` returns `estimatedMoves == null`,
meaning zero fee samples survived. K32's `scripts/reproduce_agent_gas.py` probe found 17 samples for the same session
(0xb52cb48ec1cc5ad1171c83f162228e31752f87fa), so the app path differs from the probe.

## Do
1. Find which filter drops every sample in the app: the TxIndex the app actually loads (cached vs GitHub `evidence/tx_index.json`,
   does it contain the agent txs?), receipt batch (chunk size vs RPC batch limit / `RpcReadBudget`), header batch,
   blockHash compare, the 30-day window vs block timestamps, `from == session` compare, `receipts` cache key casing
   (`receipts.remove(... .lowercase())` vs inserted key). Make the probe follow the app's exact path to prove it.
2. Fix the root cause. Honest numbers only: if there really are no samples in 30 days, keep balance-only.
   If the 30-day window is the reason (agent txs older), say so in the commit and consider using the most recent N agent txs
   regardless of age, labelled correctly. Do not invent costs.
3. Unit test the failing path. `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle ./gradlew testDebugUnitTest assembleDebug` must pass.
   Native libs: if missing, run `scripts/android_planner/build.sh` and `build_core.sh` from the worktree root.
4. Commit message states the root cause and the live probe result.
