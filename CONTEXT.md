# fusion_wallet — Project Context & Handoff

> **Living document.** Any agent working on this project: read this first, and append to the *Progress log* (bottom) + update the *Status* and *TODO* tables whenever something is done, decided, or found. Detailed schedule/rationale: `PLAN.md`. Full spec: `Fusion_Wallet_Final_Base_Architecture.md` (+ handoff zip).
> Last updated: **2026-10-17 21:00 (Asia/Dhaka)**

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
| Assets / Aave verified facts + glossary | `docs/ASSETS_AND_AAVE.md` (addresses, live market state, plain-English terms) |
| Accountant design doc | `docs/ACCOUNTANT_AGENT.md` (entities, data-egress classification, guardrails, DeepInfra specifics) |
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

8. **Accountant Agent + cloud LLM (decided 2026-10-17, by the project owner):** build an accountant with embedded Room DB entities (balances, Aave aTokens/lending positions, obligations, ledger…) and a **DeepInfra** cloud analyst, plus the Android UI. This **deliberately relaxes the spec's "no private context to a cloud LLM" rule** to: *cloud is OFF by default, opt-in with versioned consent, and receives only a coarse read-only brief behind a fail-closed egress guard*. Cloud text is untrusted advice; it never feeds the planner/policy/signer/contract. On-device LFM still handles anything that selects or executes an action. Details: `docs/ACCOUNTANT_AGENT.md`.
9. DeepInfra: `https://api.deepinfra.com/v1/openai/chat/completions`, Bearer key; host pinned; **`google/` and `anthropic/` models refused** (DeepInfra says those providers retain data); default model `meta-llama/Llama-3.3-70B-Instruct-Turbo`. API key: Keystore-wrapped at runtime; debug-only fallback `DEEPINFRA_API_KEY` in gitignored `fusion-core-android/local.properties`. **Repo is public — never commit a key; rotate any key pasted into chat.**

10. **UI restructure (owner-approved 2026-10-17):** 3 tabs — **Treasury** (what you have; the Accountant is a role inside it, not a tab) · **Agent** (AI-as-participant: passport/mandate + decision timeline with per-layer checks) · **Proof** (scorecard, attack arena, receipts, privacy/egress log). Chat/Tokens/Ledger tabs removed from navigation. English + 中文 runtime toggle.
11. **Currency/vault decision (owner-approved):** unit = **Aave Base Sepolia test USDC** (`0xba50…4D5f`, 6 dp). Vault A = real Aave ERC-4626 wrapper `waBasSepUSDC` (`0xf430…b32a`); Vault B = our own OZ ERC-4626 demo vault on the same USDC. Supersedes DemoUSD + two custom vaults. ETH is gas (not ERC-20, excluded from total). Show Aave **APR** (real: 0.21 %), never APY, never invented yields.

12. **UI principles (owner feedback: "too complex"; guided by the installed `ui-ux-pro-max` plugin):** one dominant message per screen; progressive disclosure (details behind a tap via `Expander`); plain words (Home / AI Agent / Safety, "Spare cash", "Set aside for bills"); sentence-case titles; Inter for text, mono only for numbers; calm tones in `ui/common/Common.kt` `Tone` (no purple); touch targets ≥ 44–48 dp; never colour-only status; every string in `strings_accountant.xml` for EN and zh. Keep new screens within this rule: **headline → status → one action → details on tap.**

13. **Demo sign-in & roles (owner-requested 2026-10-17):** login screen with real salted-PBKDF2 checks + 5-try/30 s lockout; 3 public demo accounts — `mei@fusion.demo` / `Owner#2026` (Owner), `wei@fusion.demo` / `Manager#2026` (Finance manager), `judge@fusion.demo` / `Auditor#2026` (Auditor, view-only). Accounts hold no financial data and unlock nothing on-chain; they are provisioned in **all** build types (unlike the financial fixtures, which are debug-only). Roles only gate UI (Owner: cloud/privacy settings + pause + approve; Manager: pause; Auditor: none) — **the contract/signer, not the app, enforce spending limits**. Avatar = 6 colours × 7 glyphs (incl. initials), editable in the profile sheet (top-right avatar). Session = user id in `fusion_prefs`.

