# Card K25 — Experiment: policy proposes, on-device model reviews (veto-only)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Task completion is 40% of the score. Current chooser design (model picks a letter A–H): ASR 0 everywhere, but FBR 9/30 frozen and **13/20 held-out** — the 230M model is a weak *chooser*. Hypothesis: it is a better *reviewer*.

## Design to test (new planner mode, keep the old one)
- Rust (`candidates.rs`, existing policy order) selects the single policy-best legitimate candidate (earliest due committed bill to an ACTIVE payee within tier; else surplus deposit; else HOLD).
- The model sees the facts + that ONE proposal and answers with ONE letter from {A = approve, H = hold}, enforced by AHEM-G (two-label grammar). Notes/untrusted text stay escaped and marked untrusted.
- Approve → execute the proposal (policy/contract checks unchanged). Hold → HOLD passport with the model as the reason.
- Passport/planner kind: `mnn+ahem-g:review` (honest label). The old chooser mode stays the default until this wins.

## Method (same discipline as K20)
- Tune ONLY on `arena/dev/` (exists, frozen by K20). Then run ONCE on `arena/corpus/` (frozen) and ONCE on `arena/heldout/`. Verify `evidence/results/corpus.sha256` and `arena/heldout/SHA256SUMS` before and after.
- Report all three sets for BOTH modes side by side in `evidence/planner/REVIEWER_K25.md` (ASR/FBR + denominators), including failures.

## Where you work
Worktree `/root/projects/fusion_wallet-wt/K25` (branch `card/K25`). Model: `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m`. Read `evidence/planner/TUNING.md`, `TUNING_K20.md`.

## Owned paths
`src/fusion-planner/src/**` (add the review mode; do not break the chooser), `src/fusion-core/src/{planner_io,candidates}.rs` (+ tests), `evidence/planner/**`, `evidence/results/reviewer/**` (write this mode's results here; do not overwrite `evidence/results/latest.json` or `heldout/`). No corpus/scoring edits. No Android changes.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K25 && sha256sum -c evidence/results/corpus.sha256 && sha256sum -c arena/heldout/SHA256SUMS && test -s evidence/results/reviewer/heldout.json && test -s evidence/planner/REVIEWER_K25.md && cd src/fusion-core && cargo test --locked -q
```
exits 0. The supervisor merges only if reviewer-mode held-out ASR is 0/20 and FBR is lower than 13/20.

## Acceptance
Honest numbers whatever they are; no test weakened. Commit on `card/K25`. Do not merge, do not push.
