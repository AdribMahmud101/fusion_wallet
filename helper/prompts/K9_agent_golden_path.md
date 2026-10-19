# Card K9 — Autonomous agent golden path on Base Sepolia (Phase 3, HIGH VALUE)

You are the agent-runtime engineer for Fusion Wallet (FinTechathon 2026, Topic F "build an AI wallet agent that autonomously executes on-chain operations"; hard deadline 2026-10-20 15:59 UTC).
This card produces the core proof: **the agent decides, the policy checks, the module executes, and every tx anchors the decision's `decisionHash`.**

## Where you work
- Worktree: `/root/projects/fusion_wallet-wt/K9` (branch `card/K9`). `cd` there first.
- Merged and verified: `src/fusion-core` (v4; `src/bin/agent-runner.rs`, 159 tests), `src/fusion-planner` (AHEM-G gated planner; host binary `planner-host`), `src/contracts` (157 tests).
- Live deployment (read-only, from card K2, not yet merged): `/root/projects/fusion_wallet-wt/K2/deployments/84532.json` — Safe `0x029869910714D9e09b854a2688311AFd42fBb44E`, KavachiModule `0xA560ec10259657E9bADd297E68286d2838Cd5a8f`. Copy it into your worktree as `deployments/84532.json` only if agent-runner needs it at a path; never edit K2's files.
- Spec: `system architecture/fusion_wallet_architecture.md` §5.4 (runtime loop), §6 (tiers), §7.6–7.8 (actions, checks, queue), §10.4–10.8 (candidates, planner I/O, preview, passport), §19.5 (evidence row format).
- Keys: `set -a; . /root/projects/fusion_wallet/.env.local; set +a` → use `SESSION_PRIVATE_KEY` (the agent session key). NEVER print, log or commit a key.

## Owned paths (exclusive)
`src/fusion-core/src/bin/**` (agent-runner fixes), `scripts/agent/**`, `evidence/agent/**`. Do not edit other `src/fusion-core/src/**` files; if a library bug blocks you, write `evidence/agent/BLOCKERS.md` with the failing test and work around it in the binary.

## Do (commit after each)
1. Dry run: `agent-runner` against Base Sepolia in a read-only mode (snapshot → candidates → planner → passport → preview) with NO broadcast. Save the snapshot + passport + preview JSON to `evidence/agent/dryrun/`.
2. Planner: use `planner-host` (mnn + AHEM-G) if it runs here with the model at `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m`; else the `deterministic` planner, labelled honestly in the passport (`planner.kind`).
3. Live: broadcast, in this order, whichever the on-chain policy allows (payee cooling for the K2 payees ends ~11:00 UTC; check `cast` before paying):
   a) an instant-tier PAY of a committed bill to a verified payee,
   b) a DEPOSIT of true surplus into a vault,
   c) a WITHDRAW,
   d) a queued (delayed-tier) PAY — record `readyAt`; execute after the delay or show the cancel path,
   e) a HOLD decision (no tx; passport only, with the reason).
   For each tx: save the passport to `evidence/agent/passports/<txhash>.json` and check on-chain that the emitted `decisionHash` equals the hash of that passport (§7.11, §10.8).
4. `scripts/agent/run_golden.sh` reproduces the whole run from scratch (idempotent where possible), and `evidence/agent/golden.json` lists the rows in §19.5 format with `source = "AGENT"`.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K9 && python3 -c "import json;r=json.load(open('evidence/agent/golden.json'));assert len([x for x in r if x.get('txHash')])>=3;print('ok')"
```
exits 0, and for every row: the receipt status is 1 on https://sepolia.base.org and the on-chain decisionHash equals the passport hash (write `evidence/agent/check.log` showing each comparison).

## Acceptance
- Real tx hashes only. No secrets anywhere. No test weakened. Commit on `card/K9`. Do not merge, do not push.

## REVISION 2 (supervisor) — previous attempt REJECTED
Your previous commit relabeled K2's existing SCRIPT transactions (E7, E8, E10, E11 in `evidence/base-sepolia.json`) as `source = "AGENT"`. That is a false claim and was reverted. Rules now:
- Every AGENT row MUST be a NEW transaction broadcast by `agent-runner` (the Rust binary) during YOUR run, signed with `SESSION_PRIVATE_KEY`. Its tx hash MUST NOT appear in `evidence/base-sepolia.json`. Save the full agent-runner stdout for each tx to `evidence/agent/runs/<txhash>.log`.
- If there is no unpaid committed bill or no surplus left, create fresh state the legitimate way: owners (OWNER + APPROVER keys, 2-of-2 Safe tx via MultiSendCallOnly, exactly as K2's `src/contracts/script/` does) commit a new obligation Merkle root for a bill to an ACTIVE payee and re-start the session. Read K2's scripts to reuse that path; mint Aave test USDC to the Safe via the faucet if needed. Record those owner txs as `source = "OWNER"`.
- Try the real planner first: `src/fusion-planner/target/release/planner-host` (build it if missing) with the model at `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m` → `planner.kind = "mnn+ahem-g"`. Use `deterministic` only if that fails, and record why in `evidence/agent/README.md`.
- The extended DoD below is checked by the supervisor mechanically.

## Definition of done (REVISION 2 — replaces the one above)
```
cd /root/projects/fusion_wallet-wt/K9 && python3 scripts/agent/check_golden.py
```
`check_golden.py` (you write it, read-only, no keys) exits 0 only if: ≥ 3 AGENT tx rows; every AGENT tx hash is absent from `evidence/base-sepolia.json`; every receipt has status 1 on https://sepolia.base.org with `from` == SESSION_ADDRESS (read the address from `deployments/84532.json` roles.session); the module event's decisionHash equals the keccak of the saved passport bytes; and `evidence/agent/runs/<txhash>.log` exists for each.
