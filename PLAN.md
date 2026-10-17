# fusion_wallet — Implementation Plan (v3 integration)

> Live status, decisions, findings and progress log: see `CONTEXT.md` (update that first). This file is the detailed schedule.

Created 2026-10-17. Deadline: **2026-10-20 21:59 Asia/Dhaka** (23:59 Beijing). Artifact freeze: **2026-10-20 15:59 Dhaka**.
Spec: `Fusion_Wallet_Final_Base_Architecture.md` + handoff zip. This plan sequences that spec for a ~3-day window.

## Ground truth

| Item | State |
|---|---|
| Repos (branch `fusion-v3-integration`, at pinned commits) | `repos/Fintech_Wallet_Agent` (e5ae118), `repos/mnn-android-sdk` (5c8f61f), `repos/ahem` (8832419) |
| Integration repo | `Fintech_Wallet_Agent` — new `contracts/`, `:inference`, `:signer` live here |
| Model | `taobao-mnn/LFM2.5-230M-MNN` @ 216a223 (132 MB, lfm2, hidden 1024) |
| Device | Redmi 8, arm64-v8a, Android 10, 3.6 GB RAM — low-end on purpose to prove responsiveness |
| Toolchains | Android SDK/NDKs on Windows (D:), Rust 1.94 + cargo-ndk + Foundry in WSL |
| Chain for dev | Anvil on the PC; phone reaches it via `adb reverse tcp:8545 tcp:8545` |
| Public chain | Base Sepolia (84532) — needs an RPC URL and faucet ETH |

Existing code to keep: Compose shell (MainActivity, FusionWalletApp, theme, nav, ChatFeed, TokensScreen, LedgerScreen, SettingsDrawer, terminal sheet, PreFlightGuardrailCard), Rust strict serde/Alloy/UniFFI patterns (~3.1k LOC in `fusion-core/src`).
Existing code to replace: DemoIntentSource/DemoData in the real build, f64 money, raw `private_key` export, broadcast-as-success, fail-open ledger.

## Biggest risks (front-loaded)

1. **LFM2 on bundled MNN 3.6.1** — `libllm.so` shows no LFM2-specific symbols. LFM2 is a hybrid conv+attention model; if 3.6.1 can't run it, we must rebuild MNN (AHEM's `third_party/MNN` @ 39be21c is a candidate). → Spike S1, go/no-go tonight.
2. **Scope vs. time** — 119 core acceptance cases won't all be evidenced by the freeze. We build one real vertical slice first, then widen. Anything unfinished is disabled and labelled, never faked.
3. **230M model output quality** — may emit malformed JSON / wrong IDs. Design already fail-safes to HOLD; measure early with the 40-case corpus.
4. **Rust builds from `/mnt/c` in WSL are slow** — set `CARGO_TARGET_DIR=~/target/<crate>` in WSL.

## Phase 0 — Spikes & baseline (tonight, 17 Oct)

- [x] **S1 LFM2 go/no-go — GO** (2026-10-17, Redmi 8): loads and runs on bundled MNN 3.6.1, ~17–32 tok/s decode, prefill 0.2–0.5 s. No MNN rebuild needed. **Caveat:** shipped chat template drops all message text (`.get("content")` unsupported); patched config required (see memory `lfm2-template-patch`). Still to measure: cold-load time and peak RSS.
- [ ] **J00 baseline**: `cargo test --locked` (WSL), `./gradlew :app:assembleDebug`, install on Redmi 8, screenshots of every screen, record failures. Write `integration-lock.json` (commits, toolchains, model hash).
- [ ] **J01 CI fix**: ABI-matched ELF import check in SDK `build.yml`; build SDK release AAR → local Maven repo with checksum.

## Phase 1 — Deterministic core + contract (18 Oct)

Two independent tracks (can run in parallel):

**A. Rust core (J02, part of J06)** — `fusion-core/src/`
- [ ] `amount.rs` decimal-string ↔ U256 raw units, checked math (no f64 anywhere in money paths)
- [ ] `treasury.rs` buckets (free / reserved / deployed / sweepable / shortfall) per spec §3
- [ ] `policy.rs` tiers (≤10 immediate, ≤100 delayed 180 s, >100 owner), hard/daily/session caps
- [ ] `obligations.rs` commitment root (sorted idHash, ABI rows) — matches `HASHING.md`
- [ ] `candidates.rs` ≤7 candidates + HOLD, deterministic ordering, digests
- [ ] `preview.rs` pure read-only preview; `protocol.rs` V3 DTOs
- [ ] Parity test against `vectors.json` (3 vectors, FUSION_WALLET_*_V3 tags)
- [ ] Remove `execute_intent(private_key)` from app-facing exports; regenerate UniFFI Kotlin

