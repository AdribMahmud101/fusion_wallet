# CONTINUATION
You (codex) already did most of this card in this worktree before a usage limit stopped you: about 59 uncommitted files, evidence/app/agent/K38_*.json, K38_hold_diagnosis.md. A different worker may have touched it since. Review `git status`/`git diff` (root and submodule), keep what is right, finish the remaining steps, run the tests, and COMMIT (submodule first, then pointer). Note: K44 (break-even guard) is now on main: a 10 USDC deposit is UNPROFITABLE_AFTER_GAS (56.96 days); do not fight that guard. If execution needs a larger per-move cap, stop after committing and say so; the supervisor will do the owner policy change.

# K38 (Card A) — The AI manages assets on-chain, for real

Read `docs/WIN_PLAN.md` first. Repo: this worktree. Android app in submodule `src/Fintech_Wallet_Agent/fusion-core-android`
(branch `fusion-v3-integration`; commit there first, then the pointer here).

## Why
Topic F: "AI wallet agent that autonomously executes on-chain operations (asset management, transaction execution, risk control)".
Today every Aave deposit/withdraw in `evidence/tx_index.json` is SCRIPT. The AI has never managed assets. Live Home shows
8,899 USDC idle spare cash, Aave 0.21%, and the AI decides HOLD. That reads as "AI does nothing".

## Do
1. Diagnose why the live tick holds: candidate generation in `src/fusion-core`, policy (per-move cap 10 USDC? vault allowlist?
   buffer? cooldown?), planner choice. Write the finding in `evidence/app/agent/K38_hold_diagnosis.md`.
2. If a rule wrongly prevents a valid deposit, fix it at the root without weakening any safety check (arena must stay 24/24,
   ASR 0). If it is correct policy (e.g. vault not allowlisted), use the owner runbook to make the correct owner-signed policy
   change, recorded as OWNER txs.
3. Through the real app path (MNN/AHEM-G planner on Android, `SessionSignerService`, edge relay lease — see
   `src/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/agent/README.md` and
   `scripts/android_planner/execute_owner_setup.py --run-app --local-host`), have the AI execute on Base Sepolia:
   (a) an Aave waUSDC deposit of ≤10 USDC from spare cash, (b) later a withdraw back to Safe cash.
   The device is the Tecno (`ANDROID_SERIAL=119703846S004282`, reachable via the laptop) or the OnePlus itself, whichever
   the runbook supports; document which.
4. Index both txs with `source` = APP and the planner id, verify receipts, sender = session key, `ActionExecuted` event.
   Extend `evidence/app/agent/` and `scripts/judge/verify_all.py` with checks for them; rebuild `evidence/tx_index.json` via
   `scripts/build_tx_index.py`.
5. Tests pass: `cargo test` (fusion-core), Android `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`
   (`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`; native libs via `scripts/android_planner/build.sh` +
   `build_core.sh` from worktree root), `python3 scripts/judge/verify_all.py`.
6. Commit message: diagnosis, fix, both tx hashes.
Never print or commit private keys. Base Sepolia only.
