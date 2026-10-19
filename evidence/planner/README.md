# K8 host evidence

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
