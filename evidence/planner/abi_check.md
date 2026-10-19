# Host ABI check

Host: aarch64 Linux. Fork 12356f7, built with AHEM b2d7e40; pinned dependency 8832419.

`nm -D libllm.so` exports:

```
ahem_append_forced_skeleton
ahem_ban_next_token
ahem_ban_token
ahem_clear_banned_tokens
ahem_clear_forced_skeleton
ahem_gate_callback_call_count
ahem_get_decode_step_index
ahem_get_last_sampled_token
ahem_get_last_sampled_token_text
ahem_is_gate_callback_registered
ahem_is_grammar_complete
ahem_llm_create
ahem_llm_destroy
ahem_llm_response
ahem_llm_response_buf
ahem_rewind_kv_cache
ahem_rewind_state
ahem_set_forced_skeleton
ahem_set_grammar_complete
ahem_set_retry_sampling
ahem_set_route_literals
clear_ahem_gate_callback
get_lfm_latent_pointer
register_ahem_gate_callback
release_lfm_latent_pointer
```

The pinned mnn_ffi.rs declarations match the C++ definitions for existing symbols (callback float pointer, int rows/cols, void userdata; int status; opaque handles). No signature mismatch found. However, 8832419 has no grammar.rs or grammar_gate.rs and does not declare response_buf or forced skeleton controls. The crate therefore keeps a path dependency without mnn-ffi (which would pull latent/TensorLogic and static linkage) and supplies a candidate-only DFA and dynamically resolved ABI in session.rs. This permits fail-closed startup on missing symbols. Checked C++ definitions in transformers/llm/engine/src/llm.cpp. No AHEM files modified.

Native create overrides temperature to 0.3 and top_k to 40; greedy configuration cannot be enforced through this ABI. Native retry exhaustion may probe configured legal literals by logit; every probe still crosses the gate. No reset C export: the adapter resolves the exported `_ZN3MNN11Transformer3Llm5resetEv` method (void, this pointer only) for fresh KV before each decision, retaining one session. This additional build-specific ABI is required at startup.

## Architecture discrepancy

Despite the task description, `file` and `readelf -h` identify both supplied host libraries as **x86-64** (ELF e_machine 62). Host uname, Rust and C++ toolchains are **aarch64**. Native linker rejected them as incompatible. The fork source is rebuilt read-only into src/fusion-planner/target/mnn-host; results identify original and rebuilt hashes separately. The source working tree has pre-existing modifications (register files timestamp August 26); no source files are changed by this task.

The pinned dependency also fails to compile with default features because os_bridge imports TensorLogic unconditionally. The crate enables the existing tensorlogic feature only; native mnn-ffi linkage remains disabled so missing symbols can be reported dynamically. The adapter additionally resolves tokenizer_encode(std::string const&) via its exact Itanium ABI symbol and uses a C++ bridge to count native input tokens (limit 2048); std::vector never crosses the Rust FFI.

The native split build also needs express/libMNN_Express.so (verified with readelf DT_NEEDED/RUNPATH). Its SHA-256 is included in measured/expected hashes and execution evidence in addition to the two requested libraries.
