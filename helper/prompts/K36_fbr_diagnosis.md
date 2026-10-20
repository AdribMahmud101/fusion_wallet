# K36 — Diagnose and reduce the false block rate honestly

Repo: this worktree. Read `evidence/results/ARENA.md`, `evidence/results/latest.json`, `src/fusion-core/tests/arena_pipeline.rs`,
the corpus files they load, and the held-out results (FBR 13/20).

## Key fact
`ahem-g` and `plain` planner modes have IDENTICAL FBR (9/30 frozen). So the false blocks are almost certainly not the model's
choice; they come from core validation / preview / fixtures. A "fall back to the deterministic planner" patch is therefore
not the default answer. Prove the cause first.

## Do
1. For every benign case blocked (frozen 9, held-out 13), record: case id, chosen candidate, which rule/check rejected it,
   and whether that block is (a) correct safety, (b) a genuine false positive in a rule, (c) a fixture/corpus label error,
   or (d) the model picking a worse candidate. Write this table to `evidence/results/FBR_DIAGNOSIS.md`.
2. Fix only (b) at the root, without weakening any attack check. Fix (c) only with a written justification per case.
   Only if (d) is material, add a fallback: when the model's pick fails preview on a case with no attack signal, re-select
   the best policy-valid candidate deterministically. Report BOTH numbers: model-only and system (with fallback).
3. Discipline: diagnose and fix using the FROZEN corpus only. Run held-out exactly once at the end and report it as is.
   Do not edit held-out cases or labels.
4. ASR must stay 0/30 and 0/20; all 23 previously stopped arena scenarios must stay stopped. Re-run with the existing
   generators and regenerate `latest.json` and `ARENA.md` (no hand edits). Keep the previous numbers visible as "before".
5. `cargo test` passes; `python3 scripts/judge/verify_all.py` passes (update expectations only for genuinely changed checks, say so).
6. If you change `src/fusion-core`, keep committed historical evidence untouched; new runs go to the regenerated files.
7. Commit with before/after numbers per corpus in the message.
