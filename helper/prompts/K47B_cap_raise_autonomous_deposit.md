# K47B — Raise the per-move cap and let the AI make a profitable deposit by itself (with a hard safety net)

Read `docs/WIN_PLAN.md` (hard invariants), the K47 brief (`helper/prompts/K47_autonomous_profitable_deposit.md`) and
`evidence/app/agent/K47_policy_before_after.json`, `K44_economics.md`, `K47_50_quant_report.json`. K47 decided not to send a
policy tx because at 50 USDC the fresh quote broke even in 17.44 days (limit 14) and the minimum actionable size was 62.28 USDC.
The user (owner) explicitly approved this change on 2026-10-20.

## Authorized change (and nothing else)
Raise ONLY the per-move instant cap (micro max) from 10 USDC to the smallest of {75, 100} USDC whose FRESH K44 quote is
ACTIONABLE (break-even <= 14 days). Hard max (2,500), daily cap (4,000), vault allowlist, delays/tiers, buffer, slippage,
guardian, pause and every other policy field stay exactly as they are. Use only the existing owner 2-of-2 runbook (the one K38
used). Read the policy before and after and write the diff to `evidence/app/agent/K47B_policy_before_after.json`; only that one
field may differ.

## Hard safety net (the live wallet is shown to judges)
1. Save the full current policy and session facts first (`K47B_policy_backup.json`) so it can be restored.
2. configurePolicy pauses the module, ends the session and moves to a new epoch. Complete the documented owner flow to a LIVE
   state: new session started with the existing session key, module resumed. Record every tx as OWNER with honest labels.
3. After resume, prove on-chain that the module is not paused, the session is live and the cap has the new value. Prove the app
   shows the module live (screenshot or adb dump of the live app).
4. HARD STOP: if this is not completely done and verified by 14:15 UTC (check `date -u`), restore the backed-up policy through
   the owner runbook, resume, verify live, and stop. Never leave the module paused or the session ended. Report exactly what happened.
5. Never print private keys. Base Sepolia only. Run the owner flow once at a time; never twice.

## Then the AI's own deposit
With a fresh session and no allocation mandate in the report, run the real app path (Autopilot Run now, or the run helper K38 used,
live mode) so the on-device AI decides. Prove the report has no `allocationMandate`. If it holds, record why and stop; do not force it.
Index the tx (source APP, planner id), add verify_all checks, rebuild `evidence/tx_index.json` (`scripts/build_tx_index.py`).
If a deposit executes, let the AI withdraw it back in a later tick (or document why not) so funds end as Safe cash.

## Checks
`cargo test`, arena still 24/24 stopped and ASR 0 (re-run only if core changed), `python3 scripts/judge/verify_all.py` passes,
Android tests if the app changed. Commit (submodule first, then pointer). Message: policy before/after, every tx hash, break-even,
and whether the AI chose the deposit on its own.
