# K44 — Quant pre-processor: utilization guard, gas-adjusted break-even, conservative APY

Read `docs/WIN_PLAN.md`, `docs/Perception & Ingestion.md` (sections B.3, 1.C, Pro-Tip "Rate Mean-Reversion"), and K38's
commit/diagnosis (`evidence/app/agent/K38_hold_diagnosis.md`). Rust core: `src/fusion-core`; Android submodule as usual.

## Build (deterministic code, before the planner sees candidates)
1. Pool utilization for the Aave USDC reserve on Base Sepolia (real reads: aToken supply, variable+stable debt, or the pool's
   reserve data). Rule: no new deposit when utilization > 85%; withdrawable-now liquidity = available underlying. Expose both.
2. Gas-adjusted break-even per deposit/rebalance candidate:
   `days = (gas_cost_usd + slippage_usd) / (size_usd * (apy_new - apy_current) / 365)`; gas from the K33 median actual fee;
   ETH/USD from Chainlink on Base Sepolia (pin feed address; staleness check; if unavailable, mark UNMEASURED, never guess).
   Tag ACTIONABLE (≤14 days) or UNPROFITABLE_AFTER_GAS. Unprofitable candidates are not offered to the planner as moves.
3. Conservative APY = min(spot, realized APY from Aave liquidityIndex growth over ~24h) using a historical eth_call at a past
   block if the RPC serves it (try publicnode); if not available, use spot and label "spot only".
4. Put all three in the decision report (so Preview, Autopilot timeline and the cloud advisor can show them) and on the Home
   "Interest per year" tile (utilization %, "withdrawable now", break-even days for the next move).
5. Economics honesty: with a 10 USDC per-move cap a deposit may be UNPROFITABLE_AFTER_GAS. If so, report it plainly and add
   to `evidence/app/agent/K44_economics.md` the per-move size at which a deposit becomes ACTIONABLE. Do NOT change on-chain
   policy yourself; the supervisor decides on an owner policy change.
6. Arena must stay 24/24, ASR 0. Tests: Rust + Android unit tests for each formula and edge (zero delta, negative delta,
   stale oracle). `cargo test`; `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`
   (`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`; native libs via build.sh + build_core.sh);
   `python3 scripts/judge/verify_all.py`.
