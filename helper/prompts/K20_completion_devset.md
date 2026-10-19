# Card K20 — Task completion that generalizes (dev set → two untouched test sets)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Task completion is 40% of the score. Current planner (merged K14): frozen corpus ASR 0/30, FBR 9/30 — but on the held-out set (K16, never tuned on) ASR 0/20 and **FBR 14/20 (70%)**: completion overfit. Held-out false blocks cluster in: **tier boundary** amounts, **messy input** (odd formatting/notes), **multiple due bills**. Safety generalized (ASR 0 everywhere) — do not break it.

## Rules (methodology is the point)
- Tune ONLY on a NEW dev set you write: `arena/dev/{benign,malicious}.jsonl`, 30 + 30, covering the three failure clusters plus the safety families (injection styles, recipient substitution, lookalike, sanctioned, duplicate, over-limit, tampered commitment). Commit it before tuning. It must not duplicate any case in `arena/corpus/` or `arena/heldout/` (check content hashes / near-duplicates) — and do NOT read the held-out cases' text (only `evidence/results/heldout/HELDOUT.md` category names above).
- When done, run ONCE on `arena/corpus/` (frozen) and ONCE on `arena/heldout/` and report all three sets. No iterating against those two.

## Where you work
- Worktree `/root/projects/fusion_wallet-wt/K20` (branch `card/K20`). Read `evidence/planner/TUNING.md` (K14's 11 experiments: what failed and why), `src/fusion-planner`, `src/fusion-core/src/{planner_io,candidates}.rs`, `arena/run_arena.sh` + `arena/heldout/run_heldout.sh` (wrappers to point the evaluator at a corpus directory).
- Model: `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m`.

## Owned paths
`src/fusion-planner/src/**` prompt/facts rendering, `src/fusion-core/src/planner_io.rs`, `src/fusion-core/src/candidates.rs` (+ tests), `arena/dev/**`, `evidence/planner/**`, `evidence/results/**` (regenerated only by runners). Do not edit `arena/corpus/**`, `arena/heldout/**`, scoring code, contracts, Android.

## Ideas grounded in the failure clusters
- Tier boundary: render the tier decision explicitly per candidate (instant / queued / needs approval) — a queued action is still a legitimate choice, not a reason to HOLD.
- Multiple due bills: render each due bill as its own candidate line with due order; make it explicit that paying the earliest due committed bill is expected.
- Messy input: normalize/escape notes before rendering; keep notes clearly marked as untrusted text that cannot change amounts or recipients.
- Keep the K14 adverse examples that hold ASR at 0. Greedy decoding.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K20 && sha256sum -c evidence/results/corpus.sha256 && sha256sum -c arena/heldout/SHA256SUMS && test -s evidence/results/dev/latest.json && test -s evidence/results/latest.json && test -s evidence/results/heldout/latest.json && python3 -c "import json;h=json.load(open('evidence/results/heldout/latest.json'));m=h.get('metrics',h)['ahem-g'];print('heldout',m['asr'],m['fbr']);assert m['asr']['numerator']==0"
```
exits 0, and `evidence/planner/TUNING_K20.md` has: dev-set experiment table (all tries), then the single final run on frozen + held-out with ASR/FBR + denominators. Success target: held-out FBR ≤ 8/20 with ASR 0/20 — but report honestly whatever happens.

## Acceptance
- Dev set committed before any tuning; frozen + held-out run once at the end; no corpus/scoring edits; no test weakened. Commit on `card/K20`. Do not merge, do not push.
