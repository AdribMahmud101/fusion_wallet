# Card K15 — Submission docs refresh with all merged evidence (Phase 4)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Judges read the docs and click the hashes. The docs (card K7) were written before most evidence existed. Bring them up to date — every claim backed by a file in the repo.

## Where you work
- Worktree `/root/projects/fusion_wallet-wt/K15` (branch `card/K15`).
- Evidence now merged on main:
  - `evidence/base-sepolia.json` — 30 rows, K2 operational scripts (`source: SCRIPT`), verified keylessly by `scripts/verify_evidence.py`.
  - `evidence/agent/golden.json` + `evidence/agent/passports/` + `evidence/agent/runs/` — **3 autonomous agent txs** broadcast by `agent-runner` with the on-device model under AHEM-G (`planner.kind = mnn+ahem-g`), each `decisionHash` = keccak(passport) found in calldata and the module event; plus a HOLD decision (no tx). `scripts/agent/check_golden.py` verifies them.
  - `evidence/app/guardian_pause.json` + `guardian_pause_app.png` — **in-app kill switch**: guardian pause sent from the Android app's `:signer` process (Keystore-wrapped key) on a real phone, then a 2-of-2 owner resume.
  - `evidence/results/latest.json` + `ARENA.md` — arena: 24 scenarios, 21 stopped; escaped: A12 (x402 overcharge — x402 not built), A17 (proxy implementation swap escapes the code-hash pin — known limit §7.2), A13 not implemented. Corpus: ASR 0/30 with AHEM-G (1/30 plain); FBR 25/30 with AHEM-G (card K14 is improving this — use whatever `latest.json` says at the time you write, and regenerate at the end).
  - `evidence/planner/host_results.json` — AHEM-G 0/200 illegal outputs vs plain 48/200 invalid.
  - `deployments/84532.json` — all addresses, Sourcify verification links.
  - Android: `src/Fintech_Wallet_Agent` (live Base Sepolia monitor, 96+ unit tests), `docs/EDGE_ANDROID_BUILD.md`.

## Owned paths (exclusive)
`docs/submission/**`, root `README.md`.

## Do
1. `docs/submission/gen_evidence.py`: render ALL evidence sources above into `06_EVIDENCE.md` (sections: deployment + verification links; owner/script operations; autonomous agent txs with planner kind + decisionHash + passport link; app-sourced txs; arena table and ASR/FBR with denominators; planner host results). Generated, never hand-typed.
2. Update `01_TECHNICAL.md`, `02_DECK.md` (10 slides), `03_VIDEO_SCRIPT.md` (≤ 5 min; include the in-app Stop → pause tx → owner resume, and the agent golden path), `05_SECURITY_SELF_ASSESSMENT.md` (permission tiers; controls → tests → evidence; **known-risk list including A12, A17, A13, the FBR figure, SCRIPT vs AGENT vs APP provenance**), and `README.md` (one-command verification: `python3 scripts/verify_evidence.py evidence/base-sepolia.json` and `python3 scripts/agent/check_golden.py`; build/test commands for contracts, core, planner, Android).
3. Provenance language must be exact: SCRIPT rows are operator scripts; AGENT rows are agent-runner + on-device model; APP rows come from the phone. Never call a SCRIPT row autonomous.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K15 && python3 docs/submission/gen_evidence.py && grep -q "mnn+ahem-g" docs/submission/06_EVIDENCE.md && grep -q "0xb36b6d22" docs/submission/06_EVIDENCE.md && grep -qi "A17" docs/submission/05_SECURITY_SELF_ASSESSMENT.md && ! grep -rniE "first ever|guaranteed yield|unhackable|bank-grade|regulatory approved|zero risk" docs/submission README.md
```
exits 0.

## Acceptance
- Every number and hash traces to a repo file. No secrets. Commit on `card/K15`. Do not merge, do not push.
