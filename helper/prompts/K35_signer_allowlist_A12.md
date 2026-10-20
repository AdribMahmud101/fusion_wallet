# K35 — Close A12: the session signer must refuse anything that is not a module call

Repo: this worktree. Read `evidence/results/ARENA.md` (row A12) and `evidence/results/core-probes.json` (id A12) first.

## Real cause
A12 is not a prompt. The generic signer signed an arbitrary prepared tx: ERC-20 `transfer(0xeeee…, 99000000)` with
`to = 0xeeee…`. Keyword filters ("x402", "402") would NOT stop this. Do not add keyword matching as the fix.

## Do
1. Find every place a session key signs: `src/fusion-core` (signer / agent-runner), and the Android signer
   (`src/Fintech_Wallet_Agent/fusion-core-android/.../agent/SessionSignerService.kt` and friends).
2. Add a deterministic signing allowlist enforced right before signing, in each signer:
   - `to` must equal the configured KavachiModule address; `value` must be 0; `chainId` must equal the configured chain;
   - the 4-byte selector must be one of the module's session-key entry points (derive from the contract ABI in `src/contracts`, not by hand);
   - optionally the decoded decision must hash to the decision hash being executed.
   Anything else: refuse with a typed error (e.g. `SignerRefused::NotModuleCall`) and never sign.
3. Re-run the A12 probe with the same prepared tx; it must report `stopped: true` with the refusal reason. Regenerate
   `core-probes.json` and the A12 row in `ARENA.md` via the existing generator scripts (do not hand-edit generated files).
   Update the ARENA notes line about x402 honestly: generic arbitrary-calldata signing is now refused; a dedicated x402
   payment/budget flow is still not implemented (keep that statement).
4. Tests: Rust unit tests for allow and each refusal; Android unit tests if you touched Android
   (`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle ./gradlew testDebugUnitTest assembleDebug`; native libs via
   `scripts/android_planner/build.sh` + `build_core.sh` from the worktree root if missing). `cargo test` in `src/fusion-core` passes.
   `python3 scripts/judge/verify_all.py` must still pass (update its expectations only if a check genuinely changed, and say so).
5. The existing in-app agent tx path (module call) must still be signable: prove with a test using the real golden tx calldata
   from `evidence/app/agent/golden_app.json`.
6. Commit (submodule first if touched, then pointer). Message: root cause, fix, before/after A12.
