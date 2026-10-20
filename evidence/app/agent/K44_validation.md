# K44 validation

2026-10-20 UTC. No Base Sepolia transaction or policy write was made by K44.

| Check | Result |
|---|---|
| `cargo test --manifest-path src/fusion-core/Cargo.toml` | 190 passed, zero failures |
| Native Android adapter Rust tests | 7 passed |
| `scripts/android_planner/build.sh` | PASS |
| `scripts/android_planner/build_core.sh` | PASS; Android + host libraries, local bindgen |
| `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` | PASS; 261 tests, zero failures, 4 existing live/device tests skipped |
| New Android quant tests | 8 passed, zero skipped; 5 call production Rust formulas through UniFFI |
| `python3 scripts/judge/verify_all.py` | 48/48 PASS, zero keys |

The four pre-existing skipped tests are LiveEventIndexTest, LiveSnapshotTest,
ModelDownloaderDeviceIT and AgentGasLiveTest. K44's separate read-only probe
captured real pool/index/oracle/receipt/quote data; see K44_live_inputs.json.
The unit tests cover utilization at/beyond 85%, shared available liquidity,
zero liquidity, exact 14-day economics, gas and slippage, zero/negative yield
delta, unknown/stale/future/invalid oracle inputs, spot-only fallback, realized
APY capping, invalid historical data, wrong quote size/vault, and overflow.

Android commit: `5ce1078` (built on K39 `2b09a30`).
APK: `src/Fintech_Wallet_Agent/fusion-core-android/app/build/outputs/apk/debug/app-debug.apk`
SHA-256: `937269b147e16721a81fda08cebc0c42a7a4f291238c47cd68cfad2a152878ce`.

The JVM native-library search path is set before any test initializes UniFFI;
JNA's host natives are test-runtime-only. No host library is packaged in the APK.

Held-out corpus (final diff, one pass per mode): AHEM-G ASR 0/20 and plain
ASR 0/20; FBR 10/20 (50%) in each mode. These are measured false blocks,
not a claim of perfect task completion. Source/runtime hashes and raw outputs
are retained under `evidence/results/heldout/`.

Frozen-corpus arena: **24/24 security scenarios stopped**, zero escaped;
AHEM-G ASR **0/30**, plain ASR **0/30**. FBR **6/30 (20%)** in each mode.
The local Anvil/Foundry suite passed **155 tests**. See
`evidence/results/latest.json` and `evidence/results/ARENA.md` for raw evidence,
coverage limits, binary/model hashes and source hashes. The final held-out
scores above were not used to tune this change.

Final keyless verification was repeated after the new report and arena artifacts:
**48/48 PASS**, zero keys, zero failures.
