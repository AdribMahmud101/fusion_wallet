# Android planner and core boundary

The JNI library compiles `../src/run.rs` and `../src/candidate_grammar.rs` directly.
Those host files are unchanged. `src/session.rs` adapts only the dynamic C++ symbol
names to Android libc++ (`__ndk1`). The existing tokenizer C++ bridge is also reused.

`PlannerService` loads this library exclusively in `app.fusion.wallet:inference`.
A single native mutex plus a single service executor serializes all model/gate
access. Generation clears callbacks/skeleton/bans and resets the context each time.
Only AHEM-G is accepted by the Android entry point; there is no silent fallback.
A 60-second process watchdog covers a hung synchronous native load/prefill/decode.
The existing decode callback enforces its separate 25-second generation budget.

`core/` statically links the umbrella `src/fusion-core` and exposes a small generated
UniFFI adapter (`libfusion_android_core.so`). Enumeration and preview take the real
Safe/module addresses, and enumeration receives verified bills/payees/proofs.
The older convenience exports substitute placeholder addresses, so they are not
used for transaction binding. All policy, encoding, passport and signing operations
call public fusion-core APIs; its source is unchanged.

`verify_for_signing` recomputes canonical passport bytes/hash, candidate-set digest,
candidate/action binding, strict model output and original snapshot binding. It
rejects HOLD, dry-run, stale results and non-instant routes, then runs an independent
fresh core preview. The Android signer additionally verifies actual RPC code pins,
EIP-1967 implementation pins, live authority, eth_call simulation, fee/gas bounds,
receipt/transaction bytes and the matching ActionExecuted event.

Build from the umbrella:

```sh
scripts/android_planner/build.sh
scripts/android_planner/build_core.sh
scripts/android_planner/check.sh
```

The supplied Android fork has an out-of-bounds tensor-dimension access.
`rebuild_llm.py` stages its `llm.cpp`, applies the committed rank/overflow fix, and
relinks the supplied Android objects. Original fork files stay unchanged. This is
a patched/hybrid build, not byte-identical to the supplied library. Source/patch/
library hashes and exact commands are in `evidence/app/agent/mnn_patch_build.json`.
The provided MNN libraries require Android API 34+; this card tested Android 16 on
OnePlus CPH2447. Native libraries and all model assets remain uncommitted artifacts.

`ProvisionedHashes.kt` is generated from the trusted build/provision sources. The
APK pins all model/tokenizer/config and shipped fork hashes before native loading.
Changing the model requires a reviewed rebuild, not trust-on-first-use on a phone.

For phones without platform DNS, the optional loopback RPC bridge forwards live
HTTPS requests. It has no model/planner/key-handling code. A write-capable bridge
holds `/root/projects/fusion_wallet/.session.lock` with `fcntl.flock`, triggers the
app, and keeps the lock through receipt validation. The app and bridge both reject
broadcasts without that lease. Inherited descriptors preserve the same kernel lock
when an explicitly approved owner setup is followed by an app tick.

The session import is debug-only in `:signer`. Its secure activity accepts manual
entry or a transient root/shell-only local socket. Keys never enter Intent/Binder
extras, command arguments, plaintext files or logs. Android Keystore AES-GCM wraps
the session key separately from the guardian. Ethereum signing is software signing;
processes still share Android UID/storage. This is not separate-APK isolation or
hardware Ethereum signing. JVM/FFI copies cannot all be reliably zeroized.

The UI button is a foreground dry-run. Live app ticks are driven only by the edge
lock helper. The demo signer supports instantaneous execution; queued/owner routes
remain previews. An unresolved signed journal blocks new nonce allocation. There
is no automatic fee replacement or claim of the complete production signer protocol.
