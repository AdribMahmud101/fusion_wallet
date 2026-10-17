# fusion_wallet — Project Context & Handoff

> **Living document.** Any agent working on this project: read this first, and append to the *Progress log* (bottom) + update the *Status* and *TODO* tables whenever something is done, decided, or found. Detailed schedule/rationale: `PLAN.md`. Full spec: `Fusion_Wallet_Final_Base_Architecture.md` (+ handoff zip).
> Last updated: **2026-10-17 19:15 (Asia/Dhaka)**

## 1. What this is
**fusion_wallet** (renamed from "Kavachi"): an Android AI treasury assistant for a small business, Topic F "Agent Wallet" hackathon entry. It reserves cash for owner-approved obligations, and only sweeps true surplus into two demo ERC-4626 vaults. A small **on-device LLM** picks one pre-computed candidate action ID (or HOLD) — it never invents amounts/addresses/keys. Rust previews, a separate **signer** app re-checks and signs, and a **Solidity wallet** enforces hard limits on-chain even if everything else is bypassed. Test assets only (DemoUSD, Anvil + Base Sepolia). No mainnet, no real funds.

**Deadline: 2026-10-20 21:59 Asia/Dhaka (23:59 Beijing). Artifact freeze target: 2026-10-20 15:59 Dhaka.**

Five required features: (1) three buckets free/reserved/deployed, (2) live attack arena, (3) three amount tiers (≤10 immediate, ≤100 delayed 180 s with on-chain cancel, >100 owner-approved), (4) decoded approval cards, (5) false-block-rate (FBR) / attack-success-rate (ASR) evaluation.

## 2. Where everything is
| What | Location |
|---|---|
| Project dir (docs) | `C:\Users\UseR\Documents\fusion_wallet\` — `CONTEXT.md`, `PLAN.md`, spec `.md`, review `.docx`, handoff `.zip` |
| Code repos (cloned, branch `fusion-v3-integration`, **nothing committed or pushed yet**) | `...\fusion_wallet\repos\` → `Fintech_Wallet_Agent` (integration/product repo, pinned `e5ae118a`), `mnn-android-sdk` (`5c8f61f4`), `ahem` (`88324195`). Upstream: github.com/AdribMahmud101/<name> |
| Model (pinned HF `taobao-mnn/LFM2.5-230M-MNN` @ `216a2236272d566697c9f417d11d42c852b2f014`, Apache-2.0, 132 MB) | `D:\Android\models\LFM2.5-230M-MNN\` (+ `SHA256SUMS`); **patched config** `D:\Android\models\llm_config.patched.json` |
| Android SDK / NDKs / adb | `D:\Android\Sdk` (platforms 36+34, build-tools 36.0.0+34.0.0, NDK 27.0.12077973 + 25.1.8937393, CMake 3.22.1, platform-tools 37.0.1) |
| Rust / Foundry | **WSL Ubuntu** (user `tory`): Rust 1.94.0 (+`aarch64-linux-android`, rustfmt, clippy), cargo-ndk 4.1.2, Foundry 1.8.5 (forge/anvil/cast), gcc/clang. Linux NDK r27 at `/mnt/d/Android/ndk-linux/android-ndk-r27` (`ANDROID_NDK_HOME` in `~/.bashrc`) |
| Agent memory | `C:\Users\UseR\.claude\projects\C--Users-UseR-Documents-fusion-wallet\memory\` (Claude Code only; not portable — everything important is repeated here) |
| Session scratch (temporary, may vanish) | `...\Temp\claude\...\scratchpad\` — has `backup/` of the *original "Kavachi"-named* docs and a Python venv used to run the handoff's verification scripts |

## 3. Environment (this Windows PC)
- Windows user env vars set: `JAVA_HOME` (Temurin 17.0.20), `ANDROID_HOME`/`ANDROID_SDK_ROOT=D:\Android\Sdk`, `GRADLE_USER_HOME=D:\Android\gradle-home`, PATH has JDK/platform-tools/cmdline-tools. **Open a fresh terminal** to pick them up.
- Docker Desktop was **uninstalled** (user request; freed ~83 GB). Disk: C: ~108 GB free, **D: ~11 GB free** (Android SDK + Gradle cache live on D: — watch it).
- Hardware: Ryzen 7 7700, 15 GB RAM. WSL capped at ~7 GB RAM. Virtualization on.
- GitHub: `gh` is logged in as `AdribMahmud101`; all 3 repos are public and heads matched the pinned commits at clone time.

### Test device
Xiaomi **Redmi 8**, arm64-v8a, Android 10 (API 29), Snapdragon 439 (msm8937), **3.6 GB RAM (~1.0–1.3 GB free)**, MIUI 12.5, 12 GB free storage. Chosen *deliberately* as a low-end device to prove responsiveness. USB debugging authorised; **"Install via USB" must stay enabled** (first install attempts failed with `INSTALL_FAILED_USER_RESTRICTED` until it was on / confirmed).
Currently on the phone: `com.mnn.sdk.test` (SDK instrumented-test APK); model files in `/data/local/tmp/LFM2.5-230M-MNN/` and copied into that app's `files/models/LFM2.5-230M-MNN/` (with the **patched** `llm_config.json`).

### Commands that worked (copy these)
```bash
# Git Bash: always prefix adb calls that contain Unix-style paths
export MSYS_NO_PATHCONV=1; A=/d/Android/Sdk/platform-tools/adb.exe

