# Planner host evidence

Current choice-quality work is described in the K14 section below.

## K8 lifecycle baseline

`host_results.json` is the final acceptance report. `ahem-g_200.jsonl` and `plain_200.jsonl` contain each decision and raw generation attempts; the corresponding logs include native callback/status diagnostics. `latency_*.jsonl` contain 20 sequential requests per mode. `selftest.log` records the lifecycle and ban-all checks. `tests.log` records the Rust suite with the model configured; `no_model_tests.log` demonstrates explicit NOT_RUN skips. The UniFFI-enabled build is recorded in `bindings_build.log`.

Reproduce after building the native fork with `build_native.sh`:

```sh
cd src/fusion-planner
FUSION_MODEL_DIR=/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m cargo test --release
FUSION_MODEL_DIR=/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m python3 ../../evidence/planner/run_host.py
```

The supplied host binaries are x86-64, so execution uses the aarch64 rebuild under the owned crate target directory. `abi_check.md` explains the pinned-submodule adapter, required C++ symbols and third split-build library. No native source or supplied binary is edited. Hashes identify the original binaries and actual executed build separately.

`pilot_placeholder/` preserves the complete first measurement. Its prompt used a literal LETTER placeholder, which the plain model often copied. The final prompt enumerates real legal JSON replies; both modes were rerun after that correction. The pilot is retained to avoid hiding this observation.

An acceptance PASS means the specified check was observed; it does not imply that the model chose a useful action. Aborted generations map to HOLD and are counted explicitly. In plain mode, the reported invalid raw count is before fallback, and first-attempt and final-attempt counts are separate. The baseline uses the same fork with all gate controls cleared; a separately built upstream stock runtime was not evaluated. Latencies are full request costs, including a plain validation retry when needed, rather than isolated sampler overhead. Cold load is measured separately. Phone AHEM-G-07 is NOT_RUN; Android cross-build and ten phone runs are NEXT.

## K14 choice-quality tuning

Experiment 10 measured **ASR 0/30, FBR 9/30** in both arms on the unchanged
frozen corpus. [`TUNING.md`](TUNING.md) records every trial, including all failed
configurations. [`../results/latest.json`](../results/latest.json) is regenerated
by the arena; archived runs retain prompts, raw outputs, preview decisions and
source hashes under [`tuning/`](tuning/). K8 reports above describe that older
revision's lifecycle checks, not current task completion.

The current pipeline uses **two model stages**, without a deterministic policy
prior or a Rust keyword filter:

1. If the core facts include an external note and a non-HOLD candidate, the same
   LFM2.5-230M model reviews the note under AHEM-G with A=continue / H=hold. Its
   four chat examples explain routine work, recipient substitution, commitment
   tampering and successful-payment replay. A model H is the actual raw final
   HOLD. An aborted review stops before any action generation.
2. A review A continues to the action model with the **entire original candidate
   label set**. It is not forwarded as an action A. The action chat has three
   examples and explains the existing due-bill/withdrawal/surplus order. No
   candidate is selected or substituted by Rust. The core validator and preview
   still evaluate the final raw choice.

Passport `planner.kind` remains `mnn+ahem-g`: both stages are MNN decisions with
AHEM-G, and no deterministic candidate prior was added. The plain comparator
runs both stages with the same model/prompts/greedy sampler and clears gate
controls. Legacy prompts without a structured core note and HOLD-only sets use
one action stage. Stage routing, original labels/digest, aborted-review behavior
and shared-budget exhaustion are tested independently of model semantics.

`noteReview` preserves the review's raw attempts, native statistics and timings;
`decisionStage` names the stage that produced the decision. `modelCalls` counts
actual native response invocations, including plain validation retries.
`attempts`/`rawAttempts` describe the final stage; aggregate AHEM counters sum both
stages. `totalMs` includes both stages; `reviewMs`, `actionMs` and `actionTtftMs`
separate their costs when both run. Each generation retains the 2,048-input-token
and 8-output-token caps. Stages share the original request's 25-second budget;
plain mode can retry once per stage. Synchronous prefill still needs an external
watchdog for a hard cancellation guarantee.

Decoding is **greedy**, configured before native sampler construction through the
pinned C++ bridge in `tokenizer_bridge.cpp`. It mirrors the native factory's
synchronous, uncached lifecycle, using exported createLLM/set_config/load methods
and the existing C destroy function. A post-load config change would leave the
sampler's copied settings unchanged. No fork source, native binary or model file
is edited. Load/decision reports expose `sampling.type=greedy`; the native probe
checks the effective loaded config and A–H single-token continuations after the
forced prefix. All 128 candidate-label subsets are checked. Prefix/suffix forcing
and native gate callbacks are unchanged; every original action letter remains
legal and H has no special sampling weight.

The unmodified K13 arena generator still writes a boilerplate limitation saying
"temperature 0.3/top_k 40". That sentence describes trials 1–4, not current
sampling. For trials 5 onward, the per-decision sampling field and effective
native config probe establish greedy decoding. The runner/scoring were not edited.

Facts show due-window eligibility/order, ACTIVE status **in the supplied
registry**, funded cash, amount tier and mandatory route. Approval is a proposal;
queueing still requires waiting. Six-decimal USDC display preserves the one-unit
bill without exposing raw integer amounts. The external note is capped at 200
characters and JSON-quoted, including escaped chat delimiters.

These facts do not establish sanctions clearance, unpaid status, registry
provenance or Merkle verification; those checks/inputs are absent from this
planner boundary. Review remains model-based and the examples were tuned on
this corpus. Zero observed ASR is not an out-of-sample safety proof. Contract
checks and core gaps remain as reported by the arena. Android execution is not
claimed. Remaining false blocks and measured costs are listed in `TUNING.md`.

To archive a fresh experiment before changing code again:

```sh
bash arena/run_arena.sh
python3 evidence/planner/record_tuning.py experiment-name
```

The recorder checks the report's planner/core source hashes before saving the
inputs, raw outputs and production preview decisions.
