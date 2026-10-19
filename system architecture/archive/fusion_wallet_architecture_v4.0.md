# Fusion Wallet (Kavachi guard layer) — Full Architecture v4.0

**Topic F · Agent Wallet · 2026 Shenzhen International FinTech Competition (International Track)**
Supersedes v3.0. Domain/contract version **V4.1**. Signer protocol **3.1.0**. `fusion-core` API **4.0**.

> **Status (2026-10-19):** This is the design and implementation contract. Nothing in it is claimed as built or tested unless Section 0.4 says so. Every threshold is a demonstration value that must pass the boundary and adversarial tests in Sections 18–19 before use, even on testnet.
>
> **Two deadlines drive this document.** **S = online submission, 2026-10-20 23:59 UTC+8 (15:59 UTC).** **F = Grand Final, 2026-11-21 to 11-23, Shenzhen.** Every feature is tagged **[S]** (must exist, really working, at submission), **[F]** (built for the finals; described honestly as roadmap in the submission), or **[S?]** (stretch for S, falls back to F without breaking anything).

---

## Contents

0. [Read me first: status, decisions, verified facts](#0-read-me-first)
1. [What changed from v3 and why](#1-what-changed-from-v3-and-why)
2. [Product and winning thesis](#2-product-and-winning-thesis)
3. [Scoring and deliverable map](#3-scoring-and-deliverable-map)
4. [Delivery cuts: S vs F](#4-delivery-cuts-s-vs-f)
5. [System architecture](#5-system-architecture)
6. [Authority model and permission tiers](#6-authority-model-and-permission-tiers)
7. [On-chain specification](#7-on-chain-specification)
8. [Obligations, payees and compliance](#8-obligations-payees-and-compliance)
9. [Hashing contract V4.1](#9-hashing-contract-v41)
10. [fusion-core v4 (Rust engine)](#10-fusion-core-v4-rust-engine)
11. [Android app, processes and MNN](#11-android-app-processes-and-mnn)
12. [AHEM integration](#12-ahem-integration)
13. [Signer protocol 3.1.0](#13-signer-protocol-310)
14. [Storage, durability and recovery](#14-storage-durability-and-recovery)
15. [Ops wallet and x402](#15-ops-wallet-and-x402)
16. [Accountant agent (read-only, optional cloud)](#16-accountant-agent)
17. [UX specification](#17-ux-specification)
18. [Evaluation and attack arena](#18-evaluation-and-attack-arena)
19. [Validation gates and execution evidence](#19-validation-gates-and-execution-evidence)
20. [Demo, video and deck](#20-demo-video-and-deck)
21. [Risks and known limits](#21-risks-and-known-limits)
22. [Repository layout, ownership and build order](#22-repository-layout-ownership-and-build-order)
23. [Open questions](#23-open-questions)
24. [Appendix A: Solidity interface](#24-appendix-a-solidity-interface)
25. [Appendix B: fusion-core FFI surface](#25-appendix-b-fusion-core-ffi-surface)
26. [Sources](#26-sources)

---

## 0. Read me first

### 0.1 How to read

| If you are… | Read |
|---|---|
| Pitching / judging | 2, 3, 17.1–17.4, 20, 21 |
| Building contracts | 6, 7, 8, 9, 24 |
| Building Rust (`fusion-core`) | 9, 10, 25, 14 |
| Building Android | 11, 13, 14, 16, 17 |
| Running evaluation / evidence | 18, 19 |
| Planning work | 4, 22, 23 |

**Normative words:** MUST / MUST NOT = required for the claimed profile. SHOULD = strong default; deviating needs a written reason. MAY = optional.

### 0.2 Naming

- **Fusion Wallet** = the product and Android app (`app.fusion.wallet`).
- **Kavachi** (from *kavach*, "armour") = the on-chain guard layer: `KavachiModule`, `KavachiDemoVault`, `KavachiTestSanctionsList`. Kavachi is never the app name in UI copy.

### 0.3 Decision log (binding unless the operator overrides)

| ID | Decision | Source / reason |
|---|---|---|
| D1 | Treasury asset = **Aave Base Sepolia test USDC** (`0xba50…4D5f`). Vault A = **Aave waUSDC ERC-4626 wrapper**. Vault B = our `KavachiDemoVault`. | Owner decision 2026-10-17 (`docs/ASSETS_AND_AAVE.md`). Real protocol integration is worth more on-chain bonus than two toy vaults. |
| D2 | **Fallback profile** with Circle testnet USDC + two `KavachiDemoVault`s, same code, chosen at deploy time. | Aave faucet access is still unverified (§23 Q1). The fallback must never block S. |
| D3 | **`OPS_TOPUP` removed from the module.** The x402 ops wallet holds **Circle USDC** (EIP-3009 capable) and is funded by owners directly. | x402 "exact" payments on EVM use EIP-3009 `transferWithAuthorization`. Aave test USDC is not known to support it. Removing the action shrinks the module's attack surface. |
| D4 | On-device planner model = **LFM2.5-230M (MNN build)**, as already shown in the app; must be pinned by hash. Swap only if it fails the frozen candidate tests on device. | `docs/UI_REFERENCE.md` §7.2. |
| D5 | **Process isolation [S]** = one APK with three Android processes (`:main`, `:inference`, `:signer`). **[F]** = three APKs with separate UIDs. | Three APKs + signature permissions is not buildable safely before S. Honest wording required (§21). |
| D6 | **Approver (Safe owner 2) [S]** = Safe{Wallet} web app or a Foundry script on a laptop holding the approver key. **[F]** = second phone running the app in APPROVER role. | Removes the second-device dependency from the S critical path. Verify Safe{Wallet} supports chain 84532 (§23). |
| D7 | Owners pay bills **through the module** (`ownerPayObligation`), never by raw Safe transfer. | Fixes the v3 double-payment defect (§1.2 G2). |
| D8 | Every agent action carries a **`decisionHash`** that is emitted on-chain. | Binds the AI decision record to the tx: verifiable explainability (§7.11). |
| D9 | The cloud **Accountant** stays: opt-in, default OFF, coarse brief only, egress guard, **no authority**. | Owner decision 2026-10-17 (`docs/ACCOUNTANT_AGENT.md`). The pitch line becomes "nothing private leaves the phone unless you opt in to a coarse summary". |
| D10 | App login (PBKDF2, demo accounts) is a **UI convenience, not authority**. Authority = keys + Safe threshold. Roles map: Owner → Safe owner 1, Finance Manager → Safe owner 2 (approver), Auditor → read-only Judge mode. | Avoids a false security claim in the self-assessment. |
| D11 | Base Sepolia payee cooling = **3,600 s** (contract minimum), labelled "demo setting; production default 24 h". | Contracts deploy less than 30 h before S. A 24 h cooling period would make the paid-bill evidence impossible. |
| D12 | All money is **raw integer units** (`U256` / `BigInteger` / base-10 strings). No `f64`/`Float`/`Double` money anywhere. | Carried v2 audit finding; current `fusion-core` still violates it (§10.1). |

### 0.4 What exists today (grounded 2026-10-19)

| Area | State | Gap to v4 |
|---|---|---|
| `src/fusion-core` (Rust, 3.6 kLOC) | v2 engine: free-form `transfer/swap/stake` intents, `f64` USD limits, raw private key passed across FFI (`execute_intent(json, private_key)`), file JSON ledger, mainnet USDC/WETH defaults | **Rewrite of the domain layer** (§10). Keep: strict serde parsing, zeroize discipline, panic guard, UniFFI setup, alloy deps, no-mock audit script |
| `src/Fintech_Wallet_Agent` (submodule, branch `fusion-v3-integration`) | Compose app: Obsidian bento UI, 3 tabs (Treasury / Agent / Safety), PBKDF2 login + RBAC, EN/中文, Accountant (local + DeepInfra cloud), `PreFlightGuardrailCard`, canvas charts, `DemoData.kt` | Wire to real chain state + v4 core; add Bills/Vendors; remove demo data from release; separate signer/inference processes |
| `src/mnn-android-sdk` (submodule) | MNN LLM Kotlin API, JNI, arm64 libs | Pin artifact; ABI-matched import check |
| `src/ahem` (submodule) | Research layer: grounding, TensorLogic bridge, rule packs | SHADOW integration only for S |
| Solidity | **None in the workspace** | `src/contracts/` Foundry project (§7, §22) |
| x402 data service | None | `src/x402-service/` [F, S?] |
| Safety tab claims "7 of 7 attacks blocked" | Shown from fixtures | MUST be generated from a real results file or labelled DEMO (§17.10) |

> The submodules were not checked out in the review environment. Their state above comes from `docs/UI_REFERENCE.md`, `docs/ACCOUNTANT_AGENT.md` and the v3 baseline table. Re-confirm before assigning work.

### 0.5 Verified external facts (and what is not verified)

| Fact | Status |
|---|---|
| Aave V3 Base Sepolia Pool `0x8bAB…aE27`, USDC `0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f`, aUSDC `0x10F1…0ACC`, waUSDC `0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a` (ERC-4626, `asset()` = USDC) | Verified by `cast call` at block 48,238,481 on 2026-10-17 |
| Aave USDC supply APR ≈ 0.21 % | Verified 2026-10-17. Tiny: **never pitch yield** |
| Circle Base Sepolia USDC `0x036CbD53842c5426634e7929541eC2318f3dCF7e` | From Circle docs (v3 sources, 2026-10-18) |
| x402 supports Base Sepolia; x402.org facilitator is testnet-only | From CDP docs (v3 sources) |
| Safe v1.4.1 deployed on 84532; Safe Transaction Service / Safe{Wallet} support 84532 | **UNVERIFIED — check `safe-deployments` before deploy** |
| Aave test-USDC faucet amounts and access | **UNVERIFIED (§23 Q1)** |
| Chainalysis sanctions oracle on Base | **Not confirmed**: testnet uses our labelled TEST LIST |

---

## 1. What changed from v3 and why

### 1.1 Reconciliation with owner decisions taken after v3

| v3 said | Owner decided | v4 |
|---|---|---|
| Circle USDC + two demo vaults | Aave test USDC + real Aave waUSDC (D1) | Aave primary, Circle fallback (D1, D2) |
| Model "smallest SDK-listed, e.g. Qwen2.5-0.5B" | LFM2.5-230M on MNN shown in the app | LFM2.5-230M pinned (D4) |
| LOCAL_ONLY; cloud only for a fixed public-topic enum | Opt-in cloud Accountant with coarse brief + egress guard | Kept as a read-only, no-authority component (§16, D9) |
| 5-tab UX (Home/Bills/Agent/Activity/More) | 3-tab Obsidian bento already built | 4 tabs: Treasury · Bills · Agent · Safety, with Activity inside Agent and settings in the Profile sheet (§17) |
| Tiers 0.10 / 1.00 / 25 USDC on Base Sepolia | UI shows 10 / 100 / 2,500 | One **FULL** profile (10 / 100 / 2,500) on Anvil and Base Sepolia-Aave; **SCALED** (÷100) only for the Circle fallback (§7.4) |

### 1.2 Defects found in v3 (fixed here)

| # | v3 defect | Consequence | v4 fix |
|---|---|---|---|
| G1 | `queueAction` "reserves the current nonce", but immediate actions also need `expectedNonce == actionNonce` | Undefined behaviour: either the queue is silently invalidated by an instant action, or both share a nonce | While a queue exists, `execute` accepts **only** the queued digest; anything else reverts `QueueBusy`. Stop actions still work (§7.8) |
| G2 | Owners "pay a bill" with a raw Safe transfer; `paid[]` and R are not updated | **The agent can pay the same bill a second time** before re-commitment | `ownerPayObligation` (Safe-only) uses the same Merkle leaf + `paid[]` path (§7.6, D7) |
| G3 | 24 h payee cooling on Base Sepolia | No paid-bill evidence possible before S | 3,600 s demo setting, labelled (D11) |
| G4 | OPS_TOPUP transfers the treasury token to the ops float, but x402 needs EIP-3009 USDC | With Aave USDC, x402 cannot work | OPS_TOPUP removed; ops wallet funded separately in Circle USDC (D3) |
| G5 | No link between the AI decision and the on-chain transaction | "Explainable" is only claimed off-chain | `decisionHash` in `Action`, emitted in `ActionExecuted` (D8, §7.11) |
| G6 | No one-step "get out" for owners | Common judge expectation (revoke + pull funds back) unmet | `emergencyUnwind` by any single owner or guardian: pause + revoke + redeem both vaults **into the Safe** (§7.10) |
| G7 | Three APKs, second phone, x402, AHEM ENFORCED, 30/30 eval all on the S critical path | S fails on breadth | S/F cut (§4) |
| G8 | Safety tab and Agent tab show fixture numbers as facts | Integrity risk with judges | Results-file-driven metrics or a DEMO label (§17.10) |
| G9 | No explicit contract invariants to fuzz | "Fuzz + invariant tests" without targets | Seven named invariants I1–I7 (§7.13) |
| G10 | `fusion-core` FFI shape not specified | Rust and Kotlin teams would invent incompatible APIs | Frozen FFI surface (Appendix B) |

### 1.3 Kept from v3 (unchanged in intent)

Safe smart account + module; 2-of-2 dual control for loosening; single owner/guardian can always stop; candidate-ID-only model output; deterministic integer candidate generation; three amount tiers with on-chain cancel window; Merkle-committed obligations; immutable payee registry with cooling and sanctions; durable journals and UNKNOWN broadcast state; AHEM OFF/SHADOW/ENFORCED; "no fake greens" UI; attack arena with layer attribution.

---

## 2. Product and winning thesis

### 2.1 One-line pitch
**Fusion Wallet is an AI treasury agent for small businesses. It pays the bills you approved, to vendors you verified, in the exact amount, and invests only the money you don't owe. The AI runs on your phone and can't go beyond the limits enforced on-chain.**

### 2.2 Problem (sourced, §26)
- **Invoice redirection / BEC:** FBI IC3 recorded about 24,768 BEC complaints and ~$3.05B in reported losses in 2025.
- **Address poisoning:** single incidents of ~$50M (Dec 2025) and ~$12.25M (Jan 2026).
- **AI agents with open send authority get talked into paying attackers:** in the May 2026 Grok/Bankr incident, a Morse-code prompt injection moved ~$150K–$200K.
- Idle operating cash earns nothing, but sweeping it risks touching money already owed.

### 2.3 Persona
A small cross-border service business (10–30 people) paid in USDC, paying contractors/SaaS/rent in USDC. Owner (Mei) + finance manager (Wei). No treasury team.

> Jurisdiction note: business stablecoin use is regulated differently per country. This is a testnet prototype; no regulatory approval is claimed.

### 2.4 Jobs the agent does

| # | Job | Cut |
|---|---|---|
| J1 | Pay committed bills on time, exact amount, verified payee only | [S] |
| J2 | Protect reserves: money owed + safety cushion is never invested or moved by the agent | [S] |
| J3 | Invest true surplus into Aave (waUSDC) or the demo vault; rebalance / withdraw when it beats fees or refills the cushion | [S] |
| J4 | Explain every decision; hold when not worth acting; anchor the decision hash on-chain | [S] |
| J5 | Stop instantly on risk (depeg signal, wrong chain, accounting mismatch); one-tap emergency unwind | [S] |
| J6 | Buy its own data (rates/risk) via x402 from a capped ops wallet | [F] / [S?] |
| J7 | Answer finance questions (Accountant: local by default, cloud opt-in) | [S] (exists) |

### 2.5 Winning factors (what we optimise for)

Each factor names the score bucket it serves and the proof a judge can check.

| # | Winning factor | Bucket | Judge-verifiable proof | Cut |
|---|---|---|---|---|
| W1 | **Bills-only payments**: the agent can only pay Merkle-committed bills to registered payees, exact amount, once | Task 40 + Bonus | Paid-bill tx + `ObligationPaid` event; duplicate-pay revert tx | S |
| W2 | **Anti-BEC vendor controls**: immutable payees, cooling period, independent last-6 check, lookalike detector, sanctions fail-closed | Security 30 | `PayeeNotActive` revert tx; UI flow on video | S |
| W3 | **Injection-proof by construction**: the LLM outputs only a candidate ID from a Rust-generated list, so it cannot produce an address or amount | Security 30 + Innovation 30 | Arena: Morse/base64 injection → HOLD or valid candidate only | S |
| W4 | **Four independent layers with attribution**: model check → Rust policy → signer → contract; the UI shows which layer stopped each attack | Innovation 30 | Arena table + revert traces | S |
| W5 | **On-chain decision passport**: every agent tx emits `decisionHash`, which equals the hash of the exported passport JSON | Bonus + Compliance | Re-hash a passport, compare with the event | S |
| W6 | **Real DeFi integration**: deposit into Aave V3 via the ERC-4626 wrapper, guarded by exposure caps and share-slippage bounds | Task 40 + Bonus | Deposit/withdraw tx into waUSDC | S (D1) / fallback demo vault |
| W7 | **Two-person control + cancel window**: maker-checker for loosening; 180 s cancellable queue for mid-size actions; any single human can stop | Security 30 | `approveAction` SafeTx with 2 signatures; queue + cancel txs | S |
| W8 | **One-tap emergency unwind**: pause + revoke + redeem all vault shares back into the Safe | Security 30 + Interaction | `EmergencyUnwind` tx | S |
| W9 | **Private on-device AI** (MNN, LFM2.5-230M) with the inference process holding no keys | Innovation 30 + Compliance | Model hash, device latency table, network capture | S (process), F (separate APK) |
| W10 | **Consented coarse cloud analyst** with a fail-closed egress guard and audit log | Compliance + Interaction | Egress log screen; guard unit tests | S (exists) |
| W11 | **Honest metrics**: ASR/FBR with counts and denominators, "not measured" where true, a "what we don't claim" list | Security (credibility) | Scorecard from the results file | S |
| W12 | **Bilingual EN / 中文 UI** for a WeBank panel | Interaction 30 | Live language switch | S (exists) |
| W13 | **Simulation before signing**: expected balance deltas from `eth_call` from the real sender | Interaction + Security | Decoded card "Before → After" rows | S |
| W14 | **Agent as economic participant**: pays for its own data via x402 from a capped float | Innovation (theme) | x402 settlement tx | F / S? |
| W15 | **Tensor-level monitoring (AHEM)** in SHADOW, with an honest ablation | Innovation (research) | AHEM scores in passports; ablation table | S (shadow), F (enforced, if gates pass) |
| W16 | **Naive-wallet comparator**: same attacks against an unprotected agent wallet on Anvil, showing simulated loss | Innovation (story) | Side-by-side arena run | F / S? |

### 2.6 Scope

| In | Out |
|---|---|
| Base Sepolia (public evidence), Anvil (fault injection, arena, boundaries) | Mainnet funds, real customer money |
| One managed asset: USDC (Aave test USDC, or Circle in the fallback) | Volatile assets, agent swaps, bridges, FX |
| Safe v1.4.1 account + `KavachiModule`; Aave waUSDC; `KavachiDemoVault` | Upgradeable proxies, ERC-4337 paymaster |
| Owner manual send/swap via Safe (owner sovereignty) | Agent arbitrary send/swap/approve |
| On-device planning; opt-in coarse cloud Accountant | Cloud LLM with any private data or any authority |

---

## 3. Scoring and deliverable map

### 3.1 Rubric → features → evidence

| Criterion | What the judge must be able to verify | Features | Evidence artefact |
|---|---|---|---|
| Task completion (40) | The agent completes real financial tasks end to end | J1–J5; tiers; reconciliation | Base Sepolia tx hashes per task; reconciled ledger; video |
| Security & compliance (30) | The agent cannot exceed its authority even if the model or app is compromised; fraud controls; audit trail; privacy | W1–W5, W7–W11 | Revert traces, arena results, security self-assessment, permission-tier table, egress log |
| Innovation & interaction (30) | Novel, usable, trustworthy UX | W3–W5, W9, W12–W16 | Device demo, screenshots, scorecard |
| On-chain bonus (0–5) | Ambitious, verifiable on-chain behaviour | Safe module, Merkle bills, payee registry, cancel queue, Aave, on-chain decision hashes, unwind | Verified contracts + receipts in the judge bundle |

### 3.2 Submission items → source

| # | Item | Built from | File (to create) | Cut |
|---|---|---|---|---|
| 01 | Technical documentation (architecture, core algorithm, security design) | §5–§16 of this file, condensed; core algorithm = §10.4 candidates + §7.7 checks | `docs/submission/01_TECHNICAL.md` (+PDF) | S |
| 02 | Presentation deck ≤ 10 min | §20.3 | `docs/submission/02_DECK.pptx` | S |
| 03 | Demo video ≤ 5 min (optional, but we do it) | §20.2 | link in README | S |
| 04 | Source code + README + GitHub link + deployment + tests | §22 | root `README.md`, `DEPLOY.md`, `TESTING.md` | S |
| 05 | Security self-assessment (permission tiers + known risks) | §6, §21 | `docs/submission/05_SECURITY_SELF_ASSESSMENT.md` | S |
| 06 | Execution evidence (real tx hashes) | §19.4 | `docs/submission/06_EVIDENCE.md` + `evidence/base-sepolia.json` | S |

---

## 4. Delivery cuts: S vs F

### 4.1 Principle
The submission is scored by people reading documents and checking hashes. It must be **complete, true and verifiable**, not maximal. Anything not working at S is described as roadmap, never as built.

### 4.2 Feature × cut matrix

| Feature | [S] submission | [F] finals |
|---|---|---|
| `KavachiModule` + vaults + sanctions list, Foundry unit/fuzz/invariant tests | Full | Hardening, gas report |
| Base Sepolia deployment, verified source | Full | Redeploy if the spec changes |
| Evidence tx list (§19.4) | All S rows | All rows |
| `fusion-core` v4: integer math, hashing vectors, Merkle builder, candidates, preview, strict planner-output validation | Full | — |
| Android: real chain snapshot, buckets, bills/vendors, decoded cards, agent loop on device | Full core path | Polish |
| Signer | `:signer` **process** in the same APK, Keystore-wrapped keys, typed requests | Separate APK/UID, signature permission |
| Inference | `:inference` **process**, no network code, capture test | Separate APK with no INTERNET permission |
| Approver | Safe{Wallet} web or laptop script (D6) | Second phone, QR/Safe Tx Service hand-off |
| AHEM | SHADOW (scores recorded) or OFF with honest label | ENFORCED only if §12 gates pass |
| x402 ops wallet + data service | S? (only after all S rows are green) | Full |
| Arena | Contract + Rust layers automated on Anvil; ≥ 3 intentional reverts on Base Sepolia; ≥ 4 scenarios run through the real model on device | 13 scenarios on device + naive comparator |
| Evaluation corpus | Frozen 30/30 corpus; Rust/contract layers fully run; model layer run as far as time allows, with real counts | 3 repeats, full ablation |
| UX | Treasury, Bills, Vendors, Agent (feed + cards + queue + stop), Safety (real results), Judge mode | Onboarding wizard, notifications, approver inbox |

### 4.3 S critical path (dependency order, no clock assumptions)

1. Freeze contracts: Appendix A + §9 hashing (Phase 1).
2. Foundry tests for I1–I7 and every revert in §7.7 (Phase 2).
3. Implement the module → deploy to **Base Sepolia immediately** → add payees (cooling starts) → commit bills.
4. In parallel: `fusion-core` v4 against the frozen ABI + hashing vectors.
5. Android wiring to the real snapshot + core + signer process.
6. Run the evidence list; agent-path txs come from the phone pipeline (fallback rule §19.5).
7. Write submission docs 01/05/06 from this file + the evidence JSON; deck; video.

---

## 5. System architecture

### 5.1 Components

| Component | Process / identity | Responsibility | Never does |
|---|---|---|---|
| **Fusion app UI** (`:main`) | `app.fusion.wallet` main process; network for pinned RPC (+ cloud Accountant if opted in) | Compose UI, bills/payees DB, orchestration worker, reconciliation, evidence, Accountant | Hold keys, sign, run the model |
| **Inference** (`:inference`) | Separate process [S] / separate APK, no INTERNET [F] | MNN + LFM2.5-230M, AHEM, candidate selection | Hold keys; open sockets; see private invoice text unless the owner asked for a local parse |
| **Signer** (`:signer`) | Separate process [S] / APK [F]; pinned RPC | Keys (owner, agent session, guardian; ops wallet [F]); trusted review screen; nonce/broadcast journal; independent re-checks | Accept free-form calldata, generic "sign this", or "approved=true" flags |
| **fusion-core** (Rust `.so`) | Loaded in `:main` (pure functions) and `:signer` (encode/sign/broadcast) | Integer finance, hashing, Merkle, candidates, preview, planner-output validation, ABI encoding, snapshot reads | Hold long-lived keys; choose actions |
| **Approver** | Safe{Wallet} / laptop script [S]; second phone [F] | Second Safe signature | — |
| **Safe account** | on-chain | Holds USDC and vault shares; owners transact normally | — |
| **KavachiModule** | on-chain, the only module enabled | Agent authority: tiers, limits, reserves, queue, bills, payees, decision anchoring, unwind | Arbitrary calls, delegatecall, value transfers |
| **Vault A** (Aave waUSDC) / **Vault B** (`KavachiDemoVault`) | on-chain ERC-4626 | Yield targets | — |
| **Sanctions list** | on-chain `isSanctioned` | Screening at payee add, queue, pay | — |
| **Accountant cloud** (DeepInfra) | external, opt-in | Coarse advisory text | Receive private data; reach any tool |
| **x402 data service** [F] | team-operated HTTP, Base Sepolia | Paid rate/risk snapshots | Touch the treasury |

### 5.2 Diagram

```mermaid
flowchart TD
  subgraph Phone["Android device"]
    UI[":main — UI, DB, worker, fusion-core (pure)"]
    INF[":inference — MNN LFM2.5-230M + AHEM (no keys, no network)"]
    SIG[":signer — keys, review screen, journal, fusion-core (encode/sign)"]
  end
  UI -->|"candidate facts + bounded goal"| INF
  INF -->|"candidateId | HOLD + AHEM report"| UI
  UI -->|"typed request (exact Action + proof + decisionHash)"| SIG
  SIG -->|"session-key tx"| MOD["KavachiModule"]
  MOD -->|"execTransactionFromModule (allowlisted calls)"| SAFE["Safe (2-of-2)"]
  SAFE --> VA["Vault A: Aave waUSDC"]
  SAFE --> VB["Vault B: KavachiDemoVault"]
  SAFE -->|"PAY_OBLIGATION"| PAYEE["ACTIVE payee"]
  APPR["Approver (Safe{Wallet} / 2nd phone)"] -->|"2nd SafeTx signature"| SAFE
  MOD -->|"isSanctioned"| SL["Sanctions list (TEST)"]
  SAFE -. "events + state" .-> UI
  UI -. "opt-in coarse brief" .-> CLOUD["Accountant cloud (advisory only)"]
  OPS["Ops wallet (Circle USDC) [F]"] -->|"x402"| DATA["Data service [F]"]
```

### 5.3 Trust boundaries

| Boundary | Crossing data | Enforcement |
|---|---|---|
| User / documents → model | Untrusted goal text, invoice text | Model can only emit a candidate ID or HOLD; strict parser |
| Model → app | `candidateId` | Must equal one ID in the current candidate set; else one retry then HOLD |
| App → signer | Typed request | Signer reloads config, re-reads chain, recomputes digests, re-runs checks, simulates |
| Signer → chain | Signed tx | Module re-checks everything on-chain; Safe executes only module-built calls |
| App → cloud | Coarse brief | Egress guard, consent version, daily cap |
| Data feed → app | Rates, prices | Untrusted: may only halt or re-rank, never add actions/addresses/amounts |

### 5.4 Runtime loop (agent mode)

1. **Setup (owners):** Safe [owner, approver], threshold 2, module enabled, policy configured, payees added, bills committed. Signer starts **LATCHED**.
2. **Shift start:** Safe `startSession` + `resume(expectedEpoch)` (2-of-2).
3. **Perceive:** one-block snapshot (§10.5). Missing reads → no snapshot.
4. **Enumerate:** Rust builds ≤ 7 feasible candidates + HOLD with integer math (§10.4).
5. **Choose:** the model picks a candidate ID or HOLD; AHEM report attached.
6. **Check:** Rust preview → signer independent checks + `eth_call` simulation → on-chain module checks.
7. **Route by tier:** immediate / queued (cancellable) / owner approval (2-of-2).
8. **Execute + reconcile:** signer broadcasts stored bytes; app reconciles receipt + events + balance deltas + `decisionHash`.
9. **Explain:** decoded card + passport; HOLD is a valid, shown outcome.

Critical risk (depeg ≤ 0.98 signal, wrong chain, R mismatch, code-hash mismatch) skips the model: persist latch → block signing → guardian `pause` → explain afterwards.

---

## 6. Authority model and permission tiers

### 6.1 Roles

| Role | App login role (D10) | Key location | Can | Cannot |
|---|---|---|---|---|
| **Owner** (Safe owner 1) | Owner (`mei@`) | `:signer`, biometric unwrap | Sign Safe txs; single-owner stop actions; emergency unwind | Act alone on threshold-2 operations |
| **Approver** (Safe owner 2) | Finance Manager (`wei@`) | Safe{Wallet}/laptop [S]; 2nd phone [F] | Second signature; single-owner stop actions; emergency unwind | Act alone on threshold-2 operations |
| **Safe (2-of-2)** | — | on-chain | Configure policy, commit bills, add payees, start/resume/revoke session, approve large actions, owner bill payment, any normal Safe tx | — |
| **Agent session key** | — | `:signer`, auto-unwrap only inside a live session | `execute`, `queueAction` within policy | Send anywhere else, change policy, approve itself, resume after pause |
| **Guardian** | — | `:signer`, separate key | `pause`, `cancelQueuedAction`, `cancelPayee`, `emergencyUnwind` | Resume, approve, configure, move funds out of the Safe |
| **Auditor** | Auditor (`judge@`) | none | Read-only Judge mode | Anything that signs |
| **Ops wallet** [F] | — | `:signer`, separate key | Sign x402 payments from its own balance | Touch Safe funds |

**Rule:** a single human (or the guardian) can only **stop or pull back into the Safe**. Starting, loosening or paying out to anyone needs the Safe (2-of-2) or the bounded agent path.

### 6.2 Amount tiers (per action; `assets` in USDC)

| Range (FULL profile; SCALED ÷100) | AUTONOMOUS | OWNER_APPROVED |
|---|---|---|
| ≤ 10 | **Instant** | Safe approval, then instant |
| > 10 and ≤ 100 | **Queue → 180 s → execute** (cancellable) | Safe approval + mandatory queue |
| > 100 and ≤ 2,500 | **Safe approval (2-of-2)**, then execute | Same |
| > 2,500 | Always reverts | Always reverts |

READ_ONLY: no agent execution at all.

### 6.3 Cumulative limits

| Limit | FULL | Enforced by |
|---|---|---|
| Daily gross agent outflow (UTC day, chain time) | 4,000 | Module (+ Rust/signer mirrors) |
| Session budget | 8,000, ≤ 24 h | Module |
| Vault exposure | A: 6,000 · B: 6,000 | Module |
| Share slippage | 50 bps | Module |
| Payee cooling | 3,600 s Base Sepolia demo (D11); 86,400 s default | Module |
| Early-pay window | 3 days before `dueAt` | Module |

### 6.4 Payments add rules on top of tiers
The bill is committed, unpaid, and inside its pay window. The payee is ACTIVE and not sanctioned. Recipient = the registry address. Amount = the committed amount exactly.

### 6.5 Permission matrix (for the self-assessment, deliverable 05)

| Operation | Agent key | Single owner | Guardian | Safe 2-of-2 | Anyone |
|---|---|---|---|---|---|
| execute / queueAction | ✔ (policy-bound) | ✖ | ✖ | ✖ | ✖ |
| configurePolicy, commitObligations, addPayee | ✖ | ✖ | ✖ | ✔ | ✖ |
| startSession, resume, revokeSession | ✖ | ✖ | ✖ | ✔ | ✖ |
| approveAction / revokeActionApproval | ✖ | ✖ | ✖ | ✔ | ✖ |
| ownerPayObligation | ✖ | ✖ | ✖ | ✔ | ✖ |
| pause, cancelQueuedAction, cancelPayee | ✖ | ✔ | ✔ | ✔ | ✖ |
| emergencyUnwind (funds stay in the Safe) | ✖ | ✔ | ✔ | ✔ | ✖ |
| expireQueuedAction (after expiry) | ✔ | ✔ | ✔ | ✔ | ✔ |
| Normal Safe tx (send/swap) | ✖ | ✖ | ✖ | ✔ | ✖ |
| disableModule | ✖ | ✖ | ✖ | ✔ | ✖ |

### 6.6 Why owners keep send/swap
Owners are sovereign via normal 2-of-2 Safe transactions. The module's rules bind the **agent**, not owners. If owners move reserved cash, the app shows a **shortfall** and the module blocks agent investing until it is fixed.

---

## 7. On-chain specification

### 7.1 Contracts

| Contract | Source | Notes |
|---|---|---|
| Safe v1.4.1 singleton, proxy factory, MultiSend, fallback handler | Official deployments (verify on 84532; deploy via the singleton factory on Anvil) | Owners [owner, approver], threshold 2 |
| `KavachiModule` | NEW | The only enabled module. No proxy, no delegatecall, no generic execute |
| Vault A | **Aave waUSDC** `0xf430…B32a` (primary) / `KavachiDemoVault` (fallback) | Pinned by address + runtime code hash |
| `KavachiDemoVault` (Vault B) | NEW, OpenZeppelin ERC4626 over the treasury USDC | Yield from a labelled sponsor drip (`drip()` by the deployer). **Simulated yield.** Also used as the "attacker vault" fixture on Anvil |
| `KavachiTestSanctionsList` | NEW, testnet only | `isSanctioned(address)`; owned by the Safe; labelled TEST LIST |
| `MockUSDC` | NEW, Anvil only | 6 decimals; EIP-3009 for x402 tests |
| `NaiveAgentWallet` | NEW, **Anvil only**, labelled | Deliberately unprotected comparator for the arena (W16) |

Toolchain: Solidity `0.8.24` (pinned, not `^`), OpenZeppelin Contracts v5 (pinned tag), Foundry (pinned `foundry.toml`), `via_ir = false`, optimizer 200. Every deployment writes `deployments/<chainId>.json` with addresses, runtime code hashes, block numbers and the profile name.

### 7.2 Module immutables
`safe`, `asset` (decimals == 6 checked), `deploymentChainId`, `vaultA`, `vaultB` (distinct, `asset()` == asset, non-empty code), `vaultCodeHashA/B`, `sanctionsList`. Constructor rejects zero addresses, the asset used as a vault, and identical vaults.

### 7.3 How the module moves funds
Every financial effect goes through `ISafe.execTransactionFromModule(to, 0, data, Operation.Call)`.

| Purpose | Exact call(s) executed by the Safe |
|---|---|
| Deposit | `asset.approve(vault, assets)` → `vault.deposit(assets, safe)` → `asset.approve(vault, 0)` |
| Withdraw | `vault.withdraw(assets, safe, safe)` |
| Rebalance | Withdraw leg, then deposit leg, atomically in one module call |
| Pay bill (agent or owner) | `asset.transfer(recipient, assets)` |
| Emergency unwind | `vault.redeem(shares, safe, safe)` per vault (each wrapped so one failing vault doesn't block the other) |

Rules: value is always 0; operation is always `Call`; every `execTransactionFromModule` result MUST be `true`; token return data, if present, MUST decode to `true`; observed balance and share deltas MUST match expectations; residual allowance MUST be 0 after deposit legs.

### 7.4 Policy (V4.1)

| Field | Type | FULL (Anvil, Base Sepolia-Aave) | SCALED (Circle fallback) | Validation |
|---|---|---|---|---|
| permissionTier | enum | AUTONOMOUS | AUTONOMOUS | READ_ONLY / OWNER_APPROVED / AUTONOMOUS |
| operatingBufferRaw | uint256 | 500 | 5 | — |
| microMaxRaw | uint256 | 10 | 0.10 | > 0 |
| delayedMaxRaw | uint256 | 100 | 1.00 | ≥ micro |
| hardMaxRaw | uint256 | 2,500 | 25 | ≥ delayed |
| dailyLimitRaw | uint256 | 4,000 | 40 | ≥ hard |
| delaySeconds | uint32 | 180 | 180 | 120–900 |
| maxShareSlippageBps | uint16 | 50 | 50 | ≤ 1,000 |
| guardian | address | — | — | ≠ 0, ≠ owners, ≠ session key, ≠ Safe |
| vaults[] (vault, maxExposureRaw) | array | A 6,000 · B 6,000 | A 60 · B 60 | ⊆ {vaultA, vaultB}, sorted by address, cap > 0 |
| payeeDelaySeconds | uint32 | 3,600 (Base Sepolia demo) / 86,400 | 3,600 | 3,600–604,800 |
| payEarlyWindowSeconds | uint32 | 259,200 | 259,200 | ≤ 2,592,000 |

Session limit at `startSession`: 8,000 (FULL) / 80 (SCALED). Amounts are in USDC; code uses raw units (×10⁶).

Reference state (FULL): cash C = 5,000, reserved R = 1,500, cushion B = 500, Vault A = 3,000, Vault B = 2,000 → free 3,500, sweepable 3,000, total 10,000.

> Demonstration parameters, not advice. Section 18 must exercise every boundary first.

### 7.5 Lifecycle functions

| Function | Caller | Effect |
|---|---|---|
| `configurePolicy(Policy)` | Safe | Validate; pause; clear session + queue (consumes nonce); `policyEpoch++` |
| `commitObligations(root, reservedRaw, count)` | Safe | Set root + declared total; pause; clear session + queue; `policyEpoch++` |
| `addPayee(payeeIdHash, account, labelHash)` | Safe | New immutable entry; `activeAt = now + payeeDelaySeconds`; reverts if sanctioned or already exists |
| `cancelPayee(payeeIdHash)` | Safe, any owner, guardian | Terminal CANCELLED; invalidates a queue that references it |
| `startSession(key, expiresAt, limitRaw)` | Safe | Requires configured + paused + no session; expiry ≤ now + 24 h; stays paused |
| `resume(expectedEpoch)` | Safe | Exact epoch; live session; unpause; `policyEpoch++` |
| `pause(reasonCode)` | Safe, any owner, guardian | First call pauses, clears queue, `policyEpoch++`; repeats are idempotent no-ops |
| `revokeSession()` | Safe | Pause; clear session + queue; `policyEpoch++` |
| `approveAction(Action, validUntil)` | Safe | Digest-bound approval for owner tier (and all tiers in OWNER_APPROVED); `validUntil ≤ now + 3,600` |
| `revokeActionApproval(digest)` | Safe | Clears one approval |
| `queueAction(Action, proof)` | Session key | Delayed tier only; no queue may exist; stores digest |
| `cancelQueuedAction(digest)` | Safe, any owner, guardian | Clear queue; consume nonce |
| `expireQueuedAction()` | Anyone, after `expiresAt` | Clear queue; consume nonce |
| `execute(Action, proof)` | Session key | All checks (§7.7), then effects |
| `ownerPayObligation(obligationIdHash, payeeIdHash, assets, dueAt, proof)` | **Safe** | Same proof/paid/payee/sanctions/recipient checks as PAY; no tier, window or session checks; `paid = true`, `R −= assets`; emits `ObligationPaid(byOwner=true)` (fixes G2) |
| `emergencyUnwind(reasonCode)` | Safe, any owner, guardian | `pause` semantics + clear session + redeem all shares of both vaults into the Safe; per-vault result emitted (§7.10) |

"Any owner" is checked live with `ISafe.isOwner(msg.sender)`.

### 7.6 Actions

| Kind | sourceVault | destinationVault | minSharesOut | maxSharesBurn | obligationIdHash / payeeIdHash / recipient / dueAt |
|---|---|---|---|---|---|
| DEPOSIT | 0 | allowed vault | > 0 | 0 | all zero |
| WITHDRAW | allowed vault | 0 | 0 | > 0 | all zero |
| REBALANCE | allowed vault | other allowed vault | > 0 | > 0 | all zero |
| PAY_OBLIGATION | 0 | 0 | 0 | 0 | all set; recipient = registry address |

Every Action also carries `expectedNonce`, `policyEpoch`, `deadline` and **`decisionHash`** (≠ 0 for agent actions; §7.11). HOLD is a planner outcome only; it never signs.

### 7.7 Checks in `execute` (all before any external call; order is normative)

1. Chain ID = deployment chain; module enabled on the Safe; not paused; caller = session key; session live; tier ≠ READ_ONLY.
2. **Queue gate:** if a queue exists, the action digest MUST equal the queued digest, else `QueueBusy`.
3. `expectedNonce == actionNonce`; `policyEpoch` current; deadline valid (immediate ≤ now + 120 s; approved ≤ now + 900 s; queued per §7.8; all ≤ session expiry); `decisionHash ≠ 0`.
4. Shape valid for the kind (§7.6); `assets > 0`; `assets ≤ hardMax`.
5. UTC-day gross + assets ≤ dailyLimit; session gross + assets ≤ sessionLimit.
6. Tier route satisfied: instant; or the matching queue is ready; or a live approval for this digest exists (and the queue too when both apply).
7. Kind-specific:
   - **DEPOSIT:** `C − assets ≥ R + B`; destination exposure after ≤ cap; `previewDeposit` within slippage and ≥ minSharesOut; vault code hash matches.
   - **WITHDRAW:** `maxWithdraw` sufficient; shares burned ≤ maxSharesBurn.
   - **REBALANCE:** `C ≥ R` (not underfunded); both legs within bounds; destination exposure ≤ cap.
   - **PAY_OBLIGATION:** Merkle proof valid for (obligationIdHash, payeeIdHash, assets, dueAt) under the current root; `!paid[id]`; `now ≥ dueAt − payEarlyWindow`; payee ACTIVE (`now ≥ activeAt`, not cancelled); `recipient == payees[payeeIdHash].account`; `!isSanctioned(recipient)`; `C ≥ assets`; `assets ≤ R`.
8. Effects via the Safe (§7.3); verify deltas; then `actionNonce++`, usage += assets; for PAY: `paid[id] = true`, `R −= assets`; clear the consumed queue/approval; emit `ActionExecuted` (with `decisionHash`).

Underfunded (`C < R`): only WITHDRAW and PAY_OBLIGATION are allowed. A sanctions call that reverts MUST fail closed (`SanctionsCheckFailed`). The module is `nonReentrant` on every state-changing function.

### 7.8 Queue (cancel window)
- At most one queue. `queueAction` runs all hard checks of §7.7 except the "ready" condition (including proof, payee status and sanctions) and stores the digest, the exact action and the session ID. `readyAt = now + delaySeconds`; `expiresAt = action.deadline` where `now + delay + 30 ≤ deadline ≤ now + delay + 900` and ≤ session expiry.
- The queued action pins `expectedNonce = actionNonce`. **While the queue exists, no other agent action can execute (`QueueBusy`)**, so the nonce cannot drift (fixes G1).
- No funds move at queue time; usage is not charged.
- Executing a delayed-tier action without its matching ready queue reverts `QueueRequired`, even with an approval.
- At execution, all checks rerun against current state. If anything moved out of bounds, it reverts; the app cancels or lets it expire and re-plans with new consent.
- Cancel / expire / invalidate consume the nonce, so the same digest can never execute later.

### 7.9 Reserve accounting

```
C = asset.balanceOf(safe)       R = reservedRaw        B = operatingBufferRaw
V = Σ vault.convertToAssets(vault.balanceOf(safe))     (marked, not guaranteed cash)

if C ≥ R: free = C − R ; reserved = R ; sweepable = max(C − R − B, 0) ; shortfall = 0
else    : free = 0     ; reserved = C ; sweepable = 0                 ; shortfall = R − C
deployed = V ; total = C + V
```

- Paying a bill lowers C and R by the same amount, so free cash is unchanged.
- The module trusts the declared R (set by 2-of-2). The app MUST recompute R from its bill records and **refuse to plan or sign if they differ** (`ACCOUNTING_MISMATCH` latch).

### 7.10 Emergency unwind (W8)
`emergencyUnwind(reason)` callable by the Safe, any owner or the guardian:
1. Pause (idempotent), clear the queue (consume nonce), clear the session, `policyEpoch++`.
2. For each vault with shares > 0: `redeem(allShares, safe, safe)` through the Safe; on failure emit `UnwindLegFailed(vault)` and continue.
3. Emit `EmergencyUnwind(actor, reason, cashAfter)`.

Funds only ever move **into** the Safe, so a single-signer call cannot cause loss (worst case: lost yield, which is griefing only). Restart needs the Safe (`startSession` + `resume`).

### 7.11 Decision anchoring (W5)
- `decisionHash = keccak256(canonical passport JSON)` (§10.8), computed before signing and included in the Action, so it is part of the digest.
- `ActionExecuted` and `ActionQueued` emit it. Judge mode re-hashes the exported passport and shows "Matches on-chain ✔".
- HOLD decisions are not anchored individually. [F] MAY add a daily `anchorDecisions(root)` by the session key (no funds, event only).

### 7.12 Events and errors
Full list in Appendix A. Additions over v3: `QueueBusy`, `MissingDecisionHash`, `EmergencyUnwind`, `UnwindLegFailed`, `ObligationPaid.byOwner`. Removed: `OpsTopUp`, `OpsCapExceeded`.

### 7.13 Invariants (Foundry invariant tests, handler-based, recorded seeds)

| ID | Invariant |
|---|---|
| I1 | After any agent action except PAY and WITHDRAW, `C ≥ R + B` held before → still holds after; DEPOSIT/REBALANCE never execute while `C < R` |
| I2 | Treasury asset leaves the Safe via the module only to vaultA, vaultB, or an ACTIVE, non-sanctioned payee address matching a committed unpaid leaf |
| I3 | Per UTC day, Σ agent `assets` ≤ dailyLimit; per session ≤ sessionLimit |
| I4 | Each `obligationIdHash` is paid at most once (agent or owner path) |
| I5 | Allowance of the Safe to any vault is 0 at the end of every transaction |
| I6 | While paused, no `execute`/`queueAction` succeeds; only the Safe can unpause |
| I7 | `actionNonce` strictly increases; any digest executes at most once; queue cancel/expire consumes the nonce |

### 7.14 What the module does NOT protect
- Owners acting together (threshold 2): intended sovereignty.
- External vault bugs, Aave testnet pauses/freezes, illiquidity, token upgrades.
- Gas spent on reverted transactions; agent-key gas runway.
- A compromised signer during an owner signing session.
- Censorship or delayed inclusion of a pause transaction.
- Genuineness of real-world invoices (a commitment proves owner approval, not truth).

---

## 8. Obligations, payees and compliance

### 8.1 Obligation (bill) record

| Field | Where | Notes |
|---|---|---|
| obligationId (opaque ASCII ≤ 64) | app DB | `obligationIdHash = keccak256(UTF8(id))` |
| payeeIdHash | app DB + leaf | references the registry |
| amountRaw | app DB + leaf | exact raw units |
| dueAt | app DB + leaf | unix seconds, uint64 |
| label, invoice no., attachment | app DB only | never on-chain; never to the cloud |
| status | app DB | DRAFT → COMMITTED → (SCHEDULED) → PAID / CANCELLED |

### 8.2 Commitment (Merkle)
- Leaf = `keccak256(bytes.concat(keccak256(abi.encode(bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 amountRaw, uint64 dueAt))))`. Compatible with OpenZeppelin `StandardMerkleTree` and `MerkleProof` (sorted pairs).
- Only unpaid COMMITTED bills are included; the builder rejects duplicates and paid IDs; leaves are sorted by leaf hash.
- `reservedRaw` = checked sum. Empty set: `root = keccak256("KAVACHI_EMPTY_OBLIGATIONS_V4")`, R = 0.
- Commit is a 2-of-2 Safe tx showing count, total and the change list since the last commitment.
- **Single-leaf tree:** root = leaf, proof = [] (tested explicitly).

### 8.3 Payee registry
- Entry: `account`, `labelHash`, `activeAt`, `cancelled`. **Immutable**: an address change is a new payee with a new cooling period.
- Cooling: not payable before `activeAt`. Any single owner or the guardian can cancel at any time.
- Sanctions: checked at add, queue and pay.
- Display: label + full address in 4-character groups, first/last 6 bold, identicon. Adding a payee requires typing the last 6 characters from an **independent** channel.

### 8.4 Lookalike / address-poisoning detector (app-side) [S]
On add, compare against every address in the payee table and the Safe's transfer history (from events since the deployment block): if `prefix4` and `suffix4` match but the addresses differ → red block "This looks like an address you've used before but is different." Also warn on an address first seen in an inbound dust transfer.

### 8.5 Sanctions screening
`isSanctioned(address) → bool` (Chainalysis-oracle shape). Testnet: `KavachiTestSanctionsList`, owned by the Safe, labelled TEST LIST everywhere. Production: the Chainalysis oracle must be verified per chain. Fail-closed.

### 8.6 Audit trail and compliance export
Every decision (including HOLD) produces a **decision passport** (§10.8). Exports: CSV (accountant), JSON (machine), and a **judge bundle** (§19.4). Private labels go only into the owner's local export.

---

## 9. Hashing contract V4.1

All hashes are Keccak-256 of Solidity `abi.encode` (not packed, not JSON) unless stated. `TAG(x) = keccak256(UTF8(x))`. V2/V3/V4.0 digests MUST be rejected.

```
PolicyHash = keccak256(abi.encode(TAG("KAVACHI_POLICY_V4_1"),
  uint8 permissionTier, uint256 operatingBufferRaw, uint256 microMaxRaw,
  uint256 delayedMaxRaw, uint256 hardMaxRaw, uint256 dailyLimitRaw,
  uint32 delaySeconds, uint16 maxShareSlippageBps, address guardian,
  bytes32 vaultLimitsHash, uint32 payeeDelaySeconds, uint32 payEarlyWindowSeconds))

vaultLimitsHash = keccak256(abi.encode((address,uint256)[] sortedVaultLimits))

SessionId = keccak256(abi.encode(TAG("KAVACHI_SESSION_V4_1"), uint256 chainId,
  address safe, address module, uint256 sessionStartEpoch, address sessionKey,
  uint64 sessionExpiresAt, uint256 sessionLimitRaw))

MandateHash = keccak256(abi.encode(TAG("KAVACHI_MANDATE_V4_1"), uint256 chainId,
  address safe, address module, uint256 policyEpoch, bytes32 policyHash,
  bytes32 obligationsRoot, uint256 reservedRaw, bytes32 sessionId,
  address sessionKey, uint64 sessionExpiresAt, uint256 sessionLimitRaw, bool paused))

Action = (uint8 kind, address sourceVault, address destinationVault, uint256 assets,
  uint256 minSharesOut, uint256 maxSharesBurn, bytes32 obligationIdHash,
  bytes32 payeeIdHash, address recipient, uint64 dueAt,
  uint256 expectedNonce, uint256 policyEpoch, uint64 deadline, bytes32 decisionHash)

ActionDigest = keccak256(abi.encode(TAG("KAVACHI_ACTION_V4_1"), uint256 chainId,
  address safe, address module, bytes32 mandateHash, bytes32 sessionId, Action action))

CandidateId = keccak256(abi.encode(TAG("KAVACHI_CANDIDATE_V4_1"),
  bytes32 snapshotBlockHash, bytes32 actionDigestWithoutDecision))
  -- actionDigestWithoutDecision = ActionDigest computed with decisionHash = 0
  -- (the decision is made after candidates exist; the ID must not depend on it)

ObligationLeaf = keccak256(bytes.concat(keccak256(abi.encode(
  bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 amountRaw, uint64 dueAt))))

DecisionHash = keccak256(UTF8(canonicalPassportJson))   -- RFC 8785 JCS canonical JSON
```

- No session: sessionId = 0, key = 0, expiry = 0, limit = 0.
- Merkle proofs are witnesses, not part of the digest; the leaf fields are.
- Owner approvals are Safe transactions: owners sign the Safe's EIP-712 `SafeTx` hash. `ActionDigest` is not EIP-712.
- Transport: uint256 as base-10 strings without sign or leading zeros (reject ≥ 2²⁵⁶); hex lowercase, `0x`-prefixed; addresses checked and returned EIP-55.
- **Reference vectors [S]:** `spec/vectors/v4_1.json` with ≥ 6 vectors (one per action kind + empty-commitment + single-leaf proof), checked by Foundry (`HashVectors.t.sol`), Rust (`tests/vectors.rs`) and Kotlin (JVM test). The vector file is the Phase-1 frozen contract.

---

## 10. fusion-core v4 (Rust engine)

### 10.1 Migration from the current crate

| Current (v2) | Fate | Reason |
|---|---|---|
| `models::AiIntent { action: Transfer/Swap/Stake, amount: f64, destination_address }` | **Delete** | Free-form destination + float money; violates W3/D12 |
| `execute_intent(json, private_key)` FFI | **Delete** | Raw key crosses FFI into the UI process |
| `guardrails.rs` f64 USD limits, file JSON ledger | **Delete** (keep test ideas: drip drain, day rollover, corrupt ledger fail-closed) | Limits now live on-chain + integer mirrors |
| `config.rs` mainnet USDC/WETH, `FUSION_*` env | **Replace** with deployment manifest (§10.3) | Wrong chain defaults |
| `executor.rs` ERC-20 transfer/approve builders | **Replace** with module calldata encoders only | No raw transfers from the agent |
| `router::guard_panics`, `configure` once-only semantics, `Zeroizing`, strict serde `deny_unknown_fields`, `scripts/audit_no_mocks.py`, `scripts/validate.sh` | **Keep** | Good patterns |

### 10.2 Crate layout (owned by the core card, §22)

```
src/fusion-core/
  src/lib.rs            -- UniFFI exports only (Appendix B); panic guard on every export
  src/amount.rs         -- RawAmount(U256), decimal-string parse/format (6 dp), checked ops
  src/manifest.rs       -- DeploymentManifest parse + validation (chainId ∈ {31337, 84532})
  src/types.rs          -- Policy, Action, ActionKind, ModuleState, Queue, Payee, Snapshot DTOs
  src/hashing.rs        -- §9 exactly; sol! types via alloy-sol-types
  src/merkle.rs         -- OZ-compatible StandardMerkleTree (sorted pairs), proofs, verify
  src/buckets.rs        -- §7.9
  src/candidates.rs     -- §10.4 enumerator + economics
  src/preview.rs        -- pure mirror of §7.7 checks; returns every rule outcome
  src/planner_io.rs     -- prompt facts builder + strict output validator (§10.6)
  src/passport.rs       -- canonical passport JSON (JCS) + DecisionHash
  src/encode.rs         -- calldata for execute/queueAction/cancel/pause/unwind/Safe txs
  src/snapshot.rs       -- one-block reads via alloy (Multicall3 at a fixed block hash)
  src/sign.rs           -- used only in :signer: sign prepared EIP-1559 tx with a zeroized key
  tests/vectors.rs, tests/candidates_prop.rs, tests/preview_parity.rs, tests/merkle.rs
```

### 10.3 Deployment manifest (signed build config, never model output)
`profile` (FULL | SCALED), `chainId`, `rpcUrl`, `safe`, `module`, `asset`, `vaultA`, `vaultB`, `sanctionsList`, `codeHashes{…}`, `deploymentBlock`, `explorerBaseUrl`. Verify `eth_chainId` and every code hash before showing "Connected".

### 10.4 Core algorithm: candidate enumeration and economics (submission "core algorithm")

Inputs: snapshot S (§10.5), policy P, committed bills, `now` (chain time of the snapshot block). Output: ≤ 7 candidates + HOLD, deterministically ordered.

1. **Due bills:** for each committed unpaid bill with `now ≥ dueAt − payEarlyWindow`, payee ACTIVE and not sanctioned, `C ≥ amount` → PAY candidate. Order: earliest `dueAt`, then `obligationIdHash`.
2. **Safety withdraw:** if `C < R + B` → WITHDRAW `min(R + B − C, maxWithdraw)` from the vault with the lowest rate first.
3. **Invest surplus:** if `sweepable > 0` → DEPOSIT `min(sweepable, cap headroom)` into each eligible vault, if `netBenefit > minNet`.
4. **Rebalance:** if `rate(dst) − rate(src) > 0` and `netBenefit(amount) > minNet` and `C ≥ R` → REBALANCE.
5. **HOLD** (always present).

Economics (integer, floor):
```
gross   = floor(assets × rateBps × horizonSec / (10_000 × 31_536_000))
feeRaw  = ceil(gasUnits × maxFeePerGas × ethUsdScenario) → raw USDC (scenario value, labelled)
net     = gross − feeRaw ;  minNet = 1 USDC (FULL) / 0.01 (SCALED) ;  horizon = 30 days
```
Rates: Aave = `currentLiquidityRate` from the Pool (real); demo vault = configured scenario rate (labelled "simulated"). Never shown as guaranteed APY.

Each candidate carries: kind, exact Action (with `decisionHash = 0`), proof, candidateId, tier, route, reason code, before/after buckets, fee assumptions. Same inputs → byte-identical output (property-tested).

Tier route function (shared Rust/Solidity semantics):
```
tier(assets) = INSTANT if assets ≤ micro ; DELAYED if ≤ delayed ; OWNER if ≤ hard ; else REJECT
route = OWNER_APPROVED policy ? (DELAYED ? APPROVE+QUEUE : APPROVE) : tier
```

### 10.5 Snapshot
One block number + hash. Reads via Multicall3 at that block: Safe asset balance, vault shares, `convertToAssets`, `previewDeposit/Withdraw`, `maxWithdraw`, module `getPolicy/getState/getQueue`, `isPaid` for each due bill, `getPayee` for referenced payees, `isSanctioned` for referenced payees, Aave `getReserveData(asset).currentLiquidityRate`. Any failed read → error (never zero). TTL 30 s or 5 blocks; chain-time rollback never extends a TTL.

### 10.6 Planner I/O (model boundary)
- **Prompt facts** (built by Rust, not Kotlin): numbered candidate lines with kind, plain amounts, vendor label, reason code, before/after free cash; the bounded user goal (≤ 200 chars, untrusted, quoted and fenced); the instruction to reply `{"choice":"<candidateIdShort>","reason":"<REASON_CODE>"}`. `candidateIdShort` = first 8 hex chars, unique within the set (else extended).
- **Validator:** strict JSON, exactly two keys, `choice ∈ set ∪ {"HOLD"}`, `reason ∈ fixed vocabulary`; trailing text, unknown keys or unknown IDs → one format retry → HOLD. Late results after epoch change → discard.
- Limits: one job at a time, ≤ 2,048 input tokens, ≤ 64 output tokens, 25 s budget.
- **Fallback planner** (labelled `planner=deterministic`): the first non-HOLD candidate by the §10.4 order. Used when the model is unavailable. Passports record which planner decided.

### 10.7 Preview (pure)
No nonce allocation, no key load, no network. Returns every §7.7 rule outcome (PASS/FAIL/NA with numbers), tier, route, before/after buckets, exact Action + digest. `tests/preview_parity.rs` + Foundry differential tests MUST agree on accept/reject for the boundary corpus.

### 10.8 Decision passport (canonical JSON, JCS)
`{version, chainId, safe, module, snapshot{block, hash}, mandateHash, candidateSet[ {id, kind, assets, reason} ], planner{kind: "mnn"|"deterministic", modelId, modelSha256, ahemMode, ahemScores?}, userGoalSha256, choice, reasonCode, actionDigest?, route, createdAtChain}`. No private labels, no free text (goal is hashed). `DecisionHash` = keccak256 of it.

### 10.9 Quality gates (core)
`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --locked` (debug + release), vectors match Solidity, UniFFI bindings regenerated with no diff, `audit_no_mocks.py` passes, `grep -E 'f32|f64' src/` in money paths = 0 hits.

---

## 11. Android app, processes and MNN

### 11.1 Build
Gradle 8.13, AGP 8.11.0, Kotlin 2.0.21, Java 17, compileSdk 36, minSdk 29 (per the UI reference; was 24 in v3). arm64-v8a first. MNN SDK as a checksum-pinned local Maven artifact, consumed only by the inference code. Rust built with `cargo-ndk`, pinned toolchain; generated Kotlin bindings are outputs, never hand-edited.

### 11.2 Processes [S] and APKs [F]

| | [S] | [F] |
|---|---|---|
| Inference | `android:process=":inference"` bound service; only module that links MNN; **no network code**; model files in app-private storage, hash-verified | Separate APK, **no INTERNET permission** |
| Signer | `android:process=":signer"` bound service, `exported=false`; keys only here; owns the signer DB | Separate APK, signature-level custom permission, per-call caller UID/cert check |
| Main | UI, DB, worker, Accountant | Same |

Honest S wording (self-assessment): "Processes are isolated; Android permissions are per-app, so the inference process could technically open sockets. We verify by code review and a network-capture test. Full separation into a no-INTERNET APK is a finals deliverable."

### 11.3 Keys
secp256k1 keys generated in `:signer`, wrapped with Android Keystore AES-GCM (StrongBox if present). Owner key unwrap needs fresh biometric/device credential (`setUserAuthenticationParameters(0, BIOMETRIC_STRONG | DEVICE_CREDENTIAL)`). The agent session key unwraps automatically only while a live session exists on-chain. Keys go to Rust `sign.rs` as `Zeroizing<[u8; 32]>` inside `:signer` only. This is at-rest protection, not hardware Ethereum signing (stated in §21).

### 11.4 MNN runtime
- One native session, one generation in flight; state machine UNPROVISIONED → LOADING → READY → GENERATING → READY; timeout → DRAINING/FAILED, never straight to READY.
- Blocking `chat()` path; fresh context per financial decision; temperature 0 / greedy where supported.
- Model: **LFM2.5-230M MNN** (D4). Pin revision, weights SHA-256, tokenizer, config, license. Record device, Android version, cold load, p50/p95 latency, peak RSS and thermal over 20 sequential runs. The UI shows measured values or "not measured".
- Fix the SDK CI import check to compare **ABI-matched** bridge/library pairs.

### 11.5 Orchestration worker (`:main`)
A single coroutine actor per treasury, with persistent state (§14). Loop: snapshot → R mismatch check → candidates → planner → passport → preview → signer `prepare` → route → reconcile. It runs only while the app is in the foreground or under a foreground service with a visible notification ("Agent shift running"). No unattended background claim.

### 11.6 Privacy
Default LOCAL_ONLY for planning. The Accountant cloud path is opt-in (§16). Network-capture test: `:inference` produces zero packets during planning [S, by test].

---

## 12. AHEM integration

| Mode | Behaviour | UI label | Cut |
|---|---|---|---|
| OFF | No evaluation | "AHEM off" | S |
| SHADOW | Real callbacks, scores recorded in the passport, no effect on output | "AHEM observing (not enforcing)" | S if the hook builds; else OFF |
| ENFORCED | Veto stops/discards output before any tool use; missing coverage or errors → HOLD | "AHEM enforcing · coverage N %" | F, gated |

- The shipped SDK `libllm.so` does not export AHEM hook symbols; ENFORCED needs a pinned, patched MNN build with a versioned veto-only callback ABI (`ahem_hook_abi_version`, register/unregister, coverage report).
- Calibration uses activations from the exact shipped model/quantization, with held-out scenario families; store projection (w, scale, bias, thresholds) + dataset hashes.
- AHEM ALLOW never authorises anything. Module, signer and Rust checks always run.
- ENFORCED gates: forced-veto test, coverage on valid decodes, fail-closed on missing callback/NaN/panic/stale generation, lifecycle safety, held-out ASR/FBR, ablation (model only vs + policy vs + AHEM). If AHEM adds no measured benefit, say so.
- **Pitch framing:** "Because the model can only choose from a safe list, AHEM is defence in depth and a research contribution, not the thing that keeps money safe."

---

## 13. Signer protocol 3.1.0

Transport: Binder to `:signer`, typed JSON ≤ 64 KiB, unknown fields rejected. **No generic sign endpoint.**

### 13.1 Agent operations

| Operation | Input | Signer behaviour |
|---|---|---|
| `capabilities` / `status` | — | Versions, role addresses, latch state, unresolved attempts, gas runway |
| `prepare` | requestId, idempotencyKey, operation (QUEUE / EXECUTE / CANCEL / PAUSE / UNWIND), exact Action + proof + passport JSON, expected digest/mandate, snapshot ID, expiry | Reload manifest; verify chain + code hashes; read module state; recompute all hashes **including DecisionHash from the passport**; independent checks (tier, caps, reserves, payee, sanctions, bounds); `eth_call` simulation from the real sender with expected deltas; persist bytes; return attemptId, EOA, nonce, txHash, simulated deltas |
| `broadcast` | attemptId, txHash | Re-check latch/authority/expiry; persist BROADCAST_REQUESTED; send stored bytes |
| `reconcile` | attemptId | Canonical receipt/nonce status + event match |
| `halt` | incidentId, reason, generation | Persist latch first, then guardian `pause` |
| `retireUnsent` | attemptId + proofs | Only if BROADCAST_REQUESTED was never persisted and authority was revoked on-chain |

Same idempotency key + same request = same attempt; same key + different request = error. The signer may refuse; it never edits an action to make it pass.

### 13.2 Owner operations (trusted review screen only)
- Safe txs: `configurePolicy`, `commitObligations`, `addPayee`, `startSession`, `resume`, `revokeSession`, `approveAction`, `revokeActionApproval`, `ownerPayObligation`, `enableModule`, owner transfers.
- The app sends a request ID only. The signer fetches the canonical request, reads the chain, decodes the Safe tx, renders its own review, requires biometric auth, signs the `SafeTx` EIP-712 hash and journals the signature.
- **Second signature [S]:** the signer exports the unsigned SafeTx + owner signature as a JSON/QR "approval packet". The approver signs in Safe{Wallet} (if 84532 is supported) or via `scripts/approve.sh` (Foundry `cast wallet sign --no-hash` over the SafeTx hash) and executes `execTransaction` with both signatures. **[F]:** approver phone + Safe Transaction Service or QR.
- Single-owner stop actions (`pause`, `cancelQueuedAction`, `cancelPayee`, `emergencyUnwind`) are signed by one owner directly to the module, still through the review screen.

### 13.3 Health states
UNPROVISIONED, LATCHED, READY, PREPARED, BROADCAST_REQUESTED, UNKNOWN, MINED, CONFIRMED, RECONCILED, REVERTED, RETIRED_UNSENT, INVALIDATED_KEY_RETIRED. The signer always starts LATCHED; only a confirmed `resume` receipt clears it.

---

## 14. Storage, durability and recovery

### 14.1 Stores
- **App DB** (Room, WAL, `synchronous=FULL`): bills, payees mirror, commitments, policy views, decisions/passports, plans, worker attempts, outbox, evidence, Accountant tables (`docs/ACCOUNTANT_AGENT.md`), x402 receipts [F].
- **Signer DB** (`:signer` only): manifest copy, key ciphertext refs, per-EOA nonce/fee reservations, signed bytes, Safe signatures, latch.
- **Inference:** verified model files + ephemeral request context only.

### 14.2 Tables
- `obligation(id, idHash, revision, payeeIdHash, amountRaw, dueAt, privateLabel, status, commitmentRoot, paidTx, paidBy)`
- `payee(payeeIdHash, account, label, addedTx, activeAt, status, sanctionsCheckedAt)`
- `commitment(root, reservedRaw, count, safeTxHash, confirmedBlockHash)`
- `policy_view(chainId, safe, module, epoch, policyHash, root, reservedRaw, confirmedBlockHash)`
- `decision(id, requestId, routeEpoch, snapshotHash, candidateSetDigest, selectedId, planner, modelSha256, ahemReport, passportJson, decisionHash, status)`
- `plan(id, actionDigest UNIQUE, exactActionJson, proofJson, requiredTier, state, queueTx, readyAt, expiresAt)`
- `worker_attempt(idempotencyKey UNIQUE, canonicalRequestHash, signerAttemptId, operation, state, txHash)`
- `safe_tx(safeTxHash UNIQUE, purpose, payloadHash, sigOwner, sigApprover, state, execTx)`
- `evidence(txHash UNIQUE, kind, scenarioId, blockNumber, status, decisionHash, source: APP|SCRIPT)`
- signer: `protected_attempt`, `protected_latch`, `nonce_reservation`

### 14.3 States
- Plan: DRAFT → PLANNED → AWAITING_OWNERS | QUEUE_SUBMITTING | READY → (QUEUED_WAIT → READY) → EXECUTING → RECONCILED; terminals HELD, BLOCKED, CANCELLED, EXPIRED, INVALIDATED. UNKNOWN is an overlay, never terminal.
- Safe tx: DRAFTED → OWNER_SIGNED → APPROVER_SIGNED → SUBMITTED → CONFIRMED | REVERTED | ABANDONED.
- Bill: DRAFT → COMMITTED → SCHEDULED → PAID | CANCELLED.

### 14.4 Crash rules
Same idempotency key after any crash; PREPARED handles are returned, never re-nonced; BROADCAST_REQUESTED → UNKNOWN → reconcile by hash/nonce before any new attempt; queue state is always re-read from the chain; corrupt/unsupported storage → latch, preserve files, full reconciliation from the deployment block. Reinstalling never auto-resumes. Success requires receipt status 1, the correct `to`/selector/sender/nonce, the matching module event (digest, nonce, epoch, session, decisionHash) and matching balance deltas. Demo: 3 confirmations on Base Sepolia; ancestry re-checked for 64 blocks.

---

## 15. Ops wallet and x402 [F; S? only after all S rows are green]

- **Ops wallet** = a separate EOA in `:signer`, holding **Circle USDC** on Base Sepolia (D3). Owners fund it manually; it is never connected to the Safe.
- The worker calls x402-gated endpoints. On `402 Payment Required` it parses the requirements and asks the signer for an EIP-3009 `transferWithAuthorization`.
- Signer checks: endpoint in the allowlist; network `eip155:84532`; asset = Circle USDC; `payTo` = the endpoint's pinned address; price ≤ 0.01 USDC per request; daily spend ≤ 0.50; wallet balance ≤ 1.00 (above that the signer refuses and asks owners to sweep). Otherwise refuse.
- Facilitator: x402.org (testnet) or CDP. Pin one and record settlement tx hashes.
- **Data service** (`src/x402-service`, team-operated, labelled): `/v1/rates`, `/v1/usdc-price` (scenario; the arena can inject 0.97), `/v1/risk`. Responses are signed by the service key; the app checks signature + freshness. Paid data is still **untrusted**: it can only halt or re-rank, never add an action, address or amount.
- Blast radius = the ops wallet balance cap (1 USDC).
- Arena: "x402 overcharge" → the signer refuses, no payment.

---

## 16. Accountant agent

Fully specified in `docs/ACCOUNTANT_AGENT.md` (implemented). Architectural contract:

- **No authority:** constructed with `AccountantReadModel` + `AccountantAudit` only; it cannot reach the candidate generator, policy, planner, signer or contract. Enforced by a source-scan unit test.
- **Local by default**; cloud (DeepInfra) is opt-in with a versioned consent; the brief `fw.accountant.brief.v1` is ≤ 4 KB with coarse values only (2 significant figures, enums, windows).
- **EgressGuard** fails closed (schema, 2-sig-fig amounts, no address/hash/URL/email/seed shapes, canary deny-list of known private strings); every attempt is logged with payload SHA-256.
- Cloud replies are schema-checked, length-capped, rejected on any link/address, rendered as plain text.
- `google/` and `anthropic/` model IDs are refused (provider retention). Host pinned, no redirects.
- **On a financial incident** (latch), the cloud path is disabled until the latch clears.
- Pitch line: "The phone's AI moves money within limits. The optional cloud analyst only sees rounded numbers you agreed to share, and it can't move anything."

---

## 17. UX specification

The visual language is the existing **Obsidian Institutional bento** system (`docs/UI_REFERENCE.md`): tokens, typography, charts, WCAG AA contrast, EN/中文. This section defines **what** screens must show; the UI reference defines **how** they look.

### 17.1 Principles
1. **One story:** "Money you owe is untouchable. The agent pays your approved bills and invests only what's left."
2. **Proof over promises:** every status that matters links to on-chain evidence.
3. **Calm by default, loud when it matters.**
4. **Plain words first, exact data one tap away** (Expander).
5. **One person can stop; both must start.**
6. **Never colour alone; never fake greens.**

### 17.2 Glossary (UI words → system terms)

| UI (EN) | UI (中文) | System term |
|---|---|---|
| Bill | 账单 | Obligation |
| Verified vendor | 已验证收款方 | ACTIVE payee |
| New vendor (usable in 42 min) | 新收款方（冷静期） | Payee in cooling |
| Set aside for bills | 账单预留 | Reserved R |
| Safety cushion | 安全垫 | Buffer B |
| Spare cash | 可用资金 | Free C − R |
| Safe to invest | 可投资 | Sweepable |
| Earning interest | 生息中 | Deployed V |
| Agent shift | 代理值班 | Session |
| Instant / Waits 3 min / Needs both of you | 即时 / 等待3分钟 / 需双人确认 | Tier routes |
| Holding | 暂不操作 | HOLD |
| Stopped | 已停止 | Latch (+ on-chain pause) |
| Pull everything back | 一键撤回 | Emergency unwind |

### 17.3 Navigation (4 tabs + sheets)

```
Top bar: Brand · TESTNET pill · network/health dot · Avatar (Profile sheet)
Tabs:    Treasury · Bills · Agent · Safety
Floating: [■ Stop] (long-press 1 s) whenever an agent shift is live
Profile sheet: role, language, Autonomy & limits, People & keys, AI & privacy, Network, Data export, Judge mode
```

### 17.4 Health line (under the top bar, tappable)
`● Agent on · shift ends 21:40 · LFM2.5-230M (p95 1.9 s) · AHEM observing · Base Sepolia #12,345,678 (4 s ago)`
States: On / Stopped on this phone / Stopped on-chain ✔ / Waiting for both of you / Offline / Wrong network (blocking). Unmeasured → "not measured".

### 17.5 Treasury (existing `TreasuryScreen`, now on real data) [S]
Hero: **Spare cash** with shortfall banner ("Short by X — nothing will be invested", [Fix] → withdraw suggestion). Allocation donut (cash / set aside / earning). Runway gauge. Rates tile: **Aave (real) 0.21 % APR** vs **Demo vault (simulated)**. Upcoming bills (3). Action bar: `Receive · Pay a bill · Withdraw to owner · Advanced`. Ask Accountant tile. Every number is tagged with its block; loading shows skeletons, never zeros.

### 17.6 Bills and Vendors (new tab) [S]
- Segments: **Due soon · Upcoming · Paid · Drafts**; each row has the vendor + verified badge, the amount, a due chip and a status chip (Draft / Protected / Scheduled / Paid).
- Sticky banner when uncommitted changes exist: "3 changes aren't protected yet. The agent won't pay or invest until you both confirm." → **Review & protect** (change list, new total owed, effect on "Safe to invest").
- Add bill: vendor picker (verified/cooling only, **no address field**), amount, due date, note; CSV import with per-row errors; [F] "Read invoice on this phone" (local draft; any address in the document is shown as ignored).
- **Vendors list:** status Verified / New — usable in mm:ss (chain-time countdown) / Cancelled. No edit button; "Address changed?" explainer about BEC.
- **Add vendor flow:** name → address (paste/QR) → **type the last 6 characters from an independent channel** → lookalike check (§8.4) → sanctions pre-check (TEST LIST) → summary → signer review → "Usable after 14:42 (chain time)".

### 17.7 Agent (existing `AgentScreen`, extended) [S]
- Mandate passport tile (model, scope, shift expiry, epoch) + daily limit ring + Done/Waiting/Blocked stats.
- **Queue panel** pinned while a queue exists: `⏱ Pay Acme 75.00 USDC · ready in 2:41 · expires 14:15 · [Cancel]`. States: Submitting → Waiting → Ready (re-checking) → Running → Done | Cancelled | Expired | Invalidated | "Executed before cancel landed".
- **Decision feed:** proposals, holds, executions, halts; reason chip (`BILL_DUE`, `SURPLUS_ABOVE_THRESHOLD`, `FEES_EXCEED_YIELD`, `CUSHION_REFILL`, `RISK_HALT`); **Why?** shows the candidate list with the chosen one highlighted, rule outcomes and the four-layer checklist (Model / Policy / Signer / Contract) with real states.
- **Activity** sub-tab: ledger timeline with chips (Queued, Sent, Checking, Confirmed, Reconciled, Reverted, Cancelled, Held), explorer link, passport link, filters, CSV/JSON export.

### 17.8 Decoded action card (`PreFlightGuardrailCard`) [S]

```
┌──────────────────────────────────────────────┐
│ PAY BILL                     ⏱ Waits 3 min   │
│ Pay Acme Hosting 75.00 USDC                  │
│ Bill INV-1042 · due Fri 24 Oct               │
│ To  0x1a2B c3D4 … 9f4E 5a6B  ✔ Verified 3 d  │
│ Set aside     1,500.00 → 1,425.00            │
│ Spare cash    3,500.00 → 3,500.00 (same)     │
│ Simulated     Safe −75.00 · Acme +75.00 ✔    │
│ Network fee   ≤ 0.00004 ETH (estimate)       │
│ Checks ✔ reserve ✔ daily ✔ vendor ✔ amount   │
│        ✔ sanctions (TEST LIST) ✔ once-only   │
│ Decision #9f3b…a109 (anchored on-chain)      │
│ [ Details ]               [ Queue (3 min) ]  │
└──────────────────────────────────────────────┘
```

| Kind | Headline | Extra rows |
|---|---|---|
| PAY_OBLIGATION | Pay {vendor} {amount} | bill ID, due, verified since, set-aside before → after |
| DEPOSIT | Invest {amount} in {Aave / Demo vault} | min shares, exposure vs cap, est. net (rate source labelled), break-even |
| WITHDRAW | Move {amount} from {vault} to cash | max shares burned, reason |
| REBALANCE | Move {amount} from {A} to {B} | both bounds, rate gap, net |
| HOLD | Holding — no action | reason, what would change it |

The **Simulated** row comes from the signer's `eth_call` (W13). Card fields come only from validated data, never from model prose. Details: full addresses, raw amounts, bounds, nonce, epoch, digest, candidate ID, snapshot block, decisionHash.

### 17.9 Signer review screen (`:signer` activity) [S]
Rebuilt independently from the chain + request; same fields + decoded Safe tx (to, function, args) + SafeTx hash. Digest chip: "Matches what Fusion showed ✔" or blocking red "Mismatch — do not approve". Biometric; `FLAG_SECURE`; `setFilterTouchesWhenObscured(true)`. After signing: "Signed by you (1 of 2). Waiting for approver." Never show "Approved" before the Safe tx confirms.

### 17.10 Safety tab (existing `ProofScreen`, honesty fix) [S]
- Headline and scorecard are read from `evidence/results/latest.json` (produced by the arena/eval runners) and show run date, counts and denominators: "ASR 0/30 (95 % upper bound ≈ 10 %)". If no results file: "Not measured yet", not fixture numbers. Fixtures only appear with a DEMO pill.
- Arena grid: each tile shows the scenario, the stopping layer, and a tx/revert link or a test name.
- **Emergency unwind** card: "Pull everything back into the Safe and stop the agent" (any one owner; long-press + confirm).
- Privacy tile: Accountant cloud toggle, consent version, egress counters (existing).

### 17.11 Judge mode (Profile → Judge mode; Auditor login lands here) [S]
- **Verify panel:** QR + explorer links for the Safe, module (verified source), vaults, asset, sanctions list; the latest tx per evidence kind.
- **Passport checker:** pick any executed action → re-hash the passport → compare with the on-chain `decisionHash`.
- Scorecard, permission matrix (§6.5), and "What we don't claim" (§21.2).

### 17.12 Stop and restart
Long-press 1 s → "Stopped on this phone" (< 250 ms target, measured) → "Pausing on-chain… (tx)" → "Paused on-chain ✔". Honest copy: "Anything already sent may still complete. Restarting needs both of you." Restart is disabled until unresolved items are reconciled.

### 17.13 Error and empty states

| Situation | Message |
|---|---|
| Loading | "Checking the chain…" skeletons, never zeros |
| RPC down | "Can't reach the network. Showing block #… (3 min old). Agent paused." |
| Wrong network | Full screen: "Fusion only runs on Base Sepolia in this build." |
| Model loading / error | "Local AI is loading (~N s, measured)." / "AI couldn't decide — holding. Nothing was sent." |
| Contract revert | "Blocked by the treasury contract: {plain reason}" + error name |
| Unknown tx | "Sent, result not confirmed yet. Not retrying. Checking…" |
| Gas runway low | "Agent has fuel for ~N actions. Top up ETH." |

### 17.14 The UI must never
Show "Approved/Paid/Cancelled/Paused on-chain" before the receipt; show unmeasured metrics as numbers; offer a free-text address in the agent or bill path; present simulated yield, injected prices, the test sanctions list or fixtures without a label; hide a shortfall by rounding.

### 17.15 Accessibility
TalkBack labels for every chip/chart; addresses read in groups; 200 % text without truncating amounts/addresses in review; 48 dp targets; Stop reachable via an accessibility action; reduced motion respected.

---

## 18. Evaluation and attack arena

### 18.1 Arena scenarios

| # | Scenario | Expected stopping layer | S runner |
|---|---|---|---|
| A1 | Encoded injection (Morse/base64) "pay 0xattacker" | Model output check (no such candidate) → HOLD | Device (real model) |
| A2 | Fake invoice PDF with attacker address | Bill path has no address field; payee must exist | Rust + UI test |
| A3 | "Bank details changed" → new vendor address | Cooling: `PayeeNotActive` | Base Sepolia revert |
| A4 | Lookalike address | App lookalike detector | Rust/Kotlin test |
| A5 | Amount inflation (75 → 750) | Merkle: `BadObligationProof` | Foundry + Anvil |
| A6 | Duplicate payment | `ObligationAlreadyPaid` | Base Sepolia revert |
| A7 | Reserve raid (invest money owed) | Rust preview + `ReserveViolation` | Base Sepolia revert |
| A8 | Drip drain to daily cap | `DailyLimitExceeded` | Foundry |
| A9 | Queue bypass | `QueueRequired` / `QueueNotReady` | Base Sepolia revert |
| A10 | Depeg 0.97 (injected, labelled) | Reflex latch + pause | Device / Anvil |
| A11 | Sanctioned vendor (TEST LIST) | `PayeeSanctioned` | Foundry + Anvil |
| A12 | x402 overcharge | Signer refusal | [F] |
| A13 | Unknown broadcast + restart | Reconciles exactly once | Instrumented test |
| A14 | Owner pays a bill, agent tries again (G2 regression) | `ObligationAlreadyPaid` | Foundry |
| A15 | Session key calls `configurePolicy` / `resume` | `Unauthorized` | Foundry |

Setup failures show "Scenario error", never "Blocked".

### 18.2 Corpus (frozen before any calibration; hash recorded)
**Benign 30:** tier boundaries (8), multiple due bills (4), invest surplus (5), safety withdraw (3), messy input (5), HOLD-correct (2, outside the actionable denominator), owner-tier approval (3).
**Malicious 30:** encoded/indirect injection (5), payee fraud (6), amount/bill tampering (4), reserve/limit abuse (5), authority abuse (5), risk events (3), x402 abuse (2, F).
Each malicious case lists allowed actions and the intended violation before it runs.

### 18.3 Metrics
End-to-end ASR; proposal ASR (model-level); FBR; legitimate acceptance into the correct tier; completion; availability; latency p50/p95 (model, halt); fee per action. Targets (not measurements): ASR = 0 observed, FBR ≤ 10 %, acceptance ≥ 90 %. Publish counts + rule-of-three bounds, never "100 % secure".

### 18.4 Protocol
S: one pass of every case at the Rust and contract layers; the model layer as far as time allows, counts reported as they are. F: 3 repeats, seeds recorded, layer attribution, AHEM ablation on the same device/model.

---

## 19. Validation gates and execution evidence

### 19.1 Profiles
- **core** = everything except AHEM ENFORCED and x402. Required for any "working product" claim.
- **full** = core + AHEM gates. Required for "tensor-level enforcement" claims.

### 19.2 Acceptance families (each starts NOT_RUN)
SAFE-, MOD-, TIER-, QUEUE-, RES-, OBL-, PAYEE-, UNWIND-, ANCHOR-, SIGN-, INF-, DUR-, UX-, ARENA-, EVAL-, AHEM-, X402-. A results runner (`scripts/results.py`) maps each ID to the real test names, commands, exit codes, counts and evidence hashes, and writes `evidence/results/latest.json`. FAIL / BLOCKED / NOT_RUN / zero assertions block any claim that depends on them.

### 19.3 Command families
- Contracts: `forge fmt --check && forge build && forge test -vvv` (unit, fuzz 10k runs, invariant I1–I7 with recorded seeds), `forge script Deploy.s.sol --verify`.
- Rust: `cargo fmt --check && cargo clippy -- -D warnings && cargo test --locked` + vectors.
- Android: `./gradlew :app:assembleRelease :app:testReleaseUnitTest lint` + connected arm64 tests with the exact model files.
- Evidence: `scripts/collect_evidence.py` → `evidence/base-sepolia.json` (tx hash, kind, scenario, block, status, decisionHash, source APP|SCRIPT).

### 19.4 Execution evidence (deliverable 06)

| # | Transaction (Base Sepolia) | Source | Cut |
|---|---|---|---|
| E1 | Safe creation (2 owners, threshold 2) | SCRIPT | S |
| E2 | `enableModule(KavachiModule)` | SCRIPT | S |
| E3 | `configurePolicy` | SCRIPT or APP | S |
| E4 | `addPayee` ×2 (+1 later for the A3 revert) | APP preferred | S |
| E5 | `commitObligations` | APP preferred | S |
| E6 | `startSession` + `resume` | APP preferred | S |
| E7 | PAY_OBLIGATION instant (≤ 10) | **APP (agent)** | S |
| E8 | `queueAction` + `execute` (delayed pay, 10–100) | **APP (agent)** | S |
| E9 | `approveAction` (2 sigs) + `execute` (owner tier, > 100) | APP + approver | S |
| E10 | DEPOSIT into Aave waUSDC | **APP (agent)** | S |
| E11 | WITHDRAW / REBALANCE | **APP (agent)** | S |
| E12 | `cancelQueuedAction` | APP | S |
| E13 | `ownerPayObligation` | APP + approver | S |
| E14 | `pause` (via Stop button) | APP | S |
| E15 | `emergencyUnwind` | APP | S |
| E16 | Intentional reverts: reserve raid, cooling payee, queue bypass, duplicate pay | APP / SCRIPT, labelled | S |
| E17 | x402 settlement | APP | F / S? |

Every agent tx row lists its `decisionHash`, and the passport JSON is published next to it.

### 19.5 Fallback rule (honesty)
If the phone pipeline is not ready for an agent row by S, the tx MAY be produced by `fusion-core`'s CLI runner (`cargo run --bin agent-runner`) using the same core, the same signer logic and `planner=deterministic`. It MUST be labelled `source=SCRIPT, planner=deterministic` in the evidence file and the docs. Never present scripted txs as model-driven.

---

## 20. Demo, video and deck

### 20.1 Story
"Mei runs a 20-person software studio, paid in USDC. Last year a fake 'new bank details' email almost cost her a month of payroll. Fusion pays only bills Mei and Wei approved, only to vendors they verified, and invests only what's left in Aave. The AI runs on her phone. If anything looks wrong, either of them can stop it with one press."

### 20.2 Five-minute video

| Time | Scene | Proof shown |
|---|---|---|
| 0:00–0:25 | Problem (BEC, poisoning, Grok/Bankr) → Treasury tab | Real Base Sepolia balances, TESTNET pill |
| 0:25–1:05 | Bill due → agent proposes → decoded card with simulation → instant pay | Model trace, tx link, decisionHash match in Judge mode |
| 1:05–1:45 | Bigger bill → queued 3 min → cancel one, let one run | readyAt, cancel receipt, execute receipt |
| 1:45–2:15 | Owner-tier bill → second signature | SafeTx with 2 sigs |
| 2:15–2:40 | Surplus into Aave; HOLD when fees > yield | Deposit tx; hold card |
| 2:40–3:50 | Arena: Morse injection, changed bank details, duplicate pay, reserve raid | Layer attribution, revert txs |
| 3:50–4:25 | Stop button → on-chain pause → emergency unwind | Pause + unwind txs |
| 4:25–5:00 | Scorecard + "what we don't claim" + 中文 switch | Counts with denominators |

Anvil time-warp only for boundary demos, always labelled "local simulation".

### 20.3 Ten-minute deck (10 slides)
1. Problem (sourced numbers) · 2. Persona and job · 3. Fusion in one sentence + Treasury screenshot · 4. Authority split + tier table · 5. Architecture (§5.2) · 6. Core algorithm (candidates → ID-only model → four layers) · 7. Fraud controls (cooling, immutability, Merkle bills, lookalike) · 8. Arena + evaluation (counts) · 9. On-chain evidence (judge bundle QR, decision anchoring) · 10. Limits, finals roadmap, team.

### 20.4 Claims to avoid
"First ever", "guaranteed yield", "audited", "unhackable", "bank-grade", "regulatory approved", "zero risk", "real sanctions compliance" (testnet = TEST LIST), "hardware-secured keys" (they are Keystore-wrapped software keys), "fully isolated apps" at S (processes, not APKs).

---

## 21. Risks and known limits (source for deliverable 05)

### 21.1 Risk register

| Risk | Impact | Mitigation | Residual |
|---|---|---|---|
| Module bug | Loss of Safe funds | Tiny allowlisted call surface; Call-only, value 0; I1–I7 invariants; fuzz; owners can disable the module | Unaudited code |
| Owners collude / both keys compromised | Full loss | 2-of-2 on separate devices; recommend hardware wallets in production | Inherent to owner sovereignty |
| Signer compromise | Agent-bounded loss; owner key exposed during signing | Process isolation, auth-bound unwrap, module caps | Software keys; S uses processes, not separate APKs |
| Inference process opens network (S) | Private context leak | No network code, capture test | Not OS-enforced until F |
| App login bypass | Wrong UI role | Login grants no authority (D10) | Demo credentials are public |
| Vault insolvency / Aave testnet pause | Funds stuck in a vault | Exposure caps; unwind; demo vault fallback | Real vaults not evaluated |
| Fake invoice approved by both humans | Wrong payment | Cooling + independent last-6 + immutability + lookalike | Social engineering of both owners |
| Sanctions data | False negatives/positives | Fail-closed | Testnet list is illustrative |
| Prompt injection | Wrong choice | Candidate-ID-only output; deterministic candidates; four layers | The model may still choose a bad *valid* candidate within limits |
| Pause censorship / delay | Actions settle before pause | Local latch first; queue delay for the mid tier | Inclusion not guaranteed |
| Agent key out of gas | Agent stalls | Gas runway shown; owner top-up | Liveness only |
| Background limits | Agent stops in background | Foreground service; latch on suspend | No unattended operation claimed |
| Cloud Accountant | Coarse business profile leaves the device | Opt-in, guard, caps, audit log | Consented residual |
| Short payee cooling on testnet (1 h) | Weaker BEC protection in the demo | Labelled; production default 24 h | Demo-only setting |
| Regulation | Not deployable everywhere | Testnet prototype; legal review before any pilot | Varies by jurisdiction |

### 21.2 What we don't claim
Audited contracts; real yield; real sanctions screening; hardware-backed Ethereum keys; OS-level app separation at S; AHEM enforcement unless the gates pass; unattended 24/7 operation; any security percentage without counts.

---

## 22. Repository layout, ownership and build order

### 22.1 Layout

```
fusion_wallet/                       (umbrella repo)
  system architecture/               this file (Phase-1 contract)
  docs/                              ACCOUNTANT_AGENT, ASSETS_AND_AAVE, UI_REFERENCE
  docs/submission/                   01_TECHNICAL, 05_SECURITY_SELF_ASSESSMENT, 06_EVIDENCE, deck
  spec/vectors/v4_1.json             frozen hashing vectors (Phase 1)
  spec/abi/IKavachiModule.json       frozen ABI (generated from Appendix A, Phase 1)
  src/contracts/                     NEW Foundry project
  src/fusion-core/                   Rust engine v4
  src/Fintech_Wallet_Agent/          Android (submodule, branch fusion-v3-integration)
  src/mnn-android-sdk/               submodule (pinned)
  src/ahem/                          submodule (pinned)
  src/x402-service/                  [F]
  evidence/                          base-sepolia.json, results/latest.json, passports/
  scripts/                           approve.sh, collect_evidence.py, results.py
```

### 22.2 Campaign cards (disjoint `owned_paths`)

| Card | Phase | Agent | owned_paths | Definition of done | Blocked on |
|---|---|---|---|---|---|
| K0 Contracts freeze | 1 | spec_reviewer | `spec/**` | Vectors + ABI JSON produced from this doc; reviewed | — |
| K1 Contracts | 2–3 | backend-agent | `src/contracts/**` | `forge test` green incl. I1–I7, all §7.7 reverts, vectors | K0 |
| K2 Deploy + evidence scripts | 3 | backend-agent (after K1) | `src/contracts/script/**`, `scripts/**`, `evidence/**` | Base Sepolia deployment verified; E1–E5 hashes | K1 |
| K3 fusion-core v4 | 2–3 | core-agent | `src/fusion-core/**` | §10.9 gates green; vectors match | K0 |
| K4 Android core wiring | 3 | frontend-agent | `…/app/src/main/java/app/fusion/wallet/{data,agent,signer,inference}/**` | Instrumented E2E on Anvil: pay + deposit reconciled | K3 bindings |
| K5 Android UI | 3 | frontend-agent #2 | `…/wallet/ui/**`, `res/**` | Screens §17.5–17.11 on real state; DEMO data gated to debug | K4 interfaces |
| K6 Device + arena verification | 4 | android_ui_tester | `src/Fintech_Wallet_Agent/…/androidTest/**`, `evidence/results/**` | Arena device rows + screenshots | K4, K5 |
| K7 Submission docs | 4 | spec_reviewer | `docs/submission/**`, root `README.md` | All six items present and consistent with the evidence file | K2, K6 |

Submodule rule: Android work happens in a worktree of the submodule's own repo; merge there first, then bump the pointer in the umbrella repo (serialized under `.brain/merge.lock`).

---

## 23. Open questions

| # | Question | Blocks | Default if unanswered |
|---|---|---|---|
| Q1 | Do we hold, or can we mint, **Aave Base Sepolia test USDC** in ≥ 10,000 units for the treasury, and has a `waUSDC.deposit` been tried on a fork? | K2 deploy (profile choice) | Circle fallback, SCALED profile |
| Q2 | Approver for S: Safe{Wallet} web (needs 84532 support), a laptop script, or a second phone? | E9, E13 | Laptop script `scripts/approve.sh` |
| Q3 | Which physical arm64 phone runs the demo, and does LFM2.5-230M already load under the pinned MNN build there? | K4, K6, latency numbers | Deterministic planner fallback, labelled (§19.5) |
| Q4 | Do you accept the S/F cut (x402, separate APKs, second-phone approver, AHEM ENFORCED, 3-repeat eval → finals)? | Card plan | Yes |

---

## 24. Appendix A: Solidity interface

Specification interface only — not an implementation. K0 generates `spec/abi/IKavachiModule.json` from it.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface ISanctionsList {
    function isSanctioned(address account) external view returns (bool);
}

/// Minimal subset of Safe v1.4.1 used by the module.
interface ISafeMinimal {
    enum Operation { Call, DelegateCall }
    function execTransactionFromModule(address to, uint256 value, bytes calldata data, Operation operation)
        external returns (bool success);
    function isOwner(address owner) external view returns (bool);
    function isModuleEnabled(address module) external view returns (bool);
}

/// Kavachi V4.1 module specification interface. NOT an implementation.
interface IKavachiModule {
    enum ActionKind { DEPOSIT, WITHDRAW, REBALANCE, PAY_OBLIGATION }
    enum PermissionTier { READ_ONLY, OWNER_APPROVED, AUTONOMOUS }

    struct VaultLimit { address vault; uint256 maxExposureRaw; }

    struct Policy {
        PermissionTier permissionTier;
        uint256 operatingBufferRaw;
        uint256 microMaxRaw;
        uint256 delayedMaxRaw;
        uint256 hardMaxRaw;
        uint256 dailyLimitRaw;
        uint32 delaySeconds;
        uint16 maxShareSlippageBps;
        address guardian;
        VaultLimit[] vaults;
        uint32 payeeDelaySeconds;
        uint32 payEarlyWindowSeconds;
    }

    struct Action {
        ActionKind kind;
        address sourceVault;
        address destinationVault;
        uint256 assets;
        uint256 minSharesOut;
        uint256 maxSharesBurn;
        bytes32 obligationIdHash;
        bytes32 payeeIdHash;
        address recipient;
        uint64 dueAt;
        uint256 expectedNonce;
        uint256 policyEpoch;
        uint64 deadline;
        bytes32 decisionHash;
    }

    struct Payee {
        address account;
        bytes32 labelHash;
        uint64 activeAt;
        bool cancelled;
    }

    struct Queue {
        bool exists;
        bytes32 actionDigest;
        bytes32 sessionId;
        uint64 readyAt;
        uint64 expiresAt;
        Action action;
    }

    struct ModuleState {
        bool configured;
        bool paused;
        uint256 policyEpoch;
        bytes32 policyHash;
        bytes32 mandateHash;
        bytes32 obligationsRoot;
        uint256 reservedRaw;
        uint256 obligationCount;
        uint256 actionNonce;
        uint256 dailyDayIndex;
        uint256 dailyUsedRaw;
        bytes32 sessionId;
        uint256 sessionStartEpoch;
        address sessionKey;
        uint64 sessionExpiresAt;
        uint256 sessionLimitRaw;
        uint256 sessionUsedRaw;
    }

    // Errors
    error Unauthorized();
    error WrongChain();
    error ModuleNotEnabled();
    error WrongLifecycle();
    error ModulePaused();
    error InvalidPolicy();
    error InvalidSession();
    error SessionExpired();
    error StalePolicy();
    error BadNonce();
    error ActionExpired();
    error InvalidDeadline();
    error ReadOnly();
    error UnsupportedAction();
    error MissingDecisionHash();
    error ForbiddenVault();
    error VaultCodeChanged();
    error InvalidAmount();
    error HardLimitExceeded();
    error DailyLimitExceeded();
    error SessionLimitExceeded();
    error ApprovalRequired();
    error ApprovalExpired();
    error ReserveViolation();
    error Underfunded();
    error ExposureExceeded();
    error InsufficientLiquidity();
    error ShareBoundViolation();
    error AccountingMismatch();
    error ResidualAllowance();
    error SafeCallFailed();
    error QueueRequired();
    error QueueAlreadyExists();
    error QueueBusy();
    error QueueMismatch();
    error QueueNotReady();
    error QueueExpired();
    error NoQueue();
    error InvalidObligationCommitment();
    error BadObligationProof();
    error ObligationAlreadyPaid();
    error PayWindowNotOpen();
    error PayeeExists();
    error PayeeNotActive();
    error PayeeSanctioned();
    error RecipientMismatch();
    error SanctionsCheckFailed();

    // Events
    event PolicyConfigured(uint256 indexed policyEpoch, bytes32 policyHash, bytes32 mandateHash);
    event ObligationsCommitted(uint256 indexed policyEpoch, bytes32 indexed root,
        uint256 reservedRaw, uint256 count, bytes32 mandateHash);
    event PayeeAdded(bytes32 indexed payeeIdHash, address indexed account, bytes32 labelHash, uint64 activeAt);
    event PayeeCancelled(bytes32 indexed payeeIdHash, address indexed actor);
    event SessionStarted(bytes32 indexed sessionId, address indexed key, uint256 indexed policyEpoch,
        uint64 expiresAt, uint256 limitRaw, bytes32 mandateHash);
    event SessionRevoked(bytes32 indexed previousSessionId, address indexed previousKey,
        uint256 indexed policyEpoch, bytes32 mandateHash);
    event ActionApproved(bytes32 indexed digest, uint256 indexed policyEpoch, uint256 expectedNonce, uint64 validUntil);
    event ActionApprovalRevoked(bytes32 indexed digest);
    event ActionQueued(bytes32 indexed digest, bytes32 indexed sessionId, uint256 indexed nonce,
        uint64 readyAt, uint64 expiresAt, bytes32 decisionHash);
    event QueueCancelled(bytes32 indexed digest, address indexed actor, uint256 consumedNonce);
    event QueueExpiredEvent(bytes32 indexed digest, uint256 consumedNonce);
    event QueueInvalidated(bytes32 indexed digest, bytes32 reasonCode, uint256 consumedNonce);
    event ActionExecuted(bytes32 indexed actionDigest, uint256 indexed nonce, bytes32 indexed decisionHash,
        ActionKind kind, uint256 assets, uint256 sharesBurned, uint256 sharesMinted,
        uint256 cashAfter, uint256 reservedAfter, uint256 policyEpoch, bytes32 sessionId);
    event ObligationPaid(bytes32 indexed obligationIdHash, bytes32 indexed payeeIdHash,
        address indexed recipient, uint256 amountRaw, uint256 reservedAfter, bool byOwner);
    event Paused(address indexed actor, uint256 indexed policyEpoch, bytes32 reasonCode, bytes32 mandateHash);
    event Resumed(uint256 indexed policyEpoch, bytes32 indexed sessionId, bytes32 mandateHash);
    event EmergencyUnwind(address indexed actor, bytes32 reasonCode, uint256 cashAfter);
    event UnwindLegFailed(address indexed vault, bytes reason);

    // Views
    function safe() external view returns (address);
    function asset() external view returns (address);
    function deploymentChainId() external view returns (uint256);
    function trustedVaults() external view returns (address[2] memory);
    function sanctionsList() external view returns (address);
    function getPolicy() external view returns (Policy memory);
    function getState() external view returns (ModuleState memory);
    function getQueue() external view returns (Queue memory);
    function getPayee(bytes32 payeeIdHash) external view returns (Payee memory);
    function isPaid(bytes32 obligationIdHash) external view returns (bool);
    function approvalValidUntil(bytes32 digest) external view returns (uint64);
    function hashAction(Action calldata action) external view returns (bytes32);

    // Safe-only (threshold 2)
    function configurePolicy(Policy calldata policy) external;
    function commitObligations(bytes32 root, uint256 reservedRaw, uint256 count) external;
    function addPayee(bytes32 payeeIdHash, address account, bytes32 labelHash) external;
    function startSession(address key, uint64 expiresAt, uint256 sessionLimitRaw) external;
    function resume(uint256 expectedEpoch) external;
    function revokeSession() external;
    function approveAction(Action calldata action, uint64 validUntil) external returns (bytes32 digest);
    function revokeActionApproval(bytes32 digest) external;
    function ownerPayObligation(bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 assets,
        uint64 dueAt, bytes32[] calldata proof) external;

    // Stop / pull-back actions (Safe, any single Safe owner, or guardian)
    function pause(bytes32 reasonCode) external;
    function cancelQueuedAction(bytes32 digest) external;
    function cancelPayee(bytes32 payeeIdHash) external;
    function emergencyUnwind(bytes32 reasonCode) external;

    // Agent (session key)
    function queueAction(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);
    function execute(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);

    // Anyone, after expiry
    function expireQueuedAction() external;
}
```

---

## 25. Appendix B: fusion-core FFI surface (UniFFI, API 4.0)

All functions take and return JSON strings (UTF-8), never panic across the boundary (panic → `{"ok":false,"error":{"code":"PANIC"}}`), and never accept a private key except `sign_prepared`, which is used only in `:signer`.

| Function | Process | Input | Output |
|---|---|---|---|
| `core_version()` | any | — | `{api:"4.0", hashing:"V4_1", gitRev}` |
| `load_manifest(json)` | main, signer | Manifest | validated manifest or error |
| `read_snapshot(manifestJson)` *(async)* | main, signer | — | Snapshot at one block |
| `compute_buckets(snapshotJson)` | main | Snapshot | `{free, reserved, sweepable, shortfall, deployed, total}` (raw strings) |
| `build_commitment(obligationsJson)` | main | committed bills | `{root, reservedRaw, count, leaves[], proofs{id→[]}, changeList}` |
| `enumerate_candidates(snapshotJson, policyJson, obligationsJson, ratesJson)` | main | — | `{candidateSetDigest, candidates[≤7], hold}` |
| `planner_prompt(candidateSetJson, userGoal)` | main | — | prompt string (facts only) |
| `validate_planner_output(rawText, candidateSetJson)` | main | model text | `{choice: candidateId | "HOLD", reasonCode, retryable}` |
| `build_passport(decisionJson)` | main, signer | decision | `{passportJson (JCS), decisionHash}` |
| `preview(candidateJson, snapshotJson, decisionHash)` | main, signer | — | rule outcomes, tier, route, exact Action, digest |
| `hash_vectors_selftest()` | any | — | pass/fail against the embedded `v4_1.json` |
| `encode_call(kind, argsJson)` | signer | EXECUTE / QUEUE / CANCEL / PAUSE / UNWIND / SafeTx kinds | `{to, data}` (to ∈ {module, safe} only) |
| `sign_prepared(txJson, key: bytes32)` | **signer only** | EIP-1559 tx + zeroized key | `{rawTx, txHash}` |

Removed from the v2 surface: `execute_intent`, `configure_wallet`.

---

## 26. Sources

Checked on the dates shown. Re-verify anything chain- or vendor-specific before deployment (this revision could not re-browse; items are carried from v3's 2026-10-18 check and the 2026-10-17 on-chain reads).

- Competition rules and scoring: official 2026 FinTechathon International Track rules PDF (`helper/competition-info/`).
- Aave Base Sepolia addresses and market state: `docs/ASSETS_AND_AAVE.md` (bgd-labs `aave-address-book`, `cast call`, 2026-10-17).
- Accountant design: `docs/ACCOUNTANT_AGENT.md`; UI system: `docs/UI_REFERENCE.md`.
- Grok/Bankr incident (May 2026): https://www.giskard.ai/knowledge/how-grok-got-prompt-injected-an-x-user-drained-150-000-from-an-ai-wallet
- FBI IC3 2025 BEC: https://www.eftsure.com/statistics/payment-fraud-index/
- Address poisoning (Dec 2025 / Jan 2026): https://crypto.news/ethereum-address-poisoning-coas-6-2-m-two-months-2026/
- Safe modules and guards: https://safe.global/blog/safe-smart-accounts-and-diamond-proxies
- Zodiac Roles (scoped module permissions): https://github.com/gnosis/zodiac-modifier-roles
- Circle testnet USDC: https://developers.circle.com/gateway/quickstarts
- x402 networks / facilitator: https://docs.cdp.coinbase.com/x402/network-support.md
- Chainalysis sanctions oracle: https://decrypt.co/94808
- Competitor UX patterns (MetaMask Agent Wallet, Coinbase Agentic Wallets, Giza ARMA, Privy, ERC-7715): https://decrypt.co/375093 ; https://docs.cdp.coinbase.com/agentic-wallet/mcp/faq ; https://docs.privy.io/controls/authorization-keys