# MNN SDK has NO gradlew script — run the wrapper jar (PowerShell, from repos\mnn-android-sdk):
#   $env:JAVA_HOME="C:\Program Files\Eclipse Adoptium\jdk-17.0.20.101-hotspot"; $env:ANDROID_HOME="D:\Android\Sdk"; $env:GRADLE_USER_HOME="D:\Android\gradle-home"
#   & "$env:JAVA_HOME\bin\java.exe" -cp gradle\wrapper\gradle-wrapper.jar org.gradle.wrapper.GradleWrapperMain :mnn-sdk:assembleDebugAndroidTest --console=plain -q
#   (needs local.properties with  sdk.dir=D\:\\Android\\Sdk  — already created, gitignored)
# Install + run SDK LLM tests on phone:
$A install -r -t <path>/mnn-sdk-debug-androidTest.apk
$A shell am instrument -w -e class com.mnn.sdk.MNNLlmInstrumentedTest com.mnn.sdk.test/androidx.test.runner.AndroidJUnitRunner
$A logcat -d -s MNNLlmInstrumentedTest:I

# WSL (Rust/Foundry): pipe a script so Windows quoting doesn't interfere
printf '%s\n' '. $HOME/.cargo/env' 'cargo --version' | wsl.exe -d Ubuntu -- bash -s
# TIP: build Rust with CARGO_TARGET_DIR=~/target/<crate> (building on /mnt/c is slow)
# Android .so:  cargo ndk -t arm64-v8a build --release   (verified with a smoke crate)
```
The wallet repo has a proper `gradlew` (Gradle 8.13, AGP 8.11.0, Kotlin 2.0.21, compile/target SDK 36, minSdk 24, `ndkVersion 27.0.12077973`, JDK 17). Its README says "NDK 30" — **wrong**; the build file pins 27.0.12077973.

## 4. Key decisions (don't relitigate without reason)
1. **Model = LFM2.5-230M-MNN** (replaces spec's Qwen2.5-0.5B suggestion) — small CPU/RAM footprint; AHEM's native hook was already written for LFM.
2. **Keep** the existing Compose UI and Rust core; replace data layer beneath it. Display name "fusion_wallet" via resources, no mass package rename (`app.fusion.wallet` stays).
3. **Three Android modules**: `:app` (UI + Rust), `:inference` (separate UID, **no INTERNET**), `:signer` (separate UID, keys). Caller UID/package/cert checked per Binder call.
4. **Dev chain = Anvil** on PC, phone reaches it via `adb reverse tcp:8545 tcp:8545`; **public = Base Sepolia (84532)**. Only chain IDs 31337/84532 allowed.
5. **AHEM = stretch / SHADOW mode only**; no claim of tensor enforcement unless the hook is patched into the shipped libllm.so and gates pass.
6. **Honesty rules** (from spec): missing tool/device/model = **BLOCKED**, never PASS; no hardcoded latencies/hashes/balances/scenario answers; no weakening assertions to go green; model never supplies amounts/addresses/calldata/keys.
7. Rename "Kavachi" → `fusion_wallet` applied everywhere in docs; hash domain tags are now `FUSION_WALLET_{POLICY,SESSION,MANDATE,ACTION,CANDIDATE}_V3` and the 15 expected hashes in `vectors.json` + 5 fixtures were regenerated; `check_package.py` etc. pass. Schema IDs use `https://fusion_wallet.local/...` (underscore isn't a valid hostname char; harmless to the validator — consider `fusion-wallet.local`).

