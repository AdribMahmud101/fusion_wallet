# K17 in-app agent evidence

Highest reached milestone: **M3 + K38 asset-management round trip** (2026-10-20). APP-sourced agent transaction successfully planned by on-device MNN+AHEM-G, signed by the Android Keystore session key in :signer, and verified on Base Sepolia under the canonical session lock.

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


## M3: APP-sourced agent transaction

Milestone M3 was authorized under REVISION 2 (supervisor) and completed on **OnePlus 11 (CPH2447), Android 16**.

### 1. Authorized owner commitment setup
Under the canonical shared session lock (`flock /root/projects/fusion_wallet/.session.lock`), the reviewed 2-of-2 owner commitment setup was executed via `scripts/android_planner/execute_owner_setup.py`:
- 8 USDC instant-tier bill committed for active Acme recipient `0x61dd06ce2f07d3e4f97d097741b0d7fd50f97f79`.
- Safe MultiSend transaction confirmed on Base Sepolia (`status: 1`, `owner_setup_receipt.json`).
- Public bill metadata provisioned to app storage (`files/agent/obligations.json`).

### 2. Session key Keystore envelope
The session private key from `/root/projects/fusion_wallet/.env.local` was imported into the `:signer` process using `import_session.py --local-host` via the transient debug socket. It is encrypted in an Android Keystore AES-GCM envelope (`no_backup/agent/session.enc`, 61 bytes). No private key is in git, APK, logs, Intent extras, or arguments.

### 3. In-app autonomous decision loop
The locked on-device agent tick was executed with `dryRun=false` under the canonical session lock via `run_tick.py --local-host`:
1. **Live State**: K4 repository read live Base Sepolia state; Rust UniFFI adapter built candidates and commitments matching the Merkle root.
2. **On-Device Inference**: `app.fusion.wallet:inference` executed LFM2.5-230M-MNN with AHEM-G ON (`vetoes = 0`, `retries = 0`, `forcedTokens = 10`, cold load 939.5 ms, decode 1059.1 ms). The model autonomously selected **Candidate A** (Pay 8 USDC bill to Acme/owner, kind 3, route INSTANT).
3. **Passport & Preview**: Canonical passport `0x2c9f8199031c9a6a1a03377dab2aab417e60f80236e9535ab6aceecb1c9e953d` was built and passed policy preview.
4. **Keystore Signing in `:signer`**: `app.fusion.wallet:signer` verified contract pins, simulation digest, and state freshness, then signed the module call with the Keystore session key.
5. **Broadcast & Mining**: The signed transaction was broadcast to Base Sepolia through the locked edge relay:
   - **Transaction Hash**: [`0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89`](https://sepolia.basescan.org/tx/0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89)
   - **Status**: `1` (Mined in block `48332176`, gas used `200658`)
   - **From**: `0xb52cb48ec1cc5ad1171c83f162228e31752f87fa` (Session address)
   - **To**: `0xa560ec10259657e9badd297e68286d2838cd5a8f` (FusionModule)
   - **DecisionHash**: `0x2c9f8199031c9a6a1a03377dab2aab417e60f80236e9535ab6aceecb1c9e953d` (matches `cast keccak` of canonical passport; confirmed in calldata and `ActionExecuted` event topic 3).
   - **Asset Transfer**: 8 USDC transferred to `0x61dd06ce2f07d3e4f97d097741b0d7fd50f97f79`.
   - **Obligation**: `isPaid` confirmed on Base Sepolia; module action nonce incremented to 11; reserve decremented.

### 4. Recorded Evidence
- `golden_app.json`: §19.5 schema with `source = "APP"`, `planner.kind = "mnn+ahem-g@android"`, `txHash`, and `decisionHash`.
- `passports/0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89.json`: canonical passport.
- `receipts/0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89.json`: canonical receipt.
- `transactions/0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89.json`: raw transaction object.
- `runs/0x4d09be10bc6c75395cf5910844d9126715ebd15e8f20dbb4a6ab38f0d3be5c89.json`: complete verified report.
- `m3_verification.json`: independent 11-point verification result from `verify_app.py`.
- `m3.png`: device screenshot showing the confirmed transaction card rendered in `AgentControlActivity`.
- `./gradlew testDebugUnitTest assembleDebug` exits 0 (115 tests passed, 0 failures).


## K38: real Android asset management

OnePlus 11 / CPH2447, Android 16, Base Sepolia. See
[K38_hold_diagnosis.md](K38_hold_diagnosis.md) for the live HOLD findings, root
fixes, explicit owner-signed strategy scope and exact transaction hashes.

- APP deposit: 10 USDC, `0x7acca495d3bd0da2e62f92d0f6f1f20ef22c1e4bd8dc985be59c37ad4bd2aaaf`.
- APP withdrawal: 9.999999 USDC, `0xe6f0105b8cd4a941943112f7507efb1dee6adc47e08aa45852223c7c62cc709a`.
- OWNER strategy mandate: `0xb4896d5bd3207eee5d05bc2bd956a23d820691c8d4eb7fc80209b432fc353f7a`.

Both actions were chosen by MNN/AHEM-G in Android, signed in `:signer` with the
existing Keystore session envelope and broadcast through the leased edge relay.
The original successful sends were recovered through receipt-only app journal
reconciliation; no second signature/send or host choice override occurred.
`K38_verification.json` is the independent result. `K38_deposit.png` and
`K38_withdraw.png` show the confirmed device cards. Safe cash is 9399001234 raw
USDC afterward; waUSDC holdings are zero. The round trip intentionally pays gas
under a one-time owner mandate; the ordinary profitability gate remains intact.

Run `python3 scripts/android_planner/verify_allocation.py` and
`python3 scripts/judge/verify_all.py` for keyless verification. Rebuild the public
index with `scripts/build_tx_index.py`. `K38_arena.tar.gz` preserves a separate
fresh arena run at canonical archive paths; prior arena evidence is untouched.
