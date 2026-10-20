# K35 session signing boundary

A12 supplied a prepared ERC-20 transfer for 99,000,000 raw units to
`0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee`. The generic EIP-1559 signer
previously signed it without checking module authority. No prompt or keyword
matching was involved.

Session signing now requires separate trusted deployment configuration. The Rust
signer checks the destination, zero native value, configured chain, and the
`execute` / `queueAction` selectors directly derived from
`src/contracts/src/IKavachiModule.sol`. It also decodes and re-encodes the entire
ABI call to refuse malformed or noncanonical calldata before constructing a
signing identity. Refusals carry the `SignerRefused` type.

The audited session paths are `src/fusion-core/src/sign.rs`, its UniFFI export,
`src/fusion-core/src/bin/agent-runner.rs`, the Android native adapter in
`src/fusion-planner/android/core`, and Android `SessionSignerEngine` /
`SessionSignerService`. Android checks before key access and repeats the policy
at the native signing boundary. Its deployment authority comes from the bundled
manifest, independently of the prepared transaction. The runner's separate
guardian path permits only a zero-value module `pause` call. The legacy
`executor.rs` wallet/provider handles EOA transfers through `execute_intent`;
it is not used by the Kavachi session runner or Android session service.

The real app calldata is resolved through
`evidence/app/agent/golden_app.json` by the Rust and native Android tests. The
Android JVM fixture is generated from that same transaction by
`scripts/android_planner/generate_session_allowlist.py`, which also derives the
Kotlin selector allowlist from the contract ABI. `--check` verifies both outputs.
The tests use a public deterministic test key, without broadcasting.

Regenerate the core probes and ARENA report with:

```sh
python3 arena/run.py --refresh-core-probes
```

This reruns the existing production core-probe generator and ARENA renderer,
records source hashes and refresh time, and retains the earlier planner and
on-chain measurements. A12's prepared transaction is identical to the previous
one. Its result changes from `stopped: false` and a signed hash to `stopped: true`,
`refusal_reason: SignerRefused::NotModuleCall`, and `signed_hash: null`.
A dedicated x402 payment/budget flow is still not implemented.


Validation on this worktree:

- `cargo test --manifest-path src/fusion-core/Cargo.toml`: 183 tests passed.
- `CARGO_TARGET_DIR="$PWD/src/fusion-core/target" cargo test --manifest-path src/fusion-planner/android/core/Cargo.toml`: 7 native adapter tests passed.
- `bash scripts/android_planner/build.sh` and `bash scripts/android_planner/build_core.sh`: succeeded; regenerated native Kotlin bindings include the separate policy argument.
- From `src/Fintech_Wallet_Agent/fusion-core-android`, `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle ./gradlew testDebugUnitTest assembleDebug`: 230 tests passed, 3 skipped, debug APK assembled.
- `python3 scripts/android_planner/generate_session_allowlist.py --check`: passed.
- `python3 arena/run.py --refresh-core-probes`: A12 refused, original prepared transaction unchanged; generated ARENA summary is 24 stopped, 0 escaped.
- `python3 scripts/judge/verify_all.py`: 48/48 passed; expectations unchanged.