14. **Bento + charts (owner-requested 2026-10-17):** every screen is a bento grid of rounded tiles (`ui/common/Bento.kt`: `BentoTile`, `BentoRow`, `StatTile`). Charts are hand-drawn with Compose Canvas (`ui/charts/Charts.kt`: `DonutChart`, `RingGauge`, `Sparkline`, `StackedBars`, `LabeledBar`, `SegmentBar`) — **no chart library**. Rules each chart follows: spoken `contentDescription`; the numbers also shown as text (never colour-only); draw-in animation skipped when system animation scale is 0. Chart data logic is pure and unit-tested (`accountant/domain/Analytics.kt`: weekly spend, 30-day totals, % change, bill timing). Charts added only where they answer a question: Home = allocation donut, cash-runway gauge, 30-day trend, weekly spending bars, interest-rate bars, bills timeline; Agent = daily-limit ring + Done/Waiting/Blocked count tiles; Safety = headline + stat tiles + 2-column attack grid.
15. **Default avatars:** all three default demo users start with the plain **person** icon (colours differ); users can change colour/icon in their profile.

## 5. Findings so far
- **S1 (done): LFM2.5-230M runs on bundled MNN 3.6.1** on the Redmi 8 — no MNN rebuild needed. Decode ~17–32 tok/s, prefill 176–529 ms, first token ~0.2–0.5 s. 7/8 SDK instrumented tests pass.
- **BUG: shipped LFM2 chat template drops all message text.** `llm_config.json` template uses `message.get("content")` / `messages[0].get("content")`; MNN 3.6.1's jinja returns empty → prompt renders as `<|im_start|>user\n<|im_end|>`. Fixed by replacing with `["content"]` (patched copy above; **must ship the patched config in `:inference` provisioning and record its sha256**). SDK's own template test is too weak (only checks for `<|im_start|>`) — add a product test asserting the user text appears in `renderPrompt`.
- The 8th SDK test (`generationTimeoutBoundsARunawayGeneration`) "fails" only because `chat()` *throws* `MNNException` on the 1 ms timeout while the test expects a return — a test/API mismatch, not a device fault.
- 230M model answered "2+2" with "3" → candidate-ID selection accuracy is unmeasured; run the 40-case corpus early. Design fails safe to HOLD.
- Not yet measured: cold-load time, peak RSS, thermal behaviour over 20 sequential requests.
- Repo audit highlights (full list in spec §2 / `REPOSITORY_AUDIT.md` inside the zip): wallet is demo-data only (DemoIntentSource, hardcoded health/balances), f64 money math, `execute_intent(private_key: String)` crosses the FFI/UI boundary, broadcast hash treated as success, fail-open ledger. SDK CI failure is a probable cross-ABI false positive (compares armv7 imports to arm64 libs). AHEM's `mnn_lfm_latent_hook.cpp` is an unfinished template with a lock-lifetime bug; bundled `libllm.so` exports none of the `ahem_*` hooks.

- **Accountant (done 2026-10-17):** package `app.fusion.wallet.accountant` in `repos/Fintech_Wallet_Agent` (commit `2fb680b` on `fusion-v3-integration`). 40 JVM unit tests pass (0 skipped); Room schema v1 exported; debug APK (17.9 MB) installed on the Redmi 8 and the new Accountant tab renders the spec reference state from the on-device DB (10,000 total · 3,500 free/3,000 sweepable · 1,500 reserved · 5,000 deployed; Aave aToken 4.20 %, ERC-4626 vault 3.10 %). **Not yet done: any live DeepInfra call** (awaiting API key), instrumented (on-device) Room tests, chain→Room sync (real Aave reads), string resources (Accountant UI copy is hard-coded English).
- Original UI bug fixed: bottom tab bar was drawn under the system navigation buttons (edge-to-edge, 3-button nav) and status-bar icons were dark-on-dark; added `navigationBarsPadding()` and `SystemBarStyle.dark`. The repo's launcher icon is still the default Android one (pre-existing).
- Build facts: wallet repo builds with `gradlew :app:testDebugUnitTest :app:assembleDebug` (Gradle 8.13, JDK 17; Room 2.6.1 + KSP 2.0.21-1.0.28). Kotlin-DSL gotcha: `java.util.Properties` must be imported (`java` resolves to the Gradle extension). Python on Windows rewrites LF→CRLF in text mode — use `newline=''` when editing repo files.
- Phone must stay unlocked/portrait for screenshots: `adb shell settings put system accelerometer_rotation 0`; taps near the bottom hit the system nav bar unless the tab bar is inset (now fixed).