**B. Contracts (J03)** — new `contracts/` Foundry project
- [ ] Pin OpenZeppelin + forge-std; `DemoUSD` (6 dp), two ERC-4626 demo vaults
- [ ] `RestrictedTreasuryWallet` V3 per `IRestrictedTreasuryWallet.sol` + `ONCHAIN_SPEC.md`: obligation commitment, sessions, 3 tiers, queue + cancel window, owner approve, pause/guardian, recovery
- [ ] Unit + fuzz + invariant tests; direct-call bypass tests; Solidity digest == Rust digest == vectors
- [ ] Anvil deploy script + deployment manifest (chainId, addresses, code hashes)

## Phase 2 — Android split + first real vertical slice (19 Oct)

- [ ] **J05 `:inference`** app (`app.fusion.wallet.inference`, separate UID, **no INTERNET**): MNN + LFM2.5-230M, model provisioning with hash check, one job at a time, strict candidate-ID JSON, deadline + stale-result rejection, real capability report
- [ ] **J04 `:signer`** app (`app.fusion.wallet.signer`): Keystore-wrapped secp256k1 keys, AIDL with caller UID/package/cert checks, independent policy re-check, trusted owner review screen, signing journal
- [ ] **J06 wiring** in `:app`: `TreasuryRepository` (pinned block snapshots from Anvil), `LocalPlannerClient`, `SignerClient`; DemoData moves to debug/test only
- [ ] **Milestone M-slice**: phone → real LFM choice → Rust preview → signer → Anvil tx → receipt reconciled → buckets update. *This is the demo's backbone; nothing else ships before it works.*
- [ ] **J07 (minimum)**: write-ahead intent journal, BROADCAST/UNKNOWN/CONFIRMED states, reconcile-before-retry, no budget release on ambiguous send

## Phase 3 — Five enhancements, evidence, demo (20 Oct, freeze 15:59)

- [ ] **J08 UI** on real state: three-bucket card, decoded action card (extends PreFlightGuardrailCard), queue panel with cancel countdown, `ArenaScreen` (attack arena), ASR/FBR metric table
- [ ] Run 20 benign + 20 malicious cases ×3 on the Redmi 8; record latency p50/p95, memory, availability
- [ ] Base Sepolia deploy + one real action and one cancel receipt
- [ ] Gate runner → `results.json`; `verify_release.py` for the profile we can honestly claim
- [ ] 5-min demo video, ≤10-min deck, README/setup, security self-assessment
- [ ] **Freeze 15:59 Dhaka**; rehearsal; no model/runtime experiments after freeze

## AHEM (J09) — stretch, honest labelling

- Day 2–3 only if the slice is green. Target **SHADOW** mode: veto-only adapter that logs what it *would* block; no claim of tensor enforcement.
- ENFORCED requires the patched MNN hook in the shipped `libllm.so`, real callbacks on device, held-out calibration and ablation. Not planned before the deadline unless S1 already forces an MNN rebuild (then the hook patch can ride along).

## Explicit cuts / deferrals (labelled in app and submission)

- AHEM ENFORCED mode; HYBRID_PUBLIC cloud mode (disabled in build)
- Full device-lifecycle matrix (reboot, signer replacement, memory-pressure) — sample only
- Reorg fault injection beyond one Anvil scenario
- Separate guardian device

## Rules we keep regardless of time (from spec)

- Model picks a candidate ID or HOLD only; never amounts/addresses/keys.
- No mainnet, no real funds, no raw key in UI/model process.
- A missing tool/device/model is **BLOCKED**, never PASS. No hardcoded latencies, hashes, balances or scenario answers.

## Open items for the user

- Base Sepolia RPC URL (public or Alchemy/Infura key) and faucet ETH for 3 throwaway addresses (deployer/owner, agent, guardian)
- Whether we commit/push the `fusion-v3-integration` branches to GitHub (not done yet)
- Who else works on this (to split tracks A/B in Phase 1)
