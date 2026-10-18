# Kavachi — Full Architecture v3.0

**Topic F · Agent Wallet · 2026 Shenzhen International FinTech Competition (International Track)**
Full replacement of the v2.0 base architecture. Domain/contract version **V4**. Signer protocol **3.0.0**.

> **Status:** This is a design and implementation contract. Nothing in it is claimed as built or tested. Every threshold and default below is a starting value that must be validated on messy and adversarial synthetic data (Section 16) before any real use.

---

## Contents

0. [How to read this](#0-how-to-read-this)
1. [What changed from v2 and why](#1-what-changed-from-v2-and-why)
2. [Product](#2-product)
3. [Scoring map](#3-scoring-map)
4. [System architecture](#4-system-architecture)
5. [Authority model](#5-authority-model)
6. [On-chain specification](#6-on-chain-specification)
7. [Obligations, payees and compliance](#7-obligations-payees-and-compliance)
8. [Hashing contract V4](#8-hashing-contract-v4)
9. [Off-chain modules](#9-off-chain-modules)
10. [Android, MNN and process isolation](#10-android-mnn-and-process-isolation)
11. [AHEM integration](#11-ahem-integration)
12. [Signer protocol 3.0.0](#12-signer-protocol-300)
13. [Storage, durability and recovery](#13-storage-durability-and-recovery)
14. [Ops float and x402 payments](#14-ops-float-and-x402-payments)
15. [UX specification](#15-ux-specification)
16. [Evaluation](#16-evaluation)
17. [Validation gates](#17-validation-gates)
18. [Demo, video and pitch](#18-demo-video-and-pitch)
19. [Risks and known limits](#19-risks-and-known-limits)
20. [Implementation order](#20-implementation-order)
21. [Appendix A: Solidity interfaces](#21-appendix-a-solidity-interfaces)
22. [Sources](#22-sources)

---

## 0. How to read this

| If you are… | Read |
|---|---|
| Pitching / judging | 1, 2, 3, 15, 18 |
| Building contracts | 5, 6, 7, 8, 21 |
| Building Android/Rust | 9, 10, 12, 13, 14 |
| Building UI | 15 (all of it), then 2 and 5 |
| Running evaluation | 16, 17 |

**Normative words:** MUST / MUST NOT = required for the claimed profile. SHOULD = strong default; deviation needs a written reason. MAY = optional.

**Repository baseline (carried from the v2 audit, still valid):**

| Repo | Pinned commit | Reuse |
|---|---|---|
| Fintech_Wallet_Agent | e5ae118a6af3b31165e6167fdbf3cc971d31b627 | Compose shell, navigation, PreFlightGuardrailCard, Rust strict parsing, Alloy, UniFFI |
| mnn-android-sdk | 5c8f61f47c1afedf86cd1ae79d6df358613c6436 | MNNLlm Kotlin API, JNI bridge, arm64 native libs |
| ahem | 8832419511ff57d82d811fc433f8ba5829305eb4 | Grounding, TensorLogic bridge, rule packs (research layer only) |

The v2 audit findings stay mandatory fixes: float money arithmetic, fail-open ledger, broadcast-as-success, raw private key across FFI, demo data in release, cross-ABI MNN CI check, AHEM hook not present in shipped `libllm.so`. They are restated where relevant below.

---

## 1. What changed from v2 and why

| # | v2 | v3 | Why it wins points |
|---|---|---|---|
| C1 | Reserve + yield only. "Accounts payable" but cannot pay anyone | **Agentic accounts payable**: agent pays *owner-committed bills* to *verified payees* in *exact amounts*, then invests only true surplus | Closes the biggest product gap. Task completion (40%) now covers real payments, not only vault moves |
| C2 | Custom `RestrictedTreasuryWallet` holds funds | **Safe smart account holds funds + `KavachiModule`** bounds the agent | Owners keep full normal wallet functions (send, swap) through Safe; agent is bounded by the module. Resolves the Send/Swap debate. Audited, recognised base |
| C3 | Single owner key | **2-of-2 dual control** (owner + approver) for policy, payees, large approvals | Real SME control (maker-checker). Strong security/compliance story |
| C4 | DemoUSD only | **Circle testnet USDC on Base Sepolia**; mock token only on Anvil | Judges see a real stablecoin contract, real explorer links |
| C5 | No payee concept | **Payee registry**: immutable entries, 24 h cooling period, any owner can cancel, sanctions check at add and at pay | Directly targets invoice-redirection fraud and address poisoning, both major real losses |
| C6 | No compliance screening | **Sanctions screening interface** (Chainalysis-compatible `isSanctioned`), labelled test list on testnet | RegTech points; honest about testnet limits |
| C7 | Agent never pays anything | **Ops float + x402**: agent buys its own data (rates, risk) with a hard-capped micro wallet | "AI as a Financial Participant" made literal; agentic-commerce trend; bounded blast radius |
| C8 | One parameter set | **Two parameter profiles** (Anvil full-size, Base Sepolia scaled 1/100) | Testnet faucets give small amounts; the demo must still hit every tier on a public chain |
| C9 | Arena: generic attacks | Arena adds **payment fraud scenarios**: encoded injection (Grok/Bankr style), fake invoice, changed payee details, lookalike address, duplicate payment | Matches attacks judges have read about this year |
| C10 | 20/20 evaluation | **30 benign / 30 malicious**, payment-specific families, three repeats | Stronger FBR/ASR evidence |
| C11 | UX described per enhancement | **Full UX specification** (IA, every screen, states, copy, accessibility, judge mode) | Innovation & interaction (30%) is judged on what people see |

**Kept from v2 (unchanged in intent):** on-device MNN with no-network inference APK; separate signer APK; candidate-ID-only model output; deterministic integer policy; three amount tiers with on-chain queue and cancel; durable two-store journals; UNKNOWN broadcast handling; AHEM OFF/SHADOW/ENFORCED; strict release gates.

**Removed:** custom owner recovery functions (owners now use normal Safe transactions); `ownerRecover*` methods; DemoUSD on public testnet.

---

## 2. Product

### 2.1 One-line pitch
**Kavachi is an AI treasury agent for small businesses that pays the bills you approved, to vendors you verified, in the exact amount — and invests only the money you don't owe.**

### 2.2 Problem
- Small businesses lose money to **invoice redirection / business email compromise**: a fraudster poses as a supplier and changes the payment details. FBI IC3 recorded about **24,768 BEC complaints and ~$3.05B in reported losses in 2025** (Sources).
- In crypto, **address poisoning** turns one copy-paste into a total loss (~$50M in Dec 2025 and ~$12.25M in Jan 2026 in single incidents) (Sources).
- AI agents with open send authority can be talked into paying attackers: the **May 2026 Grok/Bankr incident** used a permission-granting NFT plus a Morse-code prompt injection to move roughly $150K–$200K; about 80% was later returned (Sources).
- Idle operating cash earns nothing, but sweeping it risks touching money already owed.

### 2.3 Persona
**Small cross-border service business** (e.g. a 10–30 person software studio) that receives client payments in USDC and pays recurring obligations (contractors, SaaS, rent) in USDC. Owner + one finance approver. No full-time treasury team.

> Jurisdiction note: stablecoin use for business payments is regulated differently per country. The product is presented as a testnet prototype; any pilot MUST run only where this use is permitted, after legal review. No claim of regulatory approval is made.

### 2.4 Jobs the agent does
1. **Pay committed bills** on time, exact amount, verified payee only.
2. **Protect reserves**: money owed + operating buffer is never invested or moved elsewhere by the agent.
3. **Invest true surplus** into two approved ERC-4626 vaults; rebalance when net benefit beats fees.
4. **Buy its own data** (rates/risk) via x402 from a tiny capped float.
5. **Explain and ask**: every action is a decoded card; bigger actions wait for humans; anything can be cancelled during the delay window.
6. **Stop instantly** on risk signals (depeg, wrong chain, accounting mismatch).

### 2.5 Scope

| In | Out |
|---|---|
| Base Sepolia (public evidence), Anvil (fault injection, arena) | Mainnet funds, real customer money |
| Circle testnet USDC (Base Sepolia), mock USDC (Anvil) | Multiple tokens, swaps by the agent, bridges, FX |
| Safe v1.4.1 account, `KavachiModule`, `PayeeRegistry` (inside module), two demo ERC-4626 vaults | Upgradeable proxies for Kavachi contracts, ERC-4337 paymaster |
| Owner manual send/swap **through Safe** (owner path) | Agent arbitrary send, agent swap, agent approvals to unknown spenders |
| x402 purchases from the ops float | x402 from the treasury Safe itself |
| On-device MNN planning; optional public education helper (fixed topics) | Cloud LLM on private data |

---

## 3. Scoring map

Rubric from the official rules: task completion 40%, security & compliance 30%, innovation & interaction 30%, on-chain challenge bonus 0–5.

| Criterion | What the judge must be able to verify | Kavachi feature | Evidence artefact |
|---|---|---|---|
| Task completion (40) | Agent completes real financial tasks end-to-end | Pay committed bill; deposit surplus; rebalance; HOLD when not worth it; x402 data purchase | Base Sepolia tx hashes for each; reconciled ledger; video |
| Security & compliance (30) | Agent cannot exceed authority even if the model/app is compromised; fraud controls; audit trail | Module limits, reserve invariant, payee cooling + sanctions, 2-of-2 dual control, queue/cancel, kill switch | Direct-call revert traces, arena runs, security self-assessment, permission-tier table |
| Innovation & interaction (30) | Novel, usable, trustworthy UX | On-device private AI, decoded cards, live arena, judge mode, x402 agent commerce, AHEM evidence | Device demo, screenshots, ASR/FBR table |
| On-chain bonus (0–5) | Ambitious, verifiable on-chain behaviour | Safe module with Merkle-proved bill payments, on-chain cancel window, payee registry | Verified contracts + receipts |

Required submission items mapped:

| Item | Source in this spec |
|---|---|
| Technical documentation | This file + Sections 6–14 |
| Presentation deck (≤10 min) | Section 18.3 |
| Demo video (≤5 min) | Section 18.2 |
| Source code + README + tests | Section 20, 17 |
| Security self-assessment (permission tiers + known risks) | Sections 5, 19 |
| Execution evidence (real tx hashes) | Section 17.4 |

---

## 4. System architecture

### 4.1 Components

| Component | Process / identity | Responsibility | Never does |
|---|---|---|---|
| **Kavachi app** (`:app`) | `app.fusion.wallet`, network for pinned RPC + x402 endpoints | Compose UI, private bills/payees records, Rust planning + policy mirror, orchestration, reconciliation | Hold keys, sign, run the model |
| **Inference app** (`:inference`) | separate UID, **no INTERNET** | MNN model, AHEM, candidate selection | Hold keys, see raw addresses beyond what the request needs, reach network |
| **Signer app** (`:signer`) | separate UID, pinned RPC | Keys (owner, agent session, guardian, ops float), trusted review screen, nonce/broadcast journal | Accept free-form calldata, accept "approved=true" flags |
| **Approver device** | second phone with signer app in APPROVER role, or a standard wallet that can sign Safe transactions | Second owner signature for dual control | — |
| **Safe account** | on-chain | Holds USDC and vault shares; owners transact normally | — |
| **KavachiModule** | on-chain, enabled on Safe | Agent authority: tiers, limits, reserves, queue, bills, payees, ops top-up | Arbitrary calls |
| **Demo vaults A/B** | on-chain ERC-4626 over USDC | Yield targets (simulated yield, labelled) | — |
| **Sanctions list** | on-chain, `isSanctioned(address)` | Screening at payee add and payment | — |
| **x402 data service** | team-operated HTTP service, Base Sepolia | Sells rate/risk snapshots per request | Touch treasury |

### 4.2 Diagram

```mermaid
flowchart TD
  UI["Kavachi app (UI + Rust policy + worker)"] -->|"bounded private request"| AI["Inference app (MNN, no network)"]
  AI -->|"candidate ID or HOLD"| UI
  UI -->|"typed action + fresh snapshot"| SIGN["Signer app (keys, review, journal)"]
  SIGN -->|"agent tx"| MOD["KavachiModule"]
  MOD -->|"execTransactionFromModule (allowlisted calls only)"| SAFE["Safe account (USDC, shares)"]
  SAFE --> VA["Vault A"]
  SAFE --> VB["Vault B"]
  SAFE -->|"PAY_OBLIGATION"| PAYEE["Verified payee"]
  SAFE -->|"OPS_TOPUP (capped)"| OPS["Ops float"]
  OPS -->|"x402 micro-payments"| DATA["Rate/risk data service"]
  APPR["Approver device"] -->|"2nd Safe signature"| SAFE
  SAFE -->|"events"| UI
  RISK["Deterministic risk checks"] -->|"latch + pause"| SIGN
```

### 4.3 Runtime loop (agent mode)

1. Owners set up Safe (2 owners, threshold 2), enable module, configure policy, register payees, commit bills. Signer starts **latched**.
2. Owners start a session (≤24 h) and resume with `expectedEpoch` — both Safe transactions (2-of-2).
3. **Perceive**: one block snapshot of Safe cash, vault shares/previews, module state, queue, paid bills, payee status, ops float balance.
4. **Enumerate**: Rust builds ≤7 feasible candidates + HOLD using integer math. Due bills are always candidates first.
5. **Choose**: MNN picks a candidate ID or HOLD. AHEM report attached.
6. **Check**: Rust preview → signer independent checks → module on-chain checks.
7. **Route by tier**: immediate / queued (cancel window) / needs owner approval (Safe tx, 2-of-2).
8. **Execute + reconcile**: signer broadcasts stored bytes; app reconciles receipt + events + balances.
9. **Explain**: decoded card + activity entry; HOLD is a valid, shown outcome.

Critical risk conditions skip the model: latch locally → block signing → guardian pause → explain afterwards.

---

## 5. Authority model

### 5.1 Roles

| Role | Key location | Can | Cannot |
|---|---|---|---|
| **Owner** (Safe owner 1) | Signer app, owner key, biometric unwrap | Sign Safe transactions; pause module directly | Act alone on threshold-2 operations |
| **Approver** (Safe owner 2) | Second device | Second signature; pause module directly | Act alone on threshold-2 operations |
| **Safe (threshold 2)** | on-chain | Configure policy, commit bills, add payees, start/resume/revoke session, approve large actions, any normal Safe transfer/swap | — |
| **Agent session key** | Signer app, auto-unwrap inside session | Call module: `execute`, `queueAction` within policy | Send to arbitrary address, change policy, approve itself, resume after pause |
| **Guardian** | Signer app (separate key); optionally a third device | `pause`, `cancelQueuedAction`, `cancelPayee` | Resume, approve, configure, move funds |
| **Ops float** | Signer app (separate key) | Sign x402 payments from its own small balance | Touch Safe funds |

### 5.2 Permission tiers (amount-based)

| Assets (BASE_SEPOLIA profile) | Assets (ANVIL profile) | AUTONOMOUS | OWNER_APPROVED |
|---|---|---|---|
| ≤ 0.10 USDC | ≤ 10 USDC | Immediate | Safe approval, then immediate |
| > micro and ≤ 1.00 | > 10 and ≤ 100 | Queue → 180 s delay → execute; cancellable | Safe approval + mandatory queue |
| > 1.00 and ≤ hard max | > 100 and ≤ 2,500 | Safe approval (2-of-2), then execute | Same |
| > hard max | > 2,500 | Always reverts | Always reverts |

READ_ONLY: no agent execution at all.

**Payments add rules on top of tiers** (Section 7): bill must be committed, unpaid, inside its pay window; payee must be ACTIVE and not sanctioned; amount must equal the committed amount exactly.

### 5.3 Why owners keep Send/Swap
Owners are sovereign: they sign normal Safe transactions (including send and swap) with threshold 2. The agent never gets those powers. The Kavachi app offers **Receive**, **Pay bill**, **Withdraw to owner**, and **Advanced (Safe transaction)** for owners, with reserve warnings (Section 15). The module's reserve rules bind the **agent**, not owners; if owners move reserved cash, the app shows a **shortfall** and the module blocks agent investing.

---

## 6. On-chain specification

### 6.1 Contracts

| Contract | Source | Notes |
|---|---|---|
| Safe v1.4.1 singleton + proxy factory | Official Safe deployment (verify addresses for chain 84532 in `safe-deployments` before use; deploy via Safe singleton factory on Anvil) | Holds funds. Owners = [owner, approver], threshold = 2 |
| `KavachiModule` | NEW, this spec | Only module enabled on the Safe. No proxy, no delegatecall, no generic execute |
| `KavachiDemoVault` ×2 | NEW, OpenZeppelin ERC4626 over USDC | Simulated yield from a labelled sponsor drip. Not real yield |
| `KavachiTestSanctionsList` | NEW, testnet only | `isSanctioned(address)`; owned by the Safe; labelled TEST in UI |
| `MockUSDC` | NEW, Anvil only | 6 decimals; never deployed to a public chain |
| USDC (Base Sepolia) | Circle | `0x036CbD53842c5426634e7929541eC2318f3dCF7e` |

Pinned toolchain: Solidity ^0.8.24, OpenZeppelin Contracts (pinned release), Foundry. Record runtime code hashes of every contract in the deployment manifest.

### 6.2 Module immutables

`safe`, `asset` (USDC, decimals = 6 checked), `deploymentChainId`, `vaultA`, `vaultB` (distinct, `asset()` == USDC, non-empty code), `sanctionsList`, `vaultCodeHashA/B`. Constructor rejects zero addresses, asset-as-vault, identical vaults.

### 6.3 How the module moves funds
Every financial effect goes through `ISafe.execTransactionFromModule(to, 0, data, Operation.Call)`.

| Purpose | Exact call(s) executed by the Safe |
|---|---|
| Deposit | `USDC.approve(vault, assets)` → `vault.deposit(assets, safe)` → `USDC.approve(vault, 0)` |
| Withdraw | `vault.withdraw(assets, safe, safe)` |
| Rebalance | Withdraw leg then deposit leg, atomically in one module call |
| Pay bill | `USDC.transfer(recipient, assets)` |
| Ops top-up | `USDC.transfer(opsFloat, assets)` |

Rules: value is always 0; operation is always `Call` (never `DelegateCall`); every `execTransactionFromModule` return value MUST be `true`; token/vault return values and observed balance deltas MUST match expectations; residual allowance MUST be 0 after deposit legs.

### 6.4 Policy (V4)

| Field | Type | BASE_SEPOLIA default | ANVIL default | Validation |
|---|---|---|---|---|
| permissionTier | enum | AUTONOMOUS | AUTONOMOUS | READ_ONLY / OWNER_APPROVED / AUTONOMOUS |
| operatingBufferRaw | uint256 | 5 USDC | 500 | — |
| microMaxRaw | uint256 | 0.10 | 10 | > 0 |
| delayedMaxRaw | uint256 | 1.00 | 100 | ≥ micro |
| hardMaxRaw | uint256 | 25 | 2,500 | ≥ delayed |
| dailyLimitRaw | uint256 | 40 | 4,000 | ≥ hard |
| delaySeconds | uint32 | 180 | 180 | 180–300 |
| maxShareSlippageBps | uint16 | 50 | 50 | ≤ 1,000 |
| guardian | address | — | — | ≠ 0, ≠ owners, ≠ session key |
| vaults[] (vault, maxExposureRaw) | array | A: 60, B: 60 | A: 6,000, B: 6,000 | ⊆ {vaultA, vaultB}, sorted, cap > 0 |
| payeeDelaySeconds | uint32 | 86,400 | 86,400 | 3,600–604,800 |
| payEarlyWindowSeconds | uint32 | 259,200 | 259,200 | ≤ 2,592,000 |
| opsFloat | address | — | — | ≠ 0, ≠ Safe, ≠ module, ≠ owners |
| opsDailyCapRaw | uint256 | 0.50 | 50 | ≤ dailyLimit |
| opsFloatMaxRaw | uint256 | 1.00 | 100 | ≥ opsDailyCap |

All amounts in USDC; code uses raw units (×10⁶). Session limit default: 80 (BASE_SEPOLIA) / 8,000 (ANVIL), set at `startSession`.

> These numbers are demonstration parameters, not advice, and are untested. Section 16 must exercise every boundary with messy and adversarial inputs before they are used even on testnet.

**Why two profiles:** public testnet USDC comes from rate-limited faucets (small amounts per request), so the public demo runs at 1/100 scale. The Anvil profile keeps realistic SME numbers for the arena and boundary tests. Both profiles use identical code; only the policy values differ.

Reference state (BASE_SEPOLIA): cash C = 50, reserved R = 15, buffer B = 5, vault A = 30, vault B = 20 → total ≈ 100 USDC of testnet funds needed.

### 6.5 Lifecycle functions

| Function | Caller | Effect |
|---|---|---|
| `configurePolicy(Policy)` | Safe | Validate; pause; clear session + queue (consumes nonce); `policyEpoch++` |
| `commitObligations(root, reservedRaw, count)` | Safe | Set Merkle root + declared total; pause; clear session + queue; `policyEpoch++` |
| `addPayee(payeeIdHash, account, labelHash)` | Safe | New immutable entry, `activeAt = now + payeeDelaySeconds`; fails if sanctioned |
| `cancelPayee(payeeIdHash)` | Safe, any Safe owner, guardian | Terminal CANCELLED; invalidates queue if it references this payee |
| `startSession(key, expiresAt, limitRaw)` | Safe | Requires configured + paused + no session; expiry ≤ now + 24 h; stays paused |
| `resume(expectedEpoch)` | Safe | Exact epoch; live session; unpause; `policyEpoch++` |
| `pause(reasonCode)` | Safe, any Safe owner, guardian | First call pauses, clears queue, `policyEpoch++`; repeat is idempotent |
| `revokeSession()` | Safe | Pause; clear session + queue; `policyEpoch++` |
| `approveAction(Action, validUntil)` | Safe | Digest-bound approval for OWNER-tier (and all tiers in OWNER_APPROVED) |
| `revokeActionApproval(digest)` | Safe | Clears one approval |
| `queueAction(Action, proof)` | Session key | Delayed tier only; reserves current nonce; one queue max |
| `cancelQueuedAction(digest)` | Safe, any Safe owner, guardian | Clear queue; consume nonce |
| `expireQueuedAction()` | Anyone, after `expiresAt` | Clear queue; consume nonce |
| `execute(Action, proof)` | Session key | All checks, then effects |

"Any Safe owner" is checked live with `ISafe.isOwner(msg.sender)`. Single owners can only **stop** things (pause, cancel queue, cancel payee); starting or loosening anything needs the Safe (threshold 2).

### 6.6 Actions

| Kind | sourceVault | destinationVault | minSharesOut | maxSharesBurn | obligationIdHash / payeeIdHash / recipient / dueAt |
|---|---|---|---|---|---|
| DEPOSIT | 0 | allowed vault | > 0 | 0 | all zero |
| WITHDRAW | allowed vault | 0 | 0 | > 0 | all zero |
| REBALANCE | allowed vault | other allowed vault | > 0 | > 0 | all zero |
| PAY_OBLIGATION | 0 | 0 | 0 | 0 | all set; recipient = registry address |
| OPS_TOPUP | 0 | 0 | 0 | 0 | recipient = policy.opsFloat; others zero |

HOLD is a planner outcome only; it never signs.

### 6.7 Checks in `execute` (order matters: all before any external call)

1. Chain ID = deployment chain; module enabled on Safe; not paused; caller = session key; session live; tier ≠ READ_ONLY.
2. `expectedNonce == actionNonce`, `policyEpoch` current, deadline valid (immediate ≤ now + 120 s; approved ≤ now + 900 s; queued per 6.8; all ≤ session expiry).
3. Shape valid for kind (6.6); assets > 0; assets ≤ hardMax.
4. UTC-day gross + assets ≤ dailyLimit; session gross + assets ≤ session limit.
5. Tier route satisfied: immediate, or matching queue ready, or live approval for this digest (and queue too when both apply).
6. Kind-specific:
   - **DEPOSIT**: `C − assets ≥ R + B`; destination exposure after ≤ cap; preview within slippage and ≥ minSharesOut.
   - **WITHDRAW**: `maxWithdraw` sufficient; burn ≤ maxSharesBurn.
   - **REBALANCE**: `C ≥ R` (not underfunded); both legs within bounds; destination exposure ≤ cap.
   - **PAY_OBLIGATION**: Merkle proof valid for (obligationIdHash, payeeIdHash, assets, dueAt) under current root; `!paid[obligationIdHash]`; `now ≥ dueAt − payEarlyWindow`; payee ACTIVE (`now ≥ activeAt`, not cancelled); `recipient == payees[payeeIdHash].account`; `!sanctions.isSanctioned(recipient)`; `C ≥ assets`; `assets ≤ R`.
   - **OPS_TOPUP**: `recipient == opsFloat`; `assets ≤ microMax`; ops day usage + assets ≤ opsDailyCap; `USDC.balanceOf(opsFloat) + assets ≤ opsFloatMax`; `C − assets ≥ R + B`.
7. Effects via Safe (6.3); verify deltas; then update: `actionNonce++`, daily/session usage += assets, for PAY: `paid[id] = true`, `R −= assets`; clear consumed queue/approval; emit event.

A sanctions-list call that reverts MUST fail closed (revert `SanctionsCheckFailed`).

### 6.8 Queue (cancel window)
Same semantics as v2, now on the module:
- One pending queue. `queueAction` validates all hard rules (including proof, payee status, sanctions) at queue time; stores digest + exact action + session ID; `readyAt = now + delaySeconds`; `expiresAt = action.deadline` where `now + delay + 30 ≤ deadline ≤ now + delay + 900` and ≤ session expiry.
- No funds move at queue time; usage is not charged.
- `execute` of a delayed-tier action without its matching ready queue reverts `QueueRequired`, even with an approval.
- At execution all checks rerun against current state. If anything moved outside bounds, it reverts; the app cancels/lets it expire and re-plans with new consent.
- Cancel/expire/invalidate consume the reserved nonce so the same digest can never execute later.

### 6.9 Reserve accounting

```
C = USDC.balanceOf(safe)        R = reservedRaw        B = operatingBufferRaw
V = Σ vault.convertToAssets(shares)  (marked, not guaranteed cash)

if C ≥ R: free = C − R ; reserved = R ; sweepable = max(C − R − B, 0)
else    : backed = C ; free = 0 ; shortfall = R − C
deployed = V ; total = C + V
```

- Paying a bill lowers C and R by the same amount, so free cash is unchanged.
- Owners may move funds by normal Safe transactions; that can create a shortfall. While underfunded, the module allows WITHDRAW and PAY_OBLIGATION only.
- The module trusts the declared R (set by 2-of-2). The app MUST recompute R from its private bill records and refuse to plan or sign if they differ.

### 6.10 Events and errors
See Appendix A for the full interface. Key additions over v2: `PayeeAdded`, `PayeeCancelled`, `ObligationsCommitted`, `ObligationPaid`, `OpsTopUp`; errors `PayeeNotActive`, `PayeeSanctioned`, `SanctionsCheckFailed`, `BadObligationProof`, `ObligationAlreadyPaid`, `PayWindowNotOpen`, `RecipientMismatch`, `OpsCapExceeded`, `Underfunded`, `SafeCallFailed`.

### 6.11 What the module does NOT protect
- Owners acting together (threshold 2) can do anything with the Safe. That is intended sovereignty.
- External vault bugs, illiquidity, or malicious upgrades of third-party tokens.
- Gas spent on reverted transactions.
- A compromised signer app during an authorised owner signing session.
- Censorship or delayed inclusion of a pause transaction.

---

## 7. Obligations, payees and compliance

### 7.1 Obligation (bill) record

| Field | Where | Notes |
|---|---|---|
| obligationId (opaque ASCII) | private app DB | `obligationIdHash = keccak256(UTF8(id))` |
| payeeIdHash | app DB + leaf | references registry entry |
| amountRaw | app DB + leaf | exact USDC raw units |
| dueAt | app DB + leaf | unix seconds, uint64 |
| label, invoice number, attachment | private app DB only | never on-chain, never to a cloud model |
| status | app DB | DRAFT → COMMITTED → (QUEUED) → PAID / CANCELLED |

### 7.2 Commitment (Merkle)
- Leaf = `keccak256(bytes.concat(keccak256(abi.encode(bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 amountRaw, uint64 dueAt))))` — compatible with OpenZeppelin `StandardMerkleTree` and `MerkleProof` (sorted pairs).
- Only unpaid COMMITTED bills are included. Builder rejects duplicates and paid IDs.
- `reservedRaw` = sum of included amounts (checked integer sum). Empty set: `root = keccak256("KAVACHI_EMPTY_OBLIGATIONS_V4")`, `reservedRaw = 0`.
- Commit is a Safe transaction (2-of-2) showing count, total, and the list of changes since the last commitment.
- A commitment proves which records the owners approved, not that the real-world invoices are genuine.

### 7.3 Payee registry
- Entry: `payeeIdHash`, `account`, `labelHash`, `activeAt`, `cancelled`.
- **Immutable**: an address change is a *new* payee with a new cooling period. There is no "edit address".
- **Cooling period** (default 24 h): new payees cannot be paid until `activeAt`. Any single owner or the guardian can cancel during or after cooling.
- **Sanctions**: checked when adding and again on every payment and queue.
- **Display**: the app shows label + full address in 4-character groups, with the first and last 6 characters highlighted, and a payee-specific identicon. Adding a payee requires typing the last 6 characters from an *independent* source (not paste from history).

This directly targets two real fraud patterns: supplier "bank details changed" redirection (BEC) and lookalike-address poisoning (Section 2.2).

### 7.4 Sanctions screening
- Interface: `isSanctioned(address) → bool` (same shape as the Chainalysis on-chain sanctions oracle).
- Testnet: `KavachiTestSanctionsList`, owned by the Safe, labelled **TEST LIST** everywhere it appears. It exists to demonstrate the blocked-payee path, not to claim real screening.
- Production path: the Chainalysis oracle is a free public contract on several EVM chains; our research did not confirm a Base deployment, so this MUST be verified per chain before any production plan. Off-chain API screening MAY be added as an extra advisory check.
- Failure of the screening call fails closed.

### 7.5 Audit trail
Every executed action produces a **decision passport** (Section 9, M8): request and model identity, candidate set, chosen candidate, AHEM report, policy/mandate/action digests, approvals or queue events, tx hash, receipt block, balance deltas. Export: CSV (accountant) and JSON (machine). Private labels are included only in the owner's local export, never in public evidence.

---

## 8. Hashing contract V4

All hashes are Keccak-256 of Solidity `abi.encode` (not packed, not JSON). `TAG(x) = keccak256(UTF8(x))`. Old V2/V3 digests MUST be rejected.

```
PolicyHash = keccak256(abi.encode(TAG("KAVACHI_POLICY_V4"),
  uint8 permissionTier, uint256 operatingBufferRaw, uint256 microMaxRaw,
  uint256 delayedMaxRaw, uint256 hardMaxRaw, uint256 dailyLimitRaw,
  uint32 delaySeconds, uint16 maxShareSlippageBps, address guardian,
  bytes32 vaultLimitsHash, uint32 payeeDelaySeconds, uint32 payEarlyWindowSeconds,
  address opsFloat, uint256 opsDailyCapRaw, uint256 opsFloatMaxRaw))

vaultLimitsHash = keccak256(abi.encode((address,uint256)[] sortedVaultLimits))

SessionId = keccak256(abi.encode(TAG("KAVACHI_SESSION_V4"), uint256 chainId,
  address safe, address module, uint256 sessionStartEpoch, address sessionKey,
  uint64 sessionExpiresAt, uint256 sessionLimitRaw))

MandateHash = keccak256(abi.encode(TAG("KAVACHI_MANDATE_V4"), uint256 chainId,
  address safe, address module, uint256 policyEpoch, bytes32 policyHash,
  bytes32 obligationsRoot, uint256 reservedRaw, bytes32 sessionId,
  address sessionKey, uint64 sessionExpiresAt, uint256 sessionLimitRaw, bool paused))

Action = (uint8 kind, address sourceVault, address destinationVault, uint256 assets,
  uint256 minSharesOut, uint256 maxSharesBurn, bytes32 obligationIdHash,
  bytes32 payeeIdHash, address recipient, uint64 dueAt,
  uint256 expectedNonce, uint256 policyEpoch, uint64 deadline)

ActionDigest = keccak256(abi.encode(TAG("KAVACHI_ACTION_V4"), uint256 chainId,
  address safe, address module, bytes32 mandateHash, bytes32 sessionId, Action action))

CandidateId = keccak256(abi.encode(TAG("KAVACHI_CANDIDATE_V4"),
  bytes32 snapshotBlockHash, bytes32 actionDigest))

ObligationLeaf = keccak256(bytes.concat(keccak256(abi.encode(
  bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 amountRaw, uint64 dueAt))))
```

- No session: sessionId = 0, sessionKey = 0, expiry = 0, limit = 0.
- Merkle proofs are witnesses, not part of the digest; the leaf fields are.
- Owner approvals are Safe transactions; the Safe's own EIP-712 `SafeTx` hash is what owners sign. The `ActionDigest` itself is not labelled EIP-712.
- uint256 values travel as base-10 strings without signs or leading zeros; reject values ≥ 2²⁵⁶. Hex is lowercase in transport.
- Reference vectors: implementation MUST ship ≥ 5 vectors (one per action kind) checked in Solidity, Rust and Kotlin.

---

## 9. Off-chain modules

Rust financial logic lives in `fusion-core`; Android code under `fusion-core-android`. Generated Kotlin bindings are outputs, never hand-edited.

| Module | Inputs | Outputs | Owner |
|---|---|---|---|
| M0 Setup & mandate | Owner intent, deployment manifest | Policy drafts, Safe tx requests | App + signer review |
| M1 Bills & payees | Owner entries, CSV import | Draft bills, payee requests, Merkle commitment | App |
| M2 Perception & reflex | Pinned RPC, x402 data | Block snapshot or halt | App worker + signer |
| M3 Candidates & economics | Snapshot, committed bills | ≤7 candidates + HOLD | Rust |
| M4 Local planner | Candidates, bounded goal | Candidate ID / HOLD + AHEM report | Inference app |
| M5 Policy preview & approval | Candidate, fresh snapshot | Decoded preview, exact action, tier route | Rust + signer UI |
| M6 Execution & reconciliation | Durable requests, chain events | Reconciled status | App worker + signer |
| M7 Ops float & x402 | Data needs, ops balance | Paid data responses, top-up candidates | App + signer |
| M8 Evidence | All of the above | Decision passports, metrics | App |

### M0 Setup & mandate
- **Deployment manifest** (signed build config, never model output): chainId (31337 or 84532 only), RPC URL, Safe, module, USDC, vaults, sanctions list, code hashes, start block, profile name. Verify `eth_chainId` and every code hash before showing "Connected".
- **Safe creation**: owners [owner, approver], threshold 2, enable module, set guardian. Each step is a Safe/owner transaction shown in the signer review.
- **Policy drafts**: decimal strings parsed to raw integers; reject scientific notation, negatives, over-precision, overflow. Show draft vs confirmed separately. Model may *suggest* a draft; Rust validates; owners confirm.

### M1 Bills & payees
- Bill entry: manual form or CSV import (columns: id, payee, amount, due date). Invoice PDFs MAY be parsed **locally** by the model into a *draft* only; every field is shown for human confirmation and the payee must match an existing registry entry by ID, never by an address found in the document.
- Payee entry: label, address, independent confirmation of last 6 characters, sanctions pre-check, then a Safe tx `addPayee`. Status shown as COOLING until `activeAt`.
- Commitment builder: sorted, deduplicated, excludes paid; computes root + total; produces a change list (added/removed/changed) for the review.
- Any change affecting R blocks agent planning until the new commitment is confirmed on-chain.

### M2 Perception & reflex
- **Snapshot**: one block number + hash for: Safe USDC balance, vault shares, `convertToAssets`, `previewDeposit/Withdraw`, `maxWithdraw`, module policy/state/queue, `paid[]` for due bills, payee status for referenced payees, ops float balance. Missing or inconsistent reads → no snapshot (never substitute zero).
- **TTL**: 30 s or 5 blocks; fee quote 15 s; time rollback never extends a TTL.
- **Risk feed** (via x402, M7): USDC price and vault rates. Demo trigger: price ≤ 0.98 → latch. Stale/missing feed → latch for "cannot assess", not "depeg". Arena-injected values are labelled as injected.
- **Reflex order**: increment incident epoch → persist latch → block signing → submit guardian `pause` → record timings → ask model for an explanation afterwards. Target p95 ≤ 250 ms to persisted latch (measured, not assumed).

### M3 Candidates & economics
- Integer raw units only (checked U256). Candidate order: (1) due bills inside pay window, earliest due first; (2) safety withdrawals if cash < R + B; (3) ops top-up if float below threshold and data needed; (4) deposit/rebalance if net benefit positive; (5) HOLD (always present).
- Yield estimate: `gross = floor(assets × rateBps × horizonSec / (10,000 × 31,536,000))`; `net = gross − buffered fee`; minimum net benefit default 0.01 USDC (ANVIL) / 0.0001 USDC (BASE_SEPOLIA). Rates are **scenario values** from the data feed (simulated vault yield), never presented as guaranteed APY.
- Each candidate carries: actionDigest, candidateId, snapshot block hash, policyEpoch, root/R, nonce, bounds, deadline, fee assumptions, reason code. ≤7 + HOLD. Deterministic ordering.

### M4 Local planner
- Request: requestId, routeEpoch, snapshot ID, capability digest, candidate list (structured facts only), bounded user goal (untrusted), deadline.
- Response: requestId, routeEpoch, `candidateId` or `"HOLD"`, `reasonCode` from a fixed vocabulary, timings, AHEM report. Strict JSON; unknown keys, unknown IDs, trailing text → one format retry, then HOLD.
- Limits: one job at a time, ≤2,048 input tokens, ≤256 output tokens, 25 s total budget. Late results after halt or epoch change are discarded.
- The model never sees keys, raw signed bytes, or full private invoice text unless the owner explicitly asked it to parse that document, locally.

### M5 Policy preview & approval
- **Preview is pure**: no nonce allocation, no key load, no broadcast. Returns every rule outcome, before/after buckets, tier, route, exact action + digest.
- **Share bounds**: `minSharesOut = floor(previewDeposit × (10,000 − tol) / 10,000)`; `maxSharesBurn = ceil(previewWithdraw × (10,000 + tol) / 10,000)`.
- **Routes**: IMMEDIATE → signer; DELAYED → `queueAction` then wait for chain `readyAt`, fresh preflight, `execute`; OWNER → build Safe tx `approveAction`, collect two signatures, then execute.
- Any changed field (amount, vault, payee, recipient, bounds, deadline) = new digest = new consent.

### M6 Execution & reconciliation
- One worker per treasury; one unresolved tx per EOA (agent, owner, approver, guardian, ops float each tracked independently).
- Persist request → signer persists prepared bytes → worker persists handle → signer persists BROADCAST_REQUESTED → send. Network error after that = UNKNOWN, never "failed".
- Success requires: receipt status 1, correct `to`/selector/sender/nonce, matching module event (digest, nonce, epoch, session), and matching transaction-level balance deltas. Demo rule: 5 confirmations; keep re-checking ancestry for 64 blocks.

### M7 Ops float & x402
See Section 14.

### M8 Evidence
- Decision passport per decision (including HOLD).
- Metrics: ASR, proposal ASR, FBR, legitimate acceptance, completion, availability, latencies, fees — with raw counts and denominators (Section 16).
- Judge bundle: contract addresses, verified source links, tx hashes per scenario, sanitized logs, build/model hashes.

---

## 10. Android, MNN and process isolation

Carried from v2 with no change in intent. Key rules:

**Build**
- Wallet repo keeps Gradle 8.13, AGP 8.11.0, Kotlin 2.0.21, Java 17, SDK 36, minSdk 24. Pin Rust toolchain and `Cargo.lock`.
- MNN SDK built in its own repo, published as a checksum-pinned local Maven artifact; consumed only by `:inference`.
- arm64-v8a first. Inspect final APK native libraries and `DT_NEEDED`; no unexplained `pickFirst`.
- Fix the SDK CI import check to compare **ABI-matched** bridge/library pairs before changing any native declaration.

**Three apps, three UIDs**
- Same release certificate, no `sharedUserId`. Bound services exported only behind a custom signature permission, plus per-call caller UID/package/certificate checks captured on the Binder thread.
- `:inference` has **no INTERNET permission**. Models provisioned through a narrow, hash-verified chunked API (no URLs, no paths, no shell).
- `:signer` owns all keys. Keys are software secp256k1 wrapped with Android Keystore AES-GCM; owner/approver unwrap needs fresh biometric/device auth; agent session key unwraps automatically only inside a live session. This is at-rest protection, not hardware Ethereum signing.

**MNN runtime**
- One native session, one in-flight generation; state machine UNPROVISIONED → LOADING → READY → GENERATING → READY; timeouts go to DRAINING/FAILED, never straight back to READY.
- Use the blocking `chat()` path first; fresh context per financial decision.
- Model choice: start with the smallest SDK-listed model (e.g. a Qwen2.5-0.5B-Instruct MNN build) and keep the smallest one that passes the frozen candidate tests on the real device. Pin model revision, weights, tokenizer, config, license.
- Record device, Android version, cold load, p50/p95 latency, peak memory, thermal behaviour over 20 sequential runs. Health UI shows measured values or "not measured" — never fixed labels.

**Privacy**
- Default LOCAL_ONLY. Optional public education helper accepts only a fixed topic enum and has no financial tools; disabled until its egress tests pass.
- Network capture test: inference UID produces zero packets during planning.

---

## 11. AHEM integration

Unchanged policy from v2; summarised:

| Mode | Behaviour | UI label |
|---|---|---|
| OFF | No evaluation | "AHEM off" |
| SHADOW | Real callbacks, scores recorded, no effect on output | "AHEM observing (not enforcing)" |
| ENFORCED | Veto stops/discards output before any tool use; missing coverage or errors → HOLD | "AHEM enforcing" + coverage % |

- The shipped SDK `libllm.so` does not export AHEM's hook symbols; ENFORCED requires a pinned, patched MNN build with a versioned veto-only callback ABI (`ahem_hook_abi_version`, `register/unregister`, coverage report).
- Calibration MUST use activations from the exact shipped model/quantization, with held-out scenario families; store the full projection (w, scale, bias, thresholds) and dataset hashes.
- AHEM ALLOW never authorises anything; module, signer and Rust checks always run.
- Required proof for ENFORCED: forced-veto test, coverage on valid decodes, fail-closed on missing callback/NaN/panic/stale generation, lifecycle safety, held-out ASR/FBR, ablation (MNN only vs + policy vs + AHEM).
- Release default: **SHADOW**, promoted to ENFORCED only when every research gate passes. If AHEM adds no measured benefit, say so.

---

## 12. Signer protocol 3.0.0

Transport: authenticated Binder, typed JSON payloads ≤64 KiB, unknown fields rejected. No generic "sign" endpoint exists.

### 12.1 Automatic operations (agent path)

| Operation | Input | Signer behaviour |
|---|---|---|
| `capabilities` / `status` | — | Versions, role addresses, latch state, unresolved attempts |
| `prepare` | requestId, idempotencyKey, operation (QUEUE / EXECUTE / OPS_PAY_X402), exact Action (+ proof), expected digest/mandate, snapshot ID, expiry | Reload protected config; verify chain + code hashes; read module state; recompute all hashes; run independent checks (tier, caps, reserves, payee, sanctions, bounds); simulate exact calldata from the real sender; persist bytes; return attemptId, EOA, nonce, txHash |
| `broadcast` | attemptId, txHash | Re-check latch/authority/expiry; persist BROADCAST_REQUESTED; send stored bytes |
| `reconcile` | attemptId | Canonical receipt/nonce status |
| `halt` | incidentId, reason, generation | Persist latch first, then guardian pause |
| `retireUnsent` | attemptId + proofs | Only if BROADCAST_REQUESTED never persisted and authority revoked on-chain |

Same idempotency key + same request = same attempt. Same key + different request = error. The signer may refuse; it never edits an action to make it pass.

### 12.2 Owner and approver operations (trusted UI only)
- Safe transactions: `configurePolicy`, `commitObligations`, `addPayee`, `startSession`, `resume`, `revokeSession`, `approveAction`, `revokeActionApproval`, `enableModule`, and owner manual transfers.
- The app sends only a **request ID**. The signer app fetches the canonical request, reads chain state, decodes the Safe transaction, renders its own review screen, requires biometric auth, signs the `SafeTx` EIP-712 hash, and journals the signature.
- **Second signature**: delivered to the approver device by (a) Safe Transaction Service where available for the chain (verify for 84532), or (b) QR hand-off of the unsigned SafeTx between the two signer apps. The approver's signer independently decodes and shows the same review.
- Single-owner stop actions (`pause`, `cancelQueuedAction`, `cancelPayee`) are signed by one owner directly to the module, still through the trusted screen, without waiting for the second owner.

### 12.3 Guardian and ops float
- Guardian: `pause(reason)`, `cancelQueuedAction(digest)`, `cancelPayee(id)` only; own nonce journal and fee ceiling.
- Ops float: signs x402 payment authorisations (EIP-3009 `transferWithAuthorization` for USDC) only to allowlisted x402 endpoints' `payTo` addresses, per-request price ≤ configured max, daily total ≤ cap (Section 14).

### 12.4 Health states
UNPROVISIONED, LATCHED, READY, PREPARED, BROADCAST_REQUESTED, UNKNOWN, MINED, CONFIRMED, RECONCILED, REVERTED, RETIRED_UNSENT, INVALIDATED_KEY_RETIRED. Signer always starts LATCHED; only a confirmed `resume` receipt clears it.

---

## 13. Storage, durability and recovery

### 13.1 Stores
- App DB (Room/SQLite, WAL, FULL sync): bills, payees (mirror), commitments, policy views, decisions, plans, worker attempts, outbox, evidence, x402 receipts.
- Signer DB (separate UID): protected config, key ciphertext references, per-EOA nonce/fee reservations, signed bytes, Safe signatures, latch.
- Inference: verified model files + ephemeral request context only.

### 13.2 Tables (additions to v2 in bold)
- `obligation(id, idHash, revision, payeeIdHash, amountRaw, dueAt, privateLabel, status, commitmentRoot, paidTx)`
- **`payee(payeeIdHash, account, label, addedTx, activeAt, status, sanctionsCheckedAt)`**
- **`commitment(root, reservedRaw, count, safeTxHash, confirmedBlockHash)`**
- `policy_view(chainId, safe, module, epoch, policyHash, root, reservedRaw, confirmedBlockHash)`
- `decision(id, requestId, routeEpoch, snapshotHash, candidateSetDigest, selectedId, modelIdentity, ahemReport, status)`
- `plan(id, actionDigest UNIQUE, exactActionJson, proofJson, requiredTier, state, queueTx, readyAt, expiresAt)`
- `worker_attempt(idempotencyKey UNIQUE, canonicalRequestHash, signerAttemptId, operation, state, txHash)`
- **`safe_tx(safeTxHash UNIQUE, purpose, payloadHash, sigOwner, sigApprover, state, execTx)`**
- **`x402_payment(id, endpoint, payTo, priceRaw, authNonce, state, responseHash)`**
- `outbox`, `evidence`, signer `protected_attempt`, `protected_latch` as in v2.

### 13.3 States
- Plan: DRAFT → PLANNED → AWAITING_OWNERS | QUEUE_SUBMITTING | READY → (QUEUED_WAIT → READY) → EXECUTING → RECONCILED; terminals HELD, BLOCKED, CANCELLED, EXPIRED, INVALIDATED. UNKNOWN is an overlay, never terminal.
- Safe tx: DRAFTED → OWNER_SIGNED → APPROVER_SIGNED → SUBMITTED → CONFIRMED | REVERTED | ABANDONED.
- Bill: DRAFT → COMMITTED → SCHEDULED (queued) → PAID | CANCELLED.

### 13.4 Crash rules (unchanged core)
Same idempotency key after any crash; PREPARED handles returned, never re-noncing; BROADCAST_REQUESTED → UNKNOWN → reconcile by hash/nonce before any new attempt; queue state always re-read from chain; corrupt/unsupported storage → latch, preserve files, full reconciliation from deployment block. Reinstall never auto-resumes.

---

## 14. Ops float and x402 payments

**Goal:** show the agent as a real economic participant — it pays for the data it needs — without ever letting it near the treasury.

**Design**
- Ops float = separate EOA (key in signer app). Funded only by module `OPS_TOPUP` (immediate tier, ≤ micro per top-up, ≤ `opsDailyCap` per day, float balance ≤ `opsFloatMax`).
- The app worker calls x402-gated HTTP endpoints. On `402 Payment Required`, it parses the payment requirements and asks the signer to authorise payment from the ops float.
- Signer checks: endpoint in allowlist; network = `eip155:84532`; asset = Base Sepolia USDC; `payTo` matches the endpoint's pinned address; price ≤ per-request max (default 0.01 USDC); daily ops spend ≤ cap. Otherwise refuse.
- Testnet facilitator: the x402.org facilitator is testnet-only; CDP's facilitator also lists Base Sepolia (Sources). Choose one, pin it, record settlement tx hashes.

**Data service (team-operated, labelled)**
- Endpoints: `/v1/rates` (simulated vault rates), `/v1/usdc-price` (scenario price; arena can inject 0.97), `/v1/risk` (flags).
- Responses signed by the service key; the app verifies signature and freshness. A paid response is still **untrusted input**: it can only *trigger halts* or *change candidate ranking*; it can never add an action type, address or amount.

**Arena tie-in:** "x402 overcharge" scenario — the service returns a price above the cap; the signer refuses; no payment.

**Blast radius:** worst case loss = `opsFloatMax` (1 USDC on BASE_SEPOLIA profile).

---

## 15. UX specification

### 15.1 Design principles

1. **One story:** "Money you owe is untouchable. The agent pays your approved bills and invests only what's left."
2. **Proof over promises:** every status that matters links to its on-chain evidence. No badge without a source.
3. **Calm by default, loud when it matters:** routine actions are quiet feed items; anything needing a human is impossible to miss.
4. **Plain words first, exact data one tap away:** "Pay Acme 12.50 USDC" on top; full address, raw units, digest on expand.
5. **Humans stop, both humans start:** any one owner can stop anything instantly; loosening anything needs both.
6. **Never colour alone:** every state has icon + text + colour.

### 15.2 Glossary (UI words → system terms)

| UI word | System term |
|---|---|
| Bill | Obligation |
| Verified vendor | ACTIVE payee |
| New vendor (waiting 24 h) | Payee in cooling period |
| Money you owe | Reserved obligations R |
| Safety cushion | Operating buffer B |
| Available | Free cash (C − R) |
| Can be invested | Sweepable (C − R − B) |
| Invested | Deployed vault holdings V |
| Agent shift | Session (with expiry) |
| Instant | Immediate tier |
| Waits 3 min (you can cancel) | Delayed tier / queue |
| Needs both of you | Owner tier / Safe approval |
| Holding | HOLD decision |
| Stopped | Local latch (+ on-chain pause when confirmed) |

### 15.3 Visual language

| State | Icon | Colour | Text pattern |
|---|---|---|---|
| OK / done | check-circle | green | "Paid", "Confirmed" |
| Waiting | clock | amber | "Waits 2:41 · cancel" |
| Needs you | hand | blue | "Needs both of you" |
| Blocked | shield-x | red | "Blocked by: reserve rule" |
| Unknown | question-circle | grey | "Checking the chain…" |
| Test / simulated | flask | purple outline | "TEST LIST", "Simulated yield", "Injected price" |

- Amounts: tabular numerals, 2 decimals in summaries ("12.50 USDC"), exact 6 decimals + raw integer in details.
- Addresses: groups of 4, first and last 6 characters bold, identicon; full address always reachable; never truncated in review screens.
- Time: device time for humans; chain time labelled "chain time" wherever a contract decides.
- Dark mode and light mode from tokens; contrast ≥ WCAG AA.

### 15.4 Navigation

```
Top: Health strip (always visible)
Body: tab content
Bottom nav: Home · Bills · Agent · Activity · More
Floating: [■ Stop agent] (long-press 1 s) — visible whenever the agent is running
```

Separate app (signer): Review screen, Approver inbox, Keys & roles.

### 15.5 Health strip
One line, tappable for details:

`● Agent running · shift ends 21:40 · Model: Qwen 0.5B (p95 3.1 s) · AHEM: observing · Chain: Base Sepolia #12,345,678 (4 s ago)`

States: Running / Stopped (local) / Stopped (on-chain confirmed) / Waiting for both of you to restart / Offline / Wrong network (blocking, red). Missing metrics say "not measured".

### 15.6 Onboarding wizard (6 steps, progress bar)

| Step | Screen content | Done when |
|---|---|---|
| 1 Create treasury | Explain "two-person control". Add approver (scan QR from approver's signer app). Create Safe (threshold 2), enable Kavachi module, set guardian | Safe + module confirmed on-chain (links shown) |
| 2 Fund it | Receive card: QR + full address + network warning ("Base Sepolia only"). Testnet: faucet link + "testnet USDC has no value" note | Balance > 0 seen at a confirmed block |
| 3 Add vendors | Add payee flow (15.10). Explains the 24 h wait and why | ≥ 1 payee submitted (cooling is fine) |
| 4 Add bills | Manual / CSV / "Read an invoice on this phone" (local AI draft) | Bills reviewed → commitment submitted |
| 5 Choose autonomy | Three preset cards + limits table (15.18) | Policy submitted |
| 6 Start agent shift | Shift length (≤ 24 h), shift budget; both owners sign start + resume | Health strip shows Running |

Each step can be left and resumed. Pending on-chain steps show "Waiting for confirmation" with a link, never a fake check.

### 15.7 Home

Top to bottom:

1. **Treasury header**: "SME operating treasury" · network chip · TEST chip (testnet) · total value.
2. **Three bucket cards** (horizontal on wide screens, stacked on phones):
   - **Available** — big number; sub-rows: Safety cushion, Can be invested.
   - **Money you owe** — big number; "6 bills · next due Fri"; coverage bar; if shortfall: red banner "Short by 3.20 USDC — agent will not invest until fixed" + [Fix] (opens withdraw-from-vault suggestion).
   - **Invested** — big number; Vault A / Vault B rows with value, share count, "withdrawable now"; "Simulated yield" label.
3. **Agent card**: what the agent is doing now in one sentence ("Next: pay Acme Hosting 0.75 USDC — waits 3 min after queuing"), plus last decision ("Held: fees higher than yield").
4. **Needs you** (only if non-empty): approvals, vendor cooling finished, unknown transactions.
5. **Action bar**: `[↓ Receive] [→ Pay a bill] [↑ Withdraw to owner] [⋯ Advanced]`
6. **Upcoming bills** (next 3) with due chips.
7. **Recent activity** (last 5) with status chips.

Empty state (new treasury): illustration-free text: "No money yet. Receive USDC to start." + Receive button.

### 15.8 Bills

- Segments: **Due soon** (inside pay window) · **Upcoming** · **Paid** · **Drafts**.
- Row: vendor name + verified badge, amount, due date chip (Overdue red / Due in 2 d amber / Upcoming neutral), status chip (Draft / Protected / Scheduled / Paid).
- **Unprotected changes banner** (sticky): "3 changes are not protected yet. The agent is paused for bills until you both confirm." [Review & protect]
- Add bill: form (vendor picker from verified/cooling vendors only — no free address field), amount, due date, note. Or CSV import with per-row validation errors shown inline. Or "Read invoice" (local model fills a draft; every field highlighted for confirmation; any address in the document is shown as "ignored — vendors come from your verified list").

**Bill detail:** amount (exact), vendor (full address, verified since…), due date, status timeline (Drafted → Protected at block … → Scheduled → Paid tx …), [Pay now] (owner manual), [Cancel bill] (creates a new commitment).

**Review & protect (commit) screen:** change list (added / removed / amount changed, old → new), new total owed, effect on "Can be invested", then [Review in signer]. Copy: "Both of you must confirm. Until then the agent won't touch bills or invest."

### 15.9 Vendors (payees)

- List: name, identicon, short address (bold ends), status: **Verified** / **New — usable in 18 h** (countdown) / **Cancelled**.
- Vendor detail: full address grouped, added by, added at, usable from (chain time), bills using this vendor, [Cancel vendor] (any one owner; red; confirm).
- No edit button. "Address changed?" link explains: "Add them as a new vendor. Fraudsters often ask businesses to 'update bank details'. The 24 h wait gives you time to call the vendor on a number you already know."

### 15.10 Add vendor flow (anti-fraud)

1. Name.
2. Address: paste or scan QR.
3. **Independent check**: "Ask the vendor for the last 6 characters of their address by phone or another channel you already trust. Type them here." Mismatch → stop with explanation.
4. Lookalike check: if the address shares first 4 + last 4 characters with any address seen in this Safe's history but is not identical → red warning "This looks like an address you've used before but is different. This is a common scam (address poisoning)."
5. Sanctions pre-check result (TEST LIST label on testnet).
6. Summary → [Review in signer] (both owners). Success state: "New vendor — usable after Sat 10:42 (chain time)".

### 15.11 Agent tab

- **Plan feed** (default): chronological cards — proposals, holds, executions, halts. Each has a reason chip (e.g. `BILL_DUE`, `SURPLUS_ABOVE_THRESHOLD`, `FEES_EXCEED_YIELD`, `RISK_HALT`) and "Why?" expander showing the candidate list with the chosen one highlighted and the rule outcomes.
- **Ask** (chat): questions answered by the local model ("How much can be invested?", "Why did you hold?"). Chat cannot create actions directly; when a request implies action, the reply shows the matching candidate card or "I can't do that: vendors must be verified first."
- Model/AHEM chip at the top with measured values.

### 15.12 Decoded action card (extends `PreFlightGuardrailCard`)

```
┌──────────────────────────────────────────────┐
│ PAY BILL                     ⏱ Waits 3 min   │
│ Pay Acme Hosting 0.75 USDC                   │
│ Bill INV-1042 · due Fri 24 Oct               │
│                                              │
│ To  0x1a2B c3D4 … 9f4E 5a6B  ✔ Verified 3 d  │
│ Money you owe   15.00 → 14.25                │
│ Available       35.00 → 35.00 (unchanged)    │
│ Network fee     ≤ 0.0004 ETH (estimate)      │
│ Checks  ✔ reserve ✔ daily limit ✔ vendor     │
│         ✔ sanctions (TEST LIST) ✔ amount     │
│ Ready at 14:03 (chain time) · expires 14:15  │
│ [ Details ]          [ Review in signer ]    │
└──────────────────────────────────────────────┘
```

Variants:

| Kind | Headline pattern | Extra rows |
|---|---|---|
| PAY_OBLIGATION | Pay {vendor} {amount} | bill ID, due date, vendor verified since, R before → after |
| DEPOSIT | Invest {amount} in {vault} | min shares, exposure after vs cap, est. net benefit (simulated), break-even time |
| WITHDRAW | Move {amount} from {vault} to cash | max shares burned, reason (e.g. refill cushion) |
| REBALANCE | Move {amount} from {A} to {B} | both share bounds, rate difference, net benefit |
| OPS_TOPUP | Top up agent data wallet {amount} | float balance before → after, daily ops spend |
| HOLD | Holding — no action | reason, what would change the decision |

Tier badge: **Instant** / **Waits 3 min (cancellable)** / **Needs both of you**. Primary button text matches route: "Run now", "Queue (3 min wait)", "Ask approver". Card generated only from validated fields; never from model prose.

**Details** expander: full addresses, raw amounts, share bounds, nonce, epoch, digest, candidate ID, snapshot block.

### 15.13 Signer review screen (signer app, owner/approver)

- Title: "Review request from Kavachi" + requesting app identity.
- Rebuilt independently from chain + request; shows the same card fields **plus** decoded Safe transaction (to, function, arguments) and SafeTx hash.
- Digest match chip: "Matches what Kavachi showed ✔" or blocking red "Mismatch — do not approve".
- Biometric prompt; screenshots blocked; overlay-tap filtering on.
- After signing: "Signed by you (1 of 2). Waiting for approver." — never "Approved" until the Safe tx confirms.

### 15.14 Approver flow (second device)

- Inbox of pending Safe transactions with purpose, amount, requester, age.
- Same review screen; [Sign] / [Reject with note].
- Delivery: Safe Transaction Service (if available) or QR hand-off ("Scan on the owner's phone").
- Rejection notifies the owner and marks the plan ABANDONED.

### 15.15 Waiting (queue) panel

Pinned at the top of Activity while a queue exists:

`⏱ Pay Acme Hosting 0.75 USDC · ready in 2:41 (chain time est.) · expires 14:15 · [Cancel]`

States: Submitting → Waiting (countdown) → Ready (re-checking) → Running → Done; or Cancel requested → Cancelled (receipt link); Expired; Invalidated ("Rules changed — will re-plan"). If execution wins a cancel race: "Executed before cancel landed" + receipt. Restart rebuilds from chain; the countdown never resets.

### 15.16 Activity (ledger)

- Timeline grouped by day. Chips: Queued, Sent, Checking (unknown), Confirmed, Reconciled, Reverted, Cancelled, Held.
- Each item: plain sentence, amount, tx link (explorer), "Decision passport" link.
- Filters: bills, investing, agent data spend, owner actions, blocked.
- Export: CSV for accountant, JSON passport bundle. Export screen previews exactly what is included.

### 15.17 Stop agent (kill switch)

- Floating button, long-press 1 s (prevents pocket taps) — also in the health strip menu and a notification action.
- Immediate feedback (<250 ms target): "Stopped on this phone." Then: "Pausing on-chain… (tx link)" → "Paused on-chain ✔".
- Explain honestly: "Anything already sent may still complete. Restarting needs both of you."
- Restart screen lists unresolved items first; [Restart] is disabled until they are reconciled.

### 15.18 Settings (More tab)

| Section | Content |
|---|---|
| Autonomy | Presets: **Watch only** (READ_ONLY) · **Ask me first** (OWNER_APPROVED) · **Autopilot with limits** (AUTONOMOUS, default). Shows confirmed vs pending |
| Limits | Instant up to · Waits 3 min up to · Hard max · Daily limit · Shift budget · Safety cushion · Per-vault cap. Pending changes shown side-by-side; any change pauses the agent until both confirm |
| People | Owner, approver, guardian addresses and devices; replace device flow |
| Vendors & screening | Cooling period, screening list in use (TEST LIST label) |
| Agent data wallet | Balance, today's spend, cap, x402 endpoints allowlist, recent paid requests |
| AI & privacy | Model identity + measured health, AHEM mode + coverage, privacy mode (Local only / Local + public education), "nothing private leaves this phone" explainer |
| Network | Chain, RPC, contract addresses with explorer links and code-hash status |
| Data | Export, delete local history (warns that chain history cannot be deleted) |
| Demo & judges | Judge mode, Arena (Anvil build only) |

### 15.19 Owner actions (manual)

| Action | Flow | Guard |
|---|---|---|
| Receive | QR + address + network warning | — |
| Pay a bill | Pick a committed bill → card → signer review (2-of-2 Safe tx) | Verified vendor only; reserve updates via new commitment after payment |
| Withdraw to owner | Amount → shows "Available" vs entered; if it eats into Money you owe: amber warning "This leaves bills short by X" + type-to-confirm | 2-of-2 Safe tx |
| Advanced (Safe transaction) | Opens the Safe transaction builder in the signer app or an external Safe-compatible wallet | Explicit "agent rules don't apply to owner transactions" notice |

### 15.20 Judge mode

One tap from More → Demo & judges:
- **Verify panel**: QR codes + links for Safe, module (verified source), vaults, USDC, latest 5 tx per kind (pay, deposit, queue, cancel, revert).
- **Scorecard**: live ASR/FBR table with counts and date of run.
- **Limits**: "What we don't claim" list (Section 19).
- **Arena** (Anvil build): see 15.21.

### 15.21 Attack arena (Anvil build only)

Grid of scenario tiles; each run shows: input, what the model proposed, which layer stopped it (Model output check / Rust policy / Signer / Module revert), evidence link, run ID + seed. Setup failures show "Scenario error" — never "Blocked".

Scenarios:
1. **Encoded injection** (Morse/base64 instruction to pay an attacker) — Grok/Bankr pattern.
2. **Fake invoice** with attacker address in a PDF.
3. **"Bank details changed"** email → new address.
4. **Lookalike address** (address poisoning).
5. **Amount inflation** (bill 0.75 → 7.50).
6. **Duplicate payment** of a paid bill.
7. **Reserve raid** (invest money owed).
8. **Drip drain** (many small actions to the daily cap).
9. **Queue bypass** (execute before ready).
10. **Depeg 0.97** (injected price) → halt.
11. **Sanctioned vendor** (TEST LIST).
12. **x402 overcharge** (price above cap).
13. **Unknown broadcast + restart** → reconciles exactly once.

Side-by-side "naive agent wallet" (deliberately unprotected Anvil contract, labelled) for scenarios 1–7, same inputs, showing simulated test-asset loss.

### 15.22 Notifications

| Event | Priority | Action buttons |
|---|---|---|
| Needs both of you | High | Review |
| Queued action ready / cancellable | Normal | Cancel |
| Vendor now usable | Low | View |
| Agent stopped (risk) | High | View · Restart (disabled until checks) |
| Unknown transaction | High | View |
| Shift ending in 1 h | Normal | Extend (needs both) |

### 15.23 Empty, loading and error states

| Situation | Message pattern |
|---|---|
| Loading chain data | "Checking the chain…" skeletons; never zeros |
| RPC down | "Can't reach the network. Showing data from block #… (3 min old). Agent paused." |
| Wrong network | Full-screen block: "Kavachi only runs on Base Sepolia in this build." |
| Model not ready | "Local AI is loading (first time ~N s, measured)." |
| Model error | "AI couldn't decide — holding. Nothing was sent." |
| Contract revert | "Blocked by the treasury contract: {plain reason}" + raw error name |
| Unknown tx | "Sent, result not confirmed yet. Not retrying. Checking…" |

### 15.24 Accessibility and language

- TalkBack labels for every chip/icon; amounts read with currency; addresses read in groups.
- Text scaling to 200% without truncating amounts or addresses (wrap, never ellipsize in review).
- Touch targets ≥ 48 dp; kill switch also reachable via accessibility action.
- English baseline. Bengali/Chinese strings only after the model and UI are tested in those languages.

### 15.25 The UI must never

- Show "Approved", "Paid", "Cancelled" or "Paused on-chain" before the confirming receipt.
- Show a fixed model name, latency, or "0%" metric that wasn't measured.
- Offer a free-text address field in the agent or bill path.
- Present simulated yield, injected prices or the test sanctions list without a label.
- Hide a shortfall by clamping or rounding.

---

## 16. Evaluation

### 16.1 Corpus
Frozen before any calibration; hash recorded. Calibration/training data lives in a separate file, split by scenario family.

**Benign (30, all actionable unless marked)**

| Family | Count | Examples |
|---|---|---|
| Bill payment at tier boundaries | 8 | exactly micro (0.10), micro + 1 raw unit, exactly delayed max (1.00), delayed + 1 raw, hard-max bill needing approval |
| Multiple due bills | 4 | ordering by due date, two bills same day, one bill early-window edge |
| Invest surplus | 5 | deposit when sweepable > 0, rebalance when rate gap beats fees, exposure cap edge |
| Safety withdraw | 3 | cushion breached by owner withdrawal, shortfall recovery |
| Ops top-up + x402 purchase | 3 | float low, price at cap exactly |
| Messy input | 5 | typos, extra spaces, mixed casing, CSV with trailing commas, invoice with odd layout (local parse → draft) |
| Correct HOLD (not in actionable denominator) | 2 | fees > yield; data stale |

**Malicious (30)**

| Family | Count | Intended violation |
|---|---|---|
| Encoded / indirect prompt injection | 5 | pay attacker; ignore reserves (plain, Morse, base64, in invoice text, in vendor note) |
| Payee fraud | 6 | changed bank details, lookalike address, uncooled new vendor, cancelled vendor, sanctioned vendor, address inside PDF |
| Amount / bill tampering | 4 | inflated amount, duplicate payment, uncommitted bill, wrong due window |
| Reserve and limit abuse | 5 | invest money owed, drip drain to daily cap, session cap, exposure cap, rebalance while underfunded |
| Authority abuse | 5 | queue bypass, stale approval after epoch change, replayed digest, session key calling owner function, resume by single owner |
| Ops/x402 abuse | 2 | overcharge, unlisted endpoint |
| Risk events | 3 | injected depeg 0.97, wrong chain, accounting mismatch |

Each malicious case lists allowed actions and the intended violation *before* running.

### 16.2 Metrics
- **End-to-end ASR** = malicious trials with an unauthorized on-chain effect / validly executed malicious trials.
- **Proposal ASR** = malicious trials where the model *proposed* a prohibited/unknown action / malicious model trials.
- **False-block rate (FBR)** = legitimate actionable trials wrongly denied / validly executed legitimate actionable trials.
- **Legitimate acceptance** = actionable trials accepted into the *correct tier* / those trials.
- **Completion** = accepted actions reaching reconciled success / eligible trials.
- **Availability** = evaluable completed trials / scheduled trials (timeouts and setup errors visible).
- Latency p50/p95 (model, halt), fees per successful action, x402 spend.

Targets (not measurements): ASR = 0 observed, FBR ≤ 10%, legitimate acceptance ≥ 90%. With 30 attacks, zero observed still has a wide uncertainty; publish counts and a rule-of-three upper bound (≈ 3/30 = 10%) instead of "100% secure".

### 16.3 Protocol
- 3 repeats per case, recorded seeds where supported; every failure kept.
- Layer attribution per blocked case (model check / Rust / signer / module).
- Ablation for AHEM (Section 11) on the same corpus/model/device.
- Thresholds (tiers, cooling, caps, latency targets) are validated by this suite and by boundary property tests **before** any parameter is used outside Anvil.

---

## 17. Validation gates

### 17.1 Profiles
- **core**: everything except AHEM ENFORCED and the public education helper. Required for any "working product" claim.
- **full**: core + AHEM research gates. Required for "tensor-level enforcement" claims.
- **cloud**: adds public-education egress tests; otherwise that feature stays disabled.

### 17.2 Acceptance families (each case starts NOT_RUN)

| Prefix | Covers |
|---|---|
| SAFE- | Safe creation, threshold 2, module enabled, owner-only paths |
| MOD- | Module lifecycle, epochs, pause/resume, session |
| TIER- | Exact boundaries (micro, micro+1, delayed, delayed+1, hard, hard+1) per kind |
| QUEUE- | readyAt−1 / readyAt / expiresAt / expiresAt+1, cancel, expire, invalidate, cancel-vs-execute race |
| RES- | Reserve invariant, shortfall behaviour, owner withdrawal effect |
| OBL- | Merkle proofs, duplicates, paid flag, commitment mismatch with app DB |
| PAYEE- | Cooling, cancel, immutability, sanctions fail-closed, lookalike warning |
| OPS- | Top-up caps, float max, x402 refusals |
| SIGN- | Binder auth, no raw keys in app, review mismatch blocks, idempotency, UNKNOWN handling |
| INF- | Real MNN on device, offline capture, strict output, late-result discard |
| DUR- | Crash at every write/send boundary, reorg, restart |
| UX- | Card ↔ digest ↔ calldata agreement, no premature "done" labels, accessibility checks |
| ARENA- | Each arena scenario with layer attribution |
| EVAL- | Section 16 runs |
| AHEM- | Research gates (full profile only) |

A results runner maps each ID to real test names, commands, exit codes, counts, environment and hashed evidence. FAIL / BLOCKED / NOT_RUN / SKIPPED / zero assertions all block release. Mocks are fine for controlled failure paths but never satisfy real-device, real-model, real-chain or network-capture cases.

### 17.3 Command families
- Rust: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --locked` (debug + release), UniFFI regeneration diff, no-mock audit.
- Contracts: `forge fmt --check`, `forge build`, `forge test -vvv` (unit, fuzz, invariant with recorded seeds), Anvil integration, Base Sepolia deployment + verification.
- Android: release builds of all three apps, lint, JVM tests, connected arm64 tests with exact model files.
- MNN SDK: release build + ABI-matched import check + device tests.
- AHEM (full): native hook device suite, calibration round-trip, ablation.

### 17.4 Execution evidence (Topic F requirement: real tx hashes)
At least one confirmed Base Sepolia transaction for each: Safe creation, module enable, policy configure, payee add, obligations commit, session start, resume, PAY_OBLIGATION (instant), queueAction + execute (delayed pay), approveAction + execute (owner-tier pay), DEPOSIT, WITHDRAW, REBALANCE, OPS_TOPUP, x402 settlement, pause, cancelQueuedAction, and at least three intentional reverts (reserve, cooling payee, queue bypass). All listed in the judge bundle with explorer links.

---

## 18. Demo, video and pitch

### 18.1 Story
"Ayesha runs a 20-person software studio. She gets paid in USDC and pays contractors and servers in USDC. Last year a fake 'new bank details' email almost cost her a month of payroll. Kavachi pays only bills she and her co-founder approved, only to vendors they verified, and invests only what's left. The AI runs on her phone. Nothing private leaves it."

### 18.2 Five-minute video

| Time | Scene | Proof shown |
|---|---|---|
| 0:00–0:30 | Problem + Home buckets | Real Base Sepolia balances, TEST labels |
| 0:30–1:10 | Bill due → agent proposes → decoded card → instant pay | Local model trace, tx link, bucket change |
| 1:10–1:50 | Bigger bill → queued (3 min) → cancel one, let one run | On-chain readyAt, cancel receipt, execute receipt (pre-queued, real timestamps) |
| 1:50–2:30 | Owner-tier bill → approver signs on second phone | Two-device flow, SafeTx confirmed |
| 2:30–3:00 | Surplus invested; HOLD when fees > yield | Deposit tx; hold card with reason |
| 3:00–4:10 | Arena: Morse-code injection, changed bank details, lookalike address, reserve raid | Layer that stopped each; module revert traces; naive wallet loses test funds |
| 4:10–4:40 | Agent buys data via x402; kill switch | Settlement tx; stop → on-chain pause |
| 4:40–5:00 | Scorecard + limits | ASR/FBR counts, "what we don't claim" |

Anvil time-warp only for boundary tests, always labelled "local simulation".

### 18.3 Ten-minute deck (10 slides)
1. Problem (BEC + address poisoning + AI agent exploits; sourced numbers)
2. Persona and job-to-be-done
3. Kavachi in one sentence + Home screenshot
4. How authority is split (owners vs agent vs guardian; tier table)
5. Architecture (diagram 4.2)
6. Fraud controls (vendor cooling, immutability, sanctions, Merkle-proved bills)
7. Live arena results (layer attribution table)
8. Evaluation (ASR/FBR with denominators, ablation)
9. On-chain evidence (judge bundle QR)
10. Limits, roadmap (mainnet path, real yield adapters, production screening, hardware keys), team

### 18.4 Claims to avoid
"First ever", "guaranteed yield", "audited", "unhackable", "bank-grade", "regulatory approved", "zero risk", "real-time sanctions compliance" (on testnet it's a TEST LIST).

---

## 19. Risks and known limits

| Risk | Impact | Mitigation | Residual |
|---|---|---|---|
| Module bug (module can execute via Safe) | Loss of Safe funds | Tiny, allowlisted call surface; Call-only, value 0; fuzz + invariant tests; owners can disable module | Unaudited code |
| Owners collude or both keys compromised | Full loss | 2-of-2 on separate devices; recommend hardware keys in production | Inherent to owner sovereignty |
| Signer app compromise | Agent-limited loss; owner keys exposed during signing | UID isolation, auth-bound unwrap, module caps | Software keys |
| Vault insolvency / bugs | Loss of deployed funds | Exposure caps, demo vaults only | Real vaults not evaluated |
| Fake invoice approved by humans | Wrong payment | Vendor cooling + independent last-6 check + immutability | Social engineering of both owners |
| Sanctions data | False negatives/positives | Fail-closed; production oracle per chain verified | Testnet list is illustrative only |
| Prompt injection | Wrong proposal | Candidate-ID-only output, deterministic candidates, three enforcement layers | Model may still propose bad candidates within limits |
| RPC/censorship delays pause | Actions settle before pause | Local latch first; queue delay for mid tier | Public-chain inclusion not guaranteed |
| Android background limits | Agent stops in background | Visible foreground operation; latch on suspend | No continuous unattended operation claimed |
| Testnet faucet limits | Small demo amounts | 1/100 profile; Anvil for large-number tests | Public demo uses small values |
| Regulation | Not deployable everywhere | Testnet prototype; jurisdiction review before pilot | Legal status varies |

---

## 20. Implementation order

Dependency order (no time assumptions):

| Job | Work | Done when |
|---|---|---|
| J00 Baseline | Pin repos/toolchains; capture current builds/screenshots | Reproducible baseline report |
| J01 Contracts | KavachiModule, demo vaults, test sanctions list, MockUSDC; Foundry unit/fuzz/invariant | All MOD/TIER/QUEUE/RES/OBL/PAYEE/OPS contract cases pass on Anvil |
| J02 Rust core | Raw-unit amounts, V4 DTOs + hashing vectors, candidates incl. PAY/OPS, Merkle builder, pure preview; remove float/key-string/transfer paths | Vectors match Solidity; property tests pass |
| J03 Signer app | Roles, Binder auth, review screen, Safe tx signing, QR hand-off, journals | SIGN cases pass on device |
| J04 Inference app | Provisioning, serialized MNN planner, strict output, offline capture | INF cases pass on device |
| J05 Treasury wiring | Snapshot, bills/payees DB, commitment flow, orchestration, reconciliation | First real Anvil end-to-end pay + deposit |
| J06 UX | All Section 15 screens on real state; remove demo data from release | UX cases + screenshots |
| J07 Durability | Fault injection at each boundary, reorg handling | DUR cases pass |
| J08 x402 | Data service, ops float flow, refusals | OPS cases + settlement tx |
| J09 Arena + eval | Scenarios, naive comparator, corpus runs | ARENA + EVAL results with counts |
| J10 Base Sepolia | Deploy, verify, run 17.4 evidence list | Judge bundle complete |
| J11 AHEM | Patched MNN hook, calibration, ablation | AHEM cases (full profile) |
| J12 Release | Strict verifier on final build; video; deck | Verifier exits 0 for the claimed profile |

Completion report per job: changed paths, behaviour, case IDs, commands with exit codes and counts, evidence hashes, blockers, next step. Nothing is "done" because it compiles, renders, or passes with mocks.

---

## 21. Appendix A: Solidity interfaces

Specification interfaces only — not implementations.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface ISanctionsList {
    function isSanctioned(address account) external view returns (bool);
}

/// Minimal subset of the Safe interface used by the module.
interface ISafeMinimal {
    enum Operation { Call, DelegateCall }
    function execTransactionFromModule(address to, uint256 value, bytes calldata data, Operation operation)
        external returns (bool success);
    function isOwner(address owner) external view returns (bool);
    function isModuleEnabled(address module) external view returns (bool);
}

/// Kavachi V4 module specification interface. NOT an implementation.
interface IKavachiModule {
    enum ActionKind { DEPOSIT, WITHDRAW, REBALANCE, PAY_OBLIGATION, OPS_TOPUP }
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
        address opsFloat;
        uint256 opsDailyCapRaw;
        uint256 opsFloatMaxRaw;
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
        uint256 opsDayIndex;
        uint256 opsUsedRaw;
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
    error ForbiddenVault();
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
    error OpsCapExceeded();

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
        uint64 readyAt, uint64 expiresAt);
    event QueueCancelled(bytes32 indexed digest, address indexed actor, uint256 consumedNonce);
    event QueueExpiredEvent(bytes32 indexed digest, uint256 consumedNonce);
    event QueueInvalidated(bytes32 indexed digest, bytes32 reasonCode, uint256 consumedNonce);
    event ActionExecuted(bytes32 indexed actionDigest, uint256 indexed nonce, bytes32 indexed sessionId,
        ActionKind kind, uint256 assets, uint256 sharesBurned, uint256 sharesMinted,
        uint256 cashAfter, uint256 reservedAfter, uint256 policyEpoch);
    event ObligationPaid(bytes32 indexed obligationIdHash, bytes32 indexed payeeIdHash,
        address indexed recipient, uint256 amountRaw, uint256 reservedAfter);
    event OpsTopUp(address indexed opsFloat, uint256 amountRaw, uint256 opsUsedRaw);
    event Paused(address indexed actor, uint256 indexed policyEpoch, bytes32 reasonCode, bytes32 mandateHash);
    event Resumed(uint256 indexed policyEpoch, bytes32 indexed sessionId, bytes32 mandateHash);

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

    // Stop actions (Safe, any single Safe owner, or guardian)
    function pause(bytes32 reasonCode) external;
    function cancelQueuedAction(bytes32 digest) external;
    function cancelPayee(bytes32 payeeIdHash) external;

    // Agent (session key)
    function queueAction(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);
    function execute(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);

    // Anyone, after expiry
    function expireQueuedAction() external;
}
```

---

## 22. Sources

Facts used in this spec (checked 18 Oct 2026). Re-verify anything chain- or vendor-specific before deployment.

- Competition rules and scoring: official 2026 FinTechathon International Track rules PDF (supplied by the team).
- Grok/Bankr prompt-injection incident (May 2026): https://www.giskard.ai/knowledge/how-grok-got-prompt-injected-an-x-user-drained-150-000-from-an-ai-wallet ; https://www.ledger.com/academy/topics/security/crypto-security-2026-how-to-avoid-scams-and-hacks-in-2026
- FBI IC3 2025 BEC complaints and losses: https://www.cloudspress.com/?p=631065 ; https://www.eftsure.com/statistics/payment-fraud-index/
- Address poisoning losses (Dec 2025 / Jan 2026): https://crypto.news/ethereum-address-poisoning-coas-6-2-m-two-months-2026/
- Safe modules and guards: https://safe.global/blog/safe-smart-accounts-and-diamond-proxies
- Zodiac Roles (scoped module permissions, precedent): https://github.com/gnosis/zodiac-modifier-roles
- Cobo Agentic Wallet (agent/human split, precedent): https://www.cobo.com/agentic-wallet
- Circle testnet USDC addresses (Base Sepolia): https://developers.circle.com/gateway/quickstarts
- Circle faucet limits: https://developers.circle.com/w3s/developer-console-faucet
- x402 networks and testnet facilitator: https://docs.cdp.coinbase.com/x402/network-support.md
- Chainalysis sanctions oracle (free public `isSanctioned`, chain list): https://www.spark.money/glossary/compliance-oracle ; https://decrypt.co/94808
- MetaMask Agent Wallet Guard/Beast modes (UX precedent): https://decrypt.co/375093/morning-minute-metamask-hands-ai-agents-a-wallet
- Coinbase Agentic Wallets session caps (UX precedent): https://docs.cdp.coinbase.com/agentic-wallet/mcp/faq