- **UI verified on Redmi 8 (2026-10-17 20:08):** Treasury / Agent / Proof render in EN and 中文; live queue countdown ticks; blocked-attack card shows ✗ on the model layer. 45 JVM tests pass (0 skipped). Dead code still in the repo but unreachable: `MainViewModel`, `DemoIntentSource`/`DemoData`, `ChatFeed`, `PromptComposer`, `PreFlightGuardrailCard`, `TokensScreen`, `LedgerScreen`, `SettingsDrawer`, terminal sheet, `TopHealthAppBar` top bar (its `FusionBottomBar` is still used). **Remove before release** (spec: a TESTNET release must not link the fixture intent interpreter). `PreFlightGuardrailCard` is the intended base for the decoded action card — extract, don't delete blindly.
- The agent's Pause/Cancel/Run buttons are present but **disabled** (explained in-app) until the contract/signer exist; the arena shows "not run"; scorecard shows "not measured". Do not fake them. Decision/arena/mandate rows come only from the debug seeder.
- Real data checked: Aave USDC market active/not paused, supply APR 0.2075 %, waUSDC is ERC-4626 (see `docs/ASSETS_AND_AAVE.md`).

## 6. Status snapshot
| Area | State |
|---|---|
| Spec/handoff docs, rename | ✅ Done & verified (package checks pass) |
| Toolchain install | ✅ Done (see §3) |
| Phone connected & authorised | ✅ |
| Repos cloned at pinned commits | ✅ (no code changes yet) |
| S1 model go/no-go | ✅ GO (with template patch) |
| Accountant Agent (Room entities, local analyst, DeepInfra client, egress guard, UI) | ✅ Built, 40 unit tests green, runs on phone; ⛔ live cloud test needs API key |
| UI: Treasury / Agent / Proof, EN + 中文, honest badges | ✅ Built and verified on phone (data is debug fixtures) |
| Rest of the implementation (J00–J10) | ⬜ Not started (J00 next) |

## 7. TODO (brief; detailed items in `PLAN.md`)
Legend: ⬜ todo · 🟦 in progress · ✅ done · ⛔ blocked. Job IDs match the spec's J00–J10.

