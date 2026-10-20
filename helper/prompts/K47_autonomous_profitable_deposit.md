# K47 — Truly autonomous, profitable AI deposit (no owner mandate), hard security preserved

Read `docs/WIN_PLAN.md` (hard invariants), `evidence/app/agent/K44_economics.md`, K38's evidence and
`OwnerAllocationMandate.kt`, and `src/contracts/src/KavachiModule.sol` (policy fields and how configurePolicy affects
sessions/epochs). Runs after K38M merged K38 onto main.

## Goal
K38's on-chain deposit/withdraw ran under an owner allocation mandate because a 10 USDC deposit is UNPROFITABLE_AFTER_GAS
(K44: 56.96 days; 41 USDC breaks even in 13.90 days). Make the AI execute a deposit it chose because it pays, without a mandate.

## Do
1. Owner policy change through the existing owner 2-of-2 runbook (`scripts/android_planner/execute_owner_setup.py` or the
   script K38 used): raise ONLY the per-move instant-tier cap from 10 to 50 USDC. Hard max (2,500), daily cap (4,000), vault
   allowlist, delays/tiers, session expiry, guardian and pause stay unchanged. If configurePolicy bumps the policy epoch and
   invalidates the session, rotate/restart the session the documented way (owners), and record every tx as OWNER.
   Verify on-chain that only that field changed (read policy before/after; diff in evidence).
2. Re-quote K44 economics live for 50 USDC; it must be ACTIONABLE. Then run the real app path on the device (Autopilot or Run now,
   live mode, no mandate present) so the on-device AI picks and executes the deposit. Mandate code must NOT be involved (prove:
   report has no allocationMandate). If the planner holds, record why; do not force it.
3. Index the tx (source APP, planner id) with receipt/sender/event checks; add verify_all checks; rebuild tx_index.
4. Honest labels: K38's two txs become "Owner-mandated rehearsal (2-of-2)" in tx_index notes, the proof screen and docs; the new
   one "AI-initiated · break-even N days". Keep the mandate verifier code (expired, bounded) but document it in SECURITY notes.
5. Arena 24/24 ASR 0 both corpora (re-run if core changed), `cargo test`, Android tests
   (`flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`, `ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`), `python3 scripts/judge/verify_all.py` all pass.
6. Commit (submodule first, then pointer). Message: policy before/after, all tx hashes, break-even. Never print keys. Base Sepolia only.
