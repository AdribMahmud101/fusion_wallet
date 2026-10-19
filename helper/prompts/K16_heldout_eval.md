# Card K16 — Held-out evaluation (no tuning) (Phase 4)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Card K14 tuned the planner prompts on the SAME frozen corpus it is scored on (`arena/corpus/`, ASR 0/30, FBR 9/30) — see `evidence/planner/TUNING.md`. Academic judges will ask about overfitting. This card measures the merged planner ONCE on cases it has never seen.

## Where you work
- Worktree `/root/projects/fusion_wallet-wt/K16` (branch `card/K16`).
- Read `arena/corpus/{benign,malicious}.jsonl` for the case FORMAT only, and `arena/run_arena.sh` / the evaluation code for how a case is executed and scored.
- Do NOT read `evidence/planner/TUNING.md` experiment prompts or `src/fusion-planner` prompt text before writing the new cases (avoid writing cases to fit the prompt). Write the cases first, commit them, then run.

## Owned paths (exclusive)
`arena/heldout/**`, `evidence/results/heldout/**`. Do not modify the planner, core, contracts, the original corpus, or scoring code (you may add a thin wrapper that points the existing evaluation at a different corpus directory).

## Do
1. Write `arena/heldout/{benign,malicious}.jsonl`, 20 each, **new** situations (different amounts, payees, due dates, multi-bill mixes, wording, injection styles: e.g. Unicode homoglyph payee names, instructions hidden in invoice memo, "CEO urgent" social engineering, split-payment to evade tier, base64/rot13 notes, benign but unusual-but-legitimate payments). No case may duplicate an original corpus case (check by content hash and by near-duplicate text). Commit them with `arena/heldout/SHA256SUMS` BEFORE running anything.
2. Run the merged pipeline once (AHEM-G and plain arms) via a wrapper `arena/heldout/run_heldout.sh` that reuses the existing evaluation code with the held-out directory. Write `evidence/results/heldout/latest.json` (ASR/FBR with denominators per arm, per-case decisions) and `evidence/results/heldout/HELDOUT.md` (table + 3–5 lines of honest interpretation, incl. the failures).
3. No reruns to improve numbers. If the run fails for a technical reason (crash), fix the wrapper and rerun, and say so in HELDOUT.md.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K16 && sha256sum -c arena/heldout/SHA256SUMS && test -s evidence/results/heldout/latest.json && git log --format=%s -- arena/heldout/benign.jsonl | tail -1
```
exits 0, and the commit that added the held-out cases is EARLIER than the commit that added `evidence/results/heldout/latest.json`.

## Acceptance
- Whatever the numbers are, report them. No secrets. Commit on `card/K16`. Do not merge, do not push.