| # | Task | Status | Notes |
|---|---|---|---|
| S1 | LFM2 on MNN 3.6.1 on Redmi 8 | ✅ | Needs template patch; still measure load time + RSS |
| A1 | Room entities + DAOs + read model (accounts, assets/aTokens, lending, obligations, ledger, buckets, insights, egress log, privacy settings) | ✅ | `accountant/data`, schema v1 |
| A2 | LocalAccountant (deterministic, exact figures) | ✅ | |
| A3 | Egress guard + coarse brief + DeepInfra client + reply validator + agent | ✅ | 40 JVM tests |
| A4 | Accountant UI tab (+ debug seeder) | ✅ | verified on Redmi 8 |
| A5 | **Live DeepInfra test** (real key → real call → check payload in activity log, reply validates) | ⛔ | Needs `DEEPINFRA_API_KEY`; then rotate it |
| A6 | Instrumented Room tests on device (DAO queries, latest-per-asset, schema) | ⬜ | queries are only compile-checked so far |
| A7 | Chain→Room sync: Aave V3 aToken `balanceOf`/`scaledBalanceOf`, liquidity index, supply APY on Base Sepolia; replace demo seeder in real mode | ⬜ | depends on Rust/RPC path (J06) |
| A8 | Accountant UI strings to `strings.xml` (EN + zh) | ✅ | `strings_accountant.xml` ×2; accessibility pass still ⬜ |
| U1 | 3-tab shell (Home / AI Agent / Safety), top bar honesty, language toggle | ✅ | simplified per owner feedback 2026-10-17 20:30 |
| U2 | Agent tab real data: decisions/mandate/queue from chain + signer journal; wire Pause & Cancel | ⬜ | blocked on J03/J04/J07 |
| U3 | Proof tab real data: arena runner (7 scenarios) against Anvil, benign cases for FBR, scorecard | ⬜ | blocked on J03/J05; see spec ENHANCEMENTS_AND_UX.md |
| U4 | Decoded action card (extend `PreFlightGuardrailCard`) + owner-review screen in `:signer` | ⬜ | |
| U5 | Delete dead demo code listed in §5 before any release build | ⬜ | |
| U8 | Demo sign-in, roles, profile, avatars (EN + zh); role gating in UI and VM | ✅ | `app.fusion.wallet.auth`, `ui/auth`; 54 tests green; verified on Redmi 8 |
| U10 | Bento grid + charts on Home / Agent / Safety (no chart library); `Analytics` unit-tested | ✅ | 60 tests green; verified on Redmi 8 |
| U11 | Charts fed by real data: 30-day bucket snapshots written by the sync worker (today debug-seeded); weekly spend from confirmed ledger | ⬜ | depends on A7/J06 |
| U12 | Chart polish: tap-to-inspect values, dark/light check, tablet layout, TalkBack pass | ⬜ | |
| U9 | Real auth later: biometric unlock / device-bound keys for the signer app; per-user data | ⬜ | not needed for demo |
| U6 | Obligations editor (draft → owner commitment), gas top-up hint, onboarding/mandate-setup flow | ⬜ | judges' "set mandate once" moment |
| U7 | Confirm test-USDC faucet + ability to deposit into `waBasSepUSDC` on a Base Sepolia fork | ⬜ | before building the contract |
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
- **DeepInfra API key** for the live test (A5). Put it in the *gitignored* `fusion-core-android/local.properties` as `DEEPINFRA_API_KEY=...` (or type it in the app's Accountant → Cloud analysis field). Rotate it afterwards.
- Aave V3 on Base Sepolia vs. only the spec's two demo ERC-4626 vaults: entities support both; the real Aave adapter (A7) is not built.
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

## 9b. Mistakes made once — don't repeat
- **Never run `git submodule update --remote` in the root repo.** It checks the wallet submodule out at upstream `main` (the original commit), the working tree "loses" my branch's files, and a following `git add -A` records the wrong pointer. (Happened 2026-10-17; fixed by `git switch fusion-v3-integration` in `repos/Fintech_Wallet_Agent` and re-committing the pointer; no work was lost.) Bump pointers with `git add repos/<name>` only, after confirming the submodule is on its branch (`git -C repos/<name> branch --show-current`).
- Python text-mode writes rewrite LF as CRLF on Windows — pass `newline=''`.
- Two "phantom" state changes seen on the Redmi 8 (app opened already signed in once; Lin Mei's avatar icon changed briefcase→person once). A step-by-step repro (clean install → sign in → relaunch → swipes → tab taps), checking the SQLite file after each step, changed nothing. **Most likely the device owner touched the phone** (the same person who then asked for the person icon). Not treated as a bug; if it recurs, pull the DB (`adb exec-out "run-as app.fusion.wallet cat databases/accountant.db*"`) and check `app_user` / `fusion_prefs.xml`.

## 10. Progress log (append newest at bottom; one line per event, with date)
- 2026-10-17 — Read & summarised spec/handoff (Kavachi architecture; 11 jobs J00–J10; 133 acceptance defs, 119 core).
- 2026-10-17 — Renamed Kavachi → fusion_wallet across `.md`, `.docx`, and the zip (53 files); regenerated V3 test-vector hashes + 5 fixtures; rebuilt `SHA256SUMS`; package checks + 10 gate self-tests pass. Originals backed up in session scratchpad.
- 2026-10-17 — Audited system requirements vs. PC: nearly everything missing. Installed JDK 17, Android SDK/NDKs/CMake/platform-tools (D:), Rust 1.94 + cargo-ndk + Foundry + build tools (WSL), Linux NDK r27. Verified Rust→arm64 `.so` smoke build.
- 2026-10-17 — Uninstalled Docker Desktop + data (~83 GB) and cleared safe caches (npm, crash dumps, temp, WU download cache). C: free 23 → 108 GB.
- 2026-10-17 — Phone authorised: Redmi 8, arm64-v8a, Android 10, 3.6 GB RAM. Model chosen: LFM2.5-230M-MNN.
- 2026-10-17 — Cloned 3 repos at pinned commits onto branch `fusion-v3-integration`; wrote `PLAN.md`.
- 2026-10-17 — **S1 done:** built SDK test APK, ran on Redmi 8. Model works (17–32 tok/s). Found & fixed chat-template content-dropping bug (patched config). 7/8 SDK tests pass (8th is a test-design mismatch).
- 2026-10-17 — Created this `CONTEXT.md`.
- 2026-10-17 — Published project root as PUBLIC repo https://github.com/AdribMahmud101/fusion_wallet (docs, plan, context, handoff zip; 3 code repos as submodules pinned to their commits). Commits use the GitHub noreply identity (local git config); global git identity looks misconfigured (name is an email, email differs from the GitHub account) — not changed. Phone serial removed from this file for the public repo.
- 2026-10-17 — **Bento + charts + default icon:** restyled all three tabs as bento grids and added Canvas charts (donut, ring gauges, sparkline, stacked bars, rate bars, segment bar); added `Analytics` + 6 tests and 30 days of debug-seeded bucket history; shortened week labels; default demo users now all use the person icon. 60 tests pass; verified on Redmi 8 incl. login cards. Investigated a phantom avatar change by reproducing step-by-step — not reproducible (see §9b).
- 2026-10-17 — **Sign-in + profile:** added login (PBKDF2, lockout), three demo roles, role-aware controls, profile sheet with editable avatar/name/language and sign-out, avatar in the top bar; EN+zh; 9 new tests (54 total). Verified on Redmi 8: wrong password error, owner/manager/auditor differences, avatar edit, session restore. Also fixed my submodule-pointer mistake (see §9b).
- 2026-10-17 — **Simplification pass:** owner said the UI was too complex. Installed the `ui-ux-pro-max` plugin and applied its rules (the `ui-styling` skill is web/Tailwind only, so not used for Compose). Home = one "Spare cash" headline + status + Ask tile; Agent = summary + tap-to-expand decision cards; Safety = one headline + expanders for cloud/advanced; plain labels, plurals, number formatting, sky-blue instead of violet, slim top bar, ≥44 dp targets. Verified on Redmi 8 in EN and 中文; tests green.
- 2026-10-17 — **UX review + restructure:** validated currency model against live Aave contracts on Base Sepolia (`docs/ASSETS_AND_AAVE.md`); corrected my own errors (invented 4.2 % yield, APY vs APR label, missing gas concept, Accountant as a tab). Owner approved: 3 tabs, real Aave USDC + own vault, EN+中文. Built Treasury/Agent/Proof, new Room entities (mandate, decisions, arena), full zh strings with runtime switch. Verified on Redmi 8; 45 tests green. Commit on `fusion-v3-integration`.
- 2026-10-17 — **Owner pivot:** build Accountant Agent + Room entities + DeepInfra cloud LLM + guardrails + UI first. Read docs.deepinfra.com; wrote `docs/ACCOUNTANT_AGENT.md`; implemented `app.fusion.wallet.accountant` (data/domain/cloud/agent), Accountant tab and ViewModel; 40 unit tests pass; installed and verified on Redmi 8; fixed nav-bar inset + system-bar icon bugs. Committed `2fb680b` on `fusion-v3-integration` in `Fintech_Wallet_Agent`. Live DeepInfra test pending API key.
