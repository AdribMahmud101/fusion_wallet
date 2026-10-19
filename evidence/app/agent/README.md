# K17 in-app agent evidence

Highest reached milestone: **M2** (2026-10-19). M3 is in progress; no APP agent transaction is claimed yet.

## Device and provenance

All device evidence here is **OnePlus 11 (CPH2447), Android 16**, the Android host
of the edge chroot. Native generation runs in `app.fusion.wallet:inference`, not
in Linux. Tecno CL6 remains adb-unauthorized; no Tecno run is claimed.
No guardian or session key was imported on this device for M1/M2.

## M1

`m1.log` / `m1.json`: fixed A/H list returned **A**, AHEM-G ON, AHEM-L OFF,
five forced tokens, zero vetoes, no abort. Cold load 988.37 ms; decode 423.24 ms.
This first proof used the supplied fork libraries. The later live-state run
exposed an existing bounds bug in their latent callback; see the M2 fix below.

## M2

`m2_passport.json` is the exact canonical Rust-produced passport (no trailing
newline). `m2_report.json` contains the live K4 snapshot, core candidates,
model response and pure preview. `m2_summary.json` records timings/provenance.
`m2.png` shows the same decision-card component mounted on the AI Agent screen,
rendered by the debug evidence activity. Its button invokes the real runtime.

At block **48328937** the committed bills were all paid. With no rate scenario
provisioned, the model chose **HOLD**. No transaction was signed or broadcast.
Decode 502.29 ms; cold load 853.77 ms; zero vetoes; five forced tokens; no abort.
DecisionHash `0x0c75c7559ad65337c73606d0f2c49c6f263f2a7d5b411eae476dde5b657a6fdc`
was independently recomputed using `cast keccak` on the exact passport string.
HOLD's zero-amount synthetic candidate is never executable, regardless of preview.

The app uses K4's public repository constructor with public committed bills in
`files/agent/obligations.json`; Rust independently rebuilds its Merkle root.
The native Android-only core adapter supplies the real Safe/module addresses to
core enumeration/preview (the older convenience exports substitute placeholders).
No existing data/, chain/, signer/ or host planner logic was edited.

Android DNS is unavailable on this headless OnePlus host. `rpc_bridge.py` forwards
live read-only RPC to HTTPS `base-sepolia-rpc.publicnode.com`, using a loopback
socket and connection reuse. K4 still reads all state at one block and enforces
its unchanged 30-second/five-block freshness checks. There is no host planner,
snapshot replay, remote inference or model fallback in this relay.

The supplied fork indexed `Info::dim` using `Info::size` (element count).
`mnn_dimension_bounds.patch` fixes rank indexing and guards row-count overflow.
`rebuild_llm.py` stages a source copy, replaces only llm.cpp.o, and relinks the
supplied Android objects with NDK27 libc++; original fork files stay untouched.
`mnn_patch_build.json` records source/patch/library hashes and commands. M2's
patched libllm is **not** byte-identical to M1's supplied binary. The unchanged
Rust grammar and decision-loop sources compile directly into the JNI wrapper.

## Build and rerun

1. `scripts/android_planner/build.sh` (JNI, staged MNN patch, pinned hashes).
2. `scripts/android_planner/build_core.sh` (public core adapters + generated UniFFI).
3. `scripts/android_planner/check.sh` (`testDebugUnitTest assembleDebug`).
4. Install with `adb install -r`; provision model/state using `provision_model.sh`
   and `provision_state.sh`. Models stay in app files; never in APK/git.
5. Use **Run agent now** on AI Agent (dry-run only). The debug evidence activity
   accepts `--es mode m1` or `--es mode dry-run` for reproducible captures.

`build-m1.txt` and `build-m2.txt` both exit 0. M2: 105 tests, zero failures,
one inherited opt-in live-network test skipped (104 passed). APK inspection
matched all shipped fork hashes and found no model/tokenizer assets.
Native libraries are ignored reproducible build artifacts, not committed binaries.
The runtime is manual/foreground; no unattended background-loop claim is made.
