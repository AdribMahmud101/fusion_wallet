# fusion-planner 1.0

The planner accepts prompt text, candidate labels A–G plus mandatory H (HOLD), and a candidate-set digest. The caller builds redacted facts; never pass keys, addresses, raw units, or complete candidate objects. The digest is echoed for the caller to check against its current epoch. Unknown request fields are rejected. The caller must still run fusion-core's independent validator and policy checks.

The candidate-only DFA runs **inside the MNN sampler callback**. Fixed prefix `{"choice":"` and suffix `"}` are forced skeleton tokens; illegal sampled tokens are banned and retried before emission. Every native fallback probe still passes the callback. A strict JSON validator accepts only a legal choice and no additional/duplicate keys. Retry exhaustion returns H with `aborted=true`. Plain mode clears callback, skeleton, bans, and completion; it records each raw attempt before the one validation retry.

AHEM-G proves syntactic confinement to the current legal labels, or an abort mapped to HOLD. It does not prove a good choice. It does not authorize a transaction or validate candidate safety. AHEM-L is OFF. The pinned AHEM dependency lacks grammar modules; the local adapter and ABI differences are documented in [abi_check.md](../../evidence/planner/abi_check.md).

Build on Linux with Rust and C++:

```sh
cd src/fusion-planner
cargo build --release
export FUSION_MODEL_DIR=/path/to/lfm_230m
export MNN_LIB_DIR=/path/to/native/fork/build
cargo test --release -- --nocapture
cargo run --release --bin planner-host -- --selftest
cargo run --release --bin planner-host -- --eval ../../evidence/planner/adversarial_200.jsonl --mode ahem-g --out ../../evidence/planner/manual.jsonl
python3 ../../evidence/planner/run_host.py
```

`build.rs` defaults MNN_LIB_DIR to the supplied host build and sets rpath. **The supplied host libraries are actually x86-64, while this host is aarch64.** Rebuild the read-only fork using `../../evidence/planner/build_native.sh`; the output stays under target/mnn-host. With no explicit override, the library selects that local rebuild when it detects this architecture discrepancy. Evidence records both original and executed library hashes, source commit and source diff hash. Do not claim the rebuilt libraries are byte-identical to the supplied build.

The five Rust functions return JSON strings and contain panic guards. Compile with `--features bindings` to generate UniFFI scaffolding for API 1.0. `planner_load(configPath, expectedHashesJson)` requires a complete object mapping `config.json`, `llm_config.json`, `llm.mnn`, `llm.mnn.weight`, `tokenizer.mtok`, `libllm.so`, and `libMNN.so` to lowercase SHA-256 digests. The host tool measures its local files for evidence; production callers must use a trusted provisioned manifest, not trust-on-first-use hashes.

One mutex serializes load, choose, unload, and all native gate access. One native session is reset before each decision through the pinned exported C++ reset symbol; callbacks and all sampler state are cleared before and after generation. Missing required symbols or failed registration return AHEM_UNAVAILABLE. Timeout marks the session FAILED, requiring unload/reload. The callback checks elapsed time; the synchronous ABI cannot interrupt prefill or a hung native call. A process watchdog remains necessary for a hard wall-clock cancellation guarantee. Native create overrides temperature to 0.3/top_k 40; both evaluation arms use those same settings. Plain is this fork with gate disabled, not an independently built stock MNN runtime.

Model-dependent tests print `NOT_RUN` when FUSION_MODEL_DIR is absent; those early returns are not model evidence. See host_results.json for measured acceptance statuses. TTFT is measured at the first allowed callback in gated mode; plain TTFT is null because this ABI supplies no streaming timing callback. Total timing excludes cold load. Phone, Android version, thermal and network measurements are NOT_RUN; peak RSS is not measured. Phone cross-build and AHEM-G-07 are NEXT (no NDK/cargo-ndk in this task).

No weights or binaries are committed. The local model card `lfm2_230m-README.md` identifies taobao-mnn/LFM2.5-230M-MNN and declares `license: apache-2.0`; the directory name is lfm_230m. File hashes identify the actual evaluated assets. This records local metadata, not an independent audit of upstream licensing.
