# Card K19 — One-command live demo + one-command judge verification (Phase 4)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Judges must be able to (a) watch the system work end to end, live, and (b) verify everything we claim with one command and no keys. Today the pieces exist separately.

## Where you work
- Worktree `/root/projects/fusion_wallet-wt/K19` (branch `card/K19`).
- Existing: `scripts/verify_evidence.py` (keyless, `evidence/base-sepolia.json`), `scripts/agent/check_golden.py` + `scripts/agent/run_golden.sh` + `scripts/agent/owner_setup.py` (K9: owner 2-of-2 commits a bill; agent-runner with mnn+ahem-g decides and pays; passports), `evidence/app/guardian_pause.json`, `evidence/results/latest.json` (arena), `deployments/84532.json`, `src/contracts/script/` (Safe MultiSend owner ops). Keys: `/root/projects/fusion_wallet/.env.local` (never print/commit).

## Session-key lock (MANDATORY)
Another card also broadcasts with the session key. Every Base Sepolia broadcast by your scripts runs under `flock /root/projects/fusion_wallet/.session.lock …` (owner + agent steps of one demo run inside ONE lock hold).

## Owned paths
`scripts/demo/**`, `scripts/judge/**`, `evidence/demo/**`, `JUDGES.md` (new, repo root). Reuse other scripts by calling them; do not edit them.

## Build
1. `scripts/demo/live_demo.sh` — one command, idempotent, ~2–4 min, narrated output (plain English step lines + BaseScan links):
   1) snapshot: balances, buckets, module state (print block number);
   2) owners (2-of-2) commit a FRESH bill (unique id, small instant-tier amount) to an ACTIVE verified payee;
   3) agent tick with the real planner (mnn+ahem-g): print the candidates, the chosen letter, the passport hash;
   4) the payment tx; wait for the receipt;
   5) verify: receipt status, `from` == session key, decisionHash == keccak(passport) found in calldata/event, the bill is now paid on-chain (`isPaid`), a second attempt to pay it reverts (duplicate pay);
   6) append a row to `evidence/demo/runs.json` (§19.5 format, `source = "AGENT"`, `scenario = "live_demo"`).
   Fail loudly with a clear message if funds/gas are low or the module is paused (and how to fix).
2. `scripts/judge/verify_all.sh` — keyless, read-only: runs `verify_evidence.py`, `check_golden.py`, checks every `evidence/app/*.json` and `evidence/demo/runs.json` tx (status, from, decisionHash where present), checks Sourcify verification links from `deployments/84532.json`, and prints a one-screen PASS/FAIL table with counts. Exit 0 only if all pass.
3. `JUDGES.md` (≤ 1 page): "verify in 2 minutes": prerequisites (python3, `cast` optional), the two commands, what each row proves, provenance (SCRIPT / AGENT / APP).
4. Run `live_demo.sh` once for real; commit its output to `evidence/demo/`.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K19 && bash scripts/judge/verify_all.sh && python3 -c "import json;r=json.load(open('evidence/demo/runs.json'));assert len(r)>=1 and r[-1]['txHash'];print('ok')"
```
exits 0 on a machine with no `.env.local` keys loaded (verify_all must not need them).

## Acceptance
- Real tx hashes only; no relabeling of existing rows; no secrets. Commit on `card/K19`. Do not merge, do not push.