## 5. Findings so far
- **S1 (done): LFM2.5-230M runs on bundled MNN 3.6.1** on the Redmi 8 — no MNN rebuild needed. Decode ~17–32 tok/s, prefill 176–529 ms, first token ~0.2–0.5 s. 7/8 SDK instrumented tests pass.
- **BUG: shipped LFM2 chat template drops all message text.** `llm_config.json` template uses `message.get("content")` / `messages[0].get("content")`; MNN 3.6.1's jinja returns empty → prompt renders as `<|im_start|>user\n<|im_end|>`. Fixed by replacing with `["content"]` (patched copy above; **must ship the patched config in `:inference` provisioning and record its sha256**). SDK's own template test is too weak (only checks for `<|im_start|>`) — add a product test asserting the user text appears in `renderPrompt`.
- The 8th SDK test (`generationTimeoutBoundsARunawayGeneration`) "fails" only because `chat()` *throws* `MNNException` on the 1 ms timeout while the test expects a return — a test/API mismatch, not a device fault.
- 230M model answered "2+2" with "3" → candidate-ID selection accuracy is unmeasured; run the 40-case corpus early. Design fails safe to HOLD.
- Not yet measured: cold-load time, peak RSS, thermal behaviour over 20 sequential requests.
- Repo audit highlights (full list in spec §2 / `REPOSITORY_AUDIT.md` inside the zip): wallet is demo-data only (DemoIntentSource, hardcoded health/balances), f64 money math, `execute_intent(private_key: String)` crosses the FFI/UI boundary, broadcast hash treated as success, fail-open ledger. SDK CI failure is a probable cross-ABI false positive (compares armv7 imports to arm64 libs). AHEM's `mnn_lfm_latent_hook.cpp` is an unfinished template with a lock-lifetime bug; bundled `libllm.so` exports none of the `ahem_*` hooks.

## 6. Status snapshot
| Area | State |
|---|---|
| Spec/handoff docs, rename | ✅ Done & verified (package checks pass) |
| Toolchain install | ✅ Done (see §3) |
| Phone connected & authorised | ✅ |
| Repos cloned at pinned commits | ✅ (no code changes yet) |
| S1 model go/no-go | ✅ GO (with template patch) |
| Everything in the implementation (J00–J10) | ⬜ Not started (J00 next) |

## 7. TODO (brief; detailed items in `PLAN.md`)
Legend: ⬜ todo · 🟦 in progress · ✅ done · ⛔ blocked. Job IDs match the spec's J00–J10.

| # | Task | Status | Notes |
|---|---|---|---|
| S1 | LFM2 on MNN 3.6.1 on Redmi 8 | ✅ | Needs template patch; still measure load time + RSS |
| J00 | Baseline: `cargo test --locked` (WSL), wallet `assembleDebug`, install on phone, screenshots of all screens, record failures; write `integration-lock.json` | ⬜ | **Next up** |
| J01 | Fix SDK CI ABI-matched import check; build SDK AAR → local Maven + checksum | ⬜ | |
| J02 | Rust: `amount/treasury/policy/obligations/candidates/preview/protocol`, V3 DTOs, vector parity, remove raw-key export, regen UniFFI | ⬜ | Parallel track A |
| J03 | Foundry project: DemoUSD, 2 ERC-4626 vaults, `RestrictedTreasuryWallet` V3, fuzz/invariant/bypass tests, Anvil deploy + manifest | ⬜ | Parallel track B |
| J05 | `:inference` module (no INTERNET, MNN + patched LFM config, provisioning w/ hash check, strict candidate JSON, deadlines) | ⬜ | |
| J04 | `:signer` module (Keystore keys, AIDL + caller checks, owner review screen, journal) | ⬜ | |
| J06 | Wire `:app`: TreasuryRepository, LocalPlannerClient, SignerClient; demo data → debug only | ⬜ | |
| **M** | **Vertical slice: real LFM pick → Rust preview → signer → Anvil tx → reconciled → buckets update** | ⬜ | Demo backbone — nothing else ships before it |
| J07 | Durable journal, BROADCAST/UNKNOWN/CONFIRMED, reconcile-before-retry | ⬜ | |
| J08 | UI on real state: buckets, decoded cards, queue/cancel panel, ArenaScreen, ASR/FBR table | ⬜ | |
| E | Run 20 benign + 20 malicious cases ×3 on phone; latency/memory/availability | ⬜ | |
| BS | Base Sepolia deploy + action + cancel receipts | ⛔ | Needs RPC URL + faucet ETH from user |
| J09 | AHEM SHADOW veto adapter | ⬜ | Stretch |
| J10 | Gate runner, `verify_release.py`, README, demo video (≤5 min), deck (≤10 min), security self-assessment | ⬜ | Freeze 20 Oct 15:59 |

