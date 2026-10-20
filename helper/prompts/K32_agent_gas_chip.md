# K32 — Agent gas chip never shows on Home

Repo: this worktree. Android app: `src/Fintech_Wallet_Agent/fusion-core-android` (git submodule, branch `fusion-v3-integration`; commit inside the submodule).

## Bug
`ui/treasury/HomeCharts.kt` shows `StatusChip("Agent gas: … ETH (~N moves)")` only when `state.agentGas != null`.
On a real device against live Base Sepolia it never appears. `data/ChainWalletRepository.kt:222` wraps
`AgentGasReader.read(...)` in `runCatching{}.getOrNull()`, so some step in `data/AgentGasReader.kt` throws.
Also `WalletStateStore` does not persist `agentGas`, so the cached first paint never has it.

## Do
1. Find the real failure. Reproduce with a JVM unit test or a small script that makes the same JSON-RPC calls
   against https://sepolia.base.org (send a `User-Agent` header, plain urllib gets 403). Module
   `0xA560ec10259657E9bADd297E68286d2838Cd5a8f`; read the session key from the module state the app already reads.
   Suspects: how `rpc.request(...)` result is turned into a string for `quantity()`, `rpc.batch` response shape,
   the final `reader.block(...)` hash check, `require` on the session address.
2. Fix the root cause. Keep it read-only and pinned to the snapshot block. If the balance reads but no fee samples exist,
   still return `AgentGas(balance, null, null)` (chip then shows the balance only). Do not invent numbers.
3. Persist `agentGas` in `WalletStateStore` (with backward-compatible read of old caches).
4. Add unit tests for the failing path. `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle ./gradlew testDebugUnitTest assembleDebug` must pass.
   Native libs: if missing, run `scripts/android_planner/build.sh` and `build_core.sh` from the worktree root.
5. Commit in the submodule, then commit the bumped pointer in this worktree. State in the commit message what the root cause was.
