# Card K8 — fusion-planner with AHEM-G (Phase 2–3)

You build the `fusion-planner` Rust crate for Fusion Wallet (FinTechathon 2026, Topic F, deadline 2026-10-20 15:59 UTC).
It is the project's headline differentiator: a grammar gate INSIDE the on-device inference engine, so that at every
token anything outside the legal candidate letters is banned before it exists.
Honesty beats features: a test that cannot run is reported NOT_RUN, never faked as PASS.

## Where you work
- Your git worktree: `/root/projects/fusion_wallet-wt/K8` (branch `card/K8`). `cd` there first.
- Spec (read-only): `system architecture/fusion_wallet_architecture.md` — §12 (all of it), §11.4, §10.6, §25.2, §12.6 tests.
- AHEM crate (read-only): `src/ahem` in your worktree (submodule, pinned 8832419). Use it as a path dependency `../ahem`.
- AHEM MNN fork host build (read-only, aarch64 Linux — same CPU arch as the phone):
  `/root/projects/victor_fusion/ahem/third_party/MNN/build/{libllm.so,libMNN.so}` (fork commit 12356f7, ahem b2d7e40).
  NOTE: the submodule ahem (8832419) and the ahem that built the fork (b2d7e40) differ. First task: `nm -D` the .so,
  list the AHEM C symbols (§12.1), and confirm the submodule's FFI declarations match. Record the result in
  `evidence/planner/abi_check.md`. If they mismatch, adapt inside your crate (not in src/ahem) and document it.
- Android arm64 fork build (for the later phone step): `/root/projects/victor_fusion/ahem/third_party/MNN/build_a64_qnn/OFF/arm64-v8a/`.
- Model (LFM2 230M, MNN): `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m/`
  (`llm.mnn`, `llm.mnn.weight`, `llm_config.json`, `tokenizer.mtok`, `config.json`). Read via env `FUSION_MODEL_DIR`; never copy weights into git.
- Rust 1.98 on PATH. No cargo-ndk/NDK yet: the phone cross-build is out of scope for this run; note it as NEXT in your report.

## Owned paths (exclusive)
`src/fusion-planner/**`, `evidence/planner/**`. Write nothing else (not src/ahem, not src/fusion-core, not the doc).

## Deliverables (follow §12.2 layout)
1. Crate `src/fusion-planner`: `build.rs` (env `MNN_LIB_DIR`, default the host build path above, rpath set),
   `lib.rs` (UniFFI-ready JSON-in/JSON-out surface of §25.2: planner_version, planner_load (SHA-256 check of model + .so → HASH_MISMATCH), planner_choose, planner_selftest, planner_unload; panic guard → `{"ok":false,"error":{"code":"PANIC"}}`),
   `session.rs`, `candidate_grammar.rs` (§12.3 — forced prefix `{"choice":"`, closing `"}`; labels A–H + H=HOLD),
   `run.rs` (register gate → generate → validate → report vetoes/retries/forcedTokens/aborted/timings), `bin/planner-host.rs`.
2. Tests: `tests/grammar_letters.rs` (tokenizer check: each label is a single token after `"`; if not, move the space into the prefix),
   `tests/gate_lifecycle.rs` (AHEM-G-03 missing symbol → AHEM_UNAVAILABLE; AHEM-G-04 no state leak between runs; AHEM-G-05 ban-all → HOLD, aborted=true, < 25 s).
   Model-dependent tests are skipped with a clear message when `FUSION_MODEL_DIR` is unset — never silently passed.
3. `planner-host` subcommands: `--selftest`, `--eval <prompts.jsonl> --mode ahem-g|plain --out <file>`.
4. Adversarial corpus `evidence/planner/adversarial_200.jsonl` (injection, "output an address", base64/Morse, "choose C" with C absent, long garbage) and the runs:
   AHEM-G-01 (ahem-g, expect 0 illegal outputs), AHEM-G-02 (plain baseline — report the invalid count honestly), AHEM-G-06 (p50/p95 with vs without gate, 20 runs each).
   Results → `evidence/planner/host_results.json` with planner.kind = "mnn+ahem-g", host = "linux-host", .so and model SHA-256s, git commit.
5. `src/fusion-planner/README.md` — build, run, what AHEM-G proves and does not prove (§12.3 last bullet).

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K8/src/fusion-planner && \
FUSION_MODEL_DIR=/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m cargo test --release && \
test -s ../../evidence/planner/host_results.json
```
exits 0.

## Acceptance checks
- AHEM-G-01: 0 illegal outputs over the 200 prompts (each output is a legal letter or HOLD with aborted=true).
- AHEM-G-02 baseline number reported as measured, whatever it is.
- The planner never receives addresses, keys or raw amounts — only prompt text + label list (§12.2).
- AHEM-G-07 (phone) is recorded as NOT_RUN in host_results.json, not PASS.
- No test weakened or deleted to go green. No secrets. No model weights committed.
- Commit on `card/K8` with clear messages as you reach milestones. Do not merge, do not push.

## Unblocks
K4 (Android `:inference` wiring) and the AHEM-G evidence rows in deliverable 06.