## 8. Open questions / needs from the user
- Base Sepolia RPC URL (public or Alchemy/Infura key) + faucet ETH for 3 throwaway addresses (owner/deployer, agent, guardian).
- OK to commit/push the `fusion-v3-integration` branches to GitHub? (Not done.)
- Is anyone else working in parallel (to split Rust vs. contracts tracks)?
- Confirm organizer acceptance of custom test assets / testnet evidence (spec says to verify separately).

## 9. Gotchas for the next agent
- Git Bash mangles `/data/...` paths → `export MSYS_NO_PATHCONV=1`.
- PowerShell tool: don't use `Remove-Item` on `$env:LOCALAPPDATA\...\*` paths (blocked by a safety rule); `2>&1` on native exes causes `NativeCommandError` noise even on success.
- `sdkmanager` license acceptance needs `yes | sdkmanager ...` from bash (PowerShell piping didn't work).
- `wsl.exe` output contains NULs → pipe through `tr -d '\0'`; `/etc/wsl.conf` has a harmless duplicated `user.default` warning on every call.
- Don't paste an independent root build of the SDK into the wallet; spec says publish SDK as a checksum-pinned local Maven artifact instead.
- Never edit generated UniFFI Kotlin (`fusion-core-kotlin/.../fusion_core.kt`) by hand — regenerate.
- Don't upgrade dependency versions while wiring the pipeline.

## 10. Progress log (append newest at bottom; one line per event, with date)
- 2026-10-17 — Read & summarised spec/handoff (Kavachi architecture; 11 jobs J00–J10; 133 acceptance defs, 119 core).
- 2026-10-17 — Renamed Kavachi → fusion_wallet across `.md`, `.docx`, and the zip (53 files); regenerated V3 test-vector hashes + 5 fixtures; rebuilt `SHA256SUMS`; package checks + 10 gate self-tests pass. Originals backed up in session scratchpad.
- 2026-10-17 — Audited system requirements vs. PC: nearly everything missing. Installed JDK 17, Android SDK/NDKs/CMake/platform-tools (D:), Rust 1.94 + cargo-ndk + Foundry + build tools (WSL), Linux NDK r27. Verified Rust→arm64 `.so` smoke build.
- 2026-10-17 — Uninstalled Docker Desktop + data (~83 GB) and cleared safe caches (npm, crash dumps, temp, WU download cache). C: free 23 → 108 GB.
- 2026-10-17 — Phone authorised: Redmi 8, arm64-v8a, Android 10, 3.6 GB RAM. Model chosen: LFM2.5-230M-MNN.
- 2026-10-17 — Cloned 3 repos at pinned commits onto branch `fusion-v3-integration`; wrote `PLAN.md`.
- 2026-10-17 — **S1 done:** built SDK test APK, ran on Redmi 8. Model works (17–32 tok/s). Found & fixed chat-template content-dropping bug (patched config). 7/8 SDK tests pass (8th is a test-design mismatch).
- 2026-10-17 — Created this `CONTEXT.md`.
- 2026-10-17 — Published project root as PUBLIC repo https://github.com/AdribMahmud101/fusion_wallet (docs, plan, context, handoff zip; 3 code repos as submodules pinned to their commits). Commits use the GitHub noreply identity (local git config); global git identity looks misconfigured (name is an email, email differs from the GitHub account) — not changed. Phone serial removed from CONTEXT.md for the public repo. Integration branches `fusion-v3-integration` in the 3 code repos are still local-only.
