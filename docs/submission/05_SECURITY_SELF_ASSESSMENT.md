# Fusion Wallet — Security Self-Assessment & Risk Register

**Evaluation Track:** FinTechathon 2026 International Track — Topic F: Agent Wallet  
**Target Reviewers:** WeBank Industry Experts and Tsinghua PBCSF Scholars  
**Primary Standards & Decision References:** `system architecture/fusion_wallet_architecture.md` (§6, §7.2, §7.7, §7.13, §7.15, §21), `src/contracts/STATUS.md`, `spec/DECISIONS.md`, `evidence/results/latest.json`.

---

## 1. Executive Summary & Security Philosophy

Autonomous financial agents that rely solely on system prompt instructions or post-hoc text filters are fundamentally vulnerable to jailbreaking, prompt injection, and operational drift. Fusion Wallet adheres to a **defense-in-depth, fail-closed architecture** where the on-device Large Language Model (LLM) is strictly bounded:

1. **Inference Bounded by Construction:** The AI inference engine can only select among pre-computed, deterministic candidate actions (`A`–`G`, or `H` for HOLD) through an in-engine decode-time token grammar gate (**AHEM-G**). The model cannot write an address, calldata, or an amount.
2. **On-Chain Enforcement (Kavachi Guard Layer):** Policy, velocity caps, reserve math, and maker-checker authorizations are enforced natively on-chain via `KavachiModule` attached to an official Gnosis Safe v1.4.1 multi-signature account.
3. **Four-Eyes Principle:** Any policy loosening, bill commitment, or large transaction requires dual-signature authorization from both Safe owners (Mei & Wei).
4. **Asymmetric Stopping Power:** Any single human owner or automated guardian key can immediately pause the system, cancel pending queues, or unwind investments back into the Safe.

---

## 2. Authority Model, Permission Tiers & Provenance

### 2.1 Provenance Hierarchy: SCRIPT vs AGENT vs APP

To ensure absolute evaluation transparency, Fusion Wallet enforces strict provenance separation across all operational and empirical records:

| Provenance Tag | Execution Origin | Implementation Mechanism | Authority Level | Verification Script |
|---|---|---|---|---|
| **`SCRIPT`** | Operator / Test Scripts | Deterministic deployment and verification scripts (`src/contracts/script/EvidenceActions.s.sol`). **Never designated as autonomous.** | Operator authority (simulated or explicit key). | `python3 scripts/verify_evidence.py evidence/base-sepolia.json` |
| **`AGENT`** | Autonomous Agent Runner | Autonomous loop (`agent-runner`) driving local LFM2.5 model under AHEM-G token grammar gate (`planner.kind = mnn+ahem-g`). Each `decisionHash` is anchored on-chain. | Bounded session key within active policy. | `python3 scripts/agent/check_golden.py` |
| **`APP`** | Physical Android Phone | In-app actions triggered from the mobile UI and broadcast by `:signer` process using an Android Keystore-wrapped key on real hardware (Tecno CL6). | Unilateral guardian pause authority. | Keyless receipt & event verification (`evidence/app/guardian_pause.json`) |
| **`OWNER`** | Safe Multi-Signature (2-of-2) | Threshold-two dual-signature transactions (Mei & Wei) configuring policy, committing bill roots, or resuming after pause. | Full sovereign multi-signature authority. | `python3 scripts/verify_evidence.py evidence/base-sepolia.json` |

---

### 2.2 Role Definitions & Separation of Duties (§6.1)

| Role | Key Location & Storage | Authorized Capabilities | Prohibited Capabilities |
|---|---|---|---|
| **Owner 1 (Mei)** | Android `:signer` process, biometric Android Keystore unwrap. | Sign Safe multi-sig transactions; trigger unilateral emergency pause, queue cancellation, or emergency unwind. | Cannot act alone on threshold-2 operations (policy update, session creation, large payments). |
| **Owner 2 / Approver (Wei)** | Safe{Wallet} / laptop / secondary device. | Provide second signature for threshold-2 operations; trigger unilateral emergency pause or unwind. | Cannot act alone on threshold-2 operations. |
| **Safe (2-of-2 Threshold)** | On-chain multi-signature account (`0x029869910714D9e09b854a2688311AFd42fBb44E`). | Configure policy, commit bill Merkle roots, register new payees, start/resume/revoke sessions, approve owner-tier actions. | Bound by contract bytecode invariants. |
| **Agent Session Key** | Android `:signer` process, memory-only unwrap during live session (`0xB52Cb48eC1cc5Ad1171C83F162228e31752F87fA`). | Call `execute` and `queueAction` strictly within active policy limits, session budget, and time window. | Cannot alter policy, cannot approve its own actions, cannot unpause, cannot pay uncommitted or cooling payees. |
| **Guardian** | Isolated key in `:signer` process or monitoring service (`0x46B8E40EB9767BD46E03C5eCF398C10E1372251E`). | Unilateral `pause`, `cancelQueuedAction`, `cancelPayee`, and `emergencyUnwind`. | Cannot resume, cannot configure policy, cannot move funds anywhere except into the Safe. |
| **Auditor** | No keys (public read-only interface). | Inspect Judge Mode, export audit trails, verify on-chain decision hashes. | Cannot sign or alter any state. |

**The Stopping Principle:** A single human or guardian can only **stop operations or pull funds back into the Safe**. Starting, loosening parameters, or paying external accounts requires full 2-of-2 Safe authorization or policy-bounded agent execution.

---

### 2.3 Amount Tiers & Transaction Velocity Controls (§6.2 & §6.3)

Every transaction candidate is classified into an on-chain velocity tier based on gross transfer assets (USDC, 6 decimals):

| Tier Range (FULL Profile) | Autonomous Policy Route | Owner-Approved Policy Route | Contract Enforcement |
|---|---|---|---|
| **Micro Tier** ($\le 10.00$ USDC) | **Instant Execution:** Session key executes immediately if within daily and session caps. | Safe approval required, then instant execution. | Enforced in `_validateTiersAndRouting`. |
| **Delayed Tier** ($> 10.00$ and $\le 100.00$ USDC) | **Mandatory 180s Queue:** Session key must submit to `queueAction`, wait 180 seconds, then call `execute`. | Safe approval + mandatory 180s queue. | Enforced via `actionQueue` mapping and `QueueRequired` revert. |
| **Owner Tier** ($> 100.00$ and $\le 2,500.00$ USDC) | **Safe Approval (2-of-2):** Must be explicitly approved via `approveAction` before execution. | Safe approval (2-of-2) required. | Enforced via `actionApprovals` mapping and `ApprovalRequired` revert. |
| **Hard Limit** ($> 2,500.00$ USDC) | **Reverts Unconditionally:** Exceeds maximum single-action authority. | Reverts unconditionally. | Reverts with `HardLimitExceeded`. |

#### Cumulative Velocity & Operational Caps (§6.3)
* **Daily Gross Agent Outflow:** Capped at **4,000.00 USDC** per UTC day (measured by block timestamp).
* **Session Outflow Budget:** Capped at **8,000.00 USDC** per session (maximum session duration: 24 hours).
* **Vault Exposure Ceilings:** Vault A (Aave waUSDC): 6,000.00 USDC; Vault B (`KavachiDemoVault`): 6,000.00 USDC.
* **Maximum Share Slippage:** 50 basis points (0.50%).
* **Payee Cooling Period:** 3,600 seconds (1 hour) on Base Sepolia demo; 86,400 seconds (24 hours) production default.
* **Early Payment Window:** Maximum 3 days (259,200 seconds) prior to invoice `dueAt`.

---

### 2.4 Comprehensive Permission Matrix (§6.5)

| Contract Operation / Function | Agent Session Key | Single Owner | Guardian Key | Safe (2-of-2) | Public / Anyone |
|---|---|---|---|---|---|
| `execute` (Instant / Ready Queued) | **YES** (Policy-bound) | NO | NO | NO | NO |
| `queueAction` (Delayed Tier) | **YES** (Policy-bound) | NO | NO | NO | NO |
| `configurePolicy` | NO | NO | NO | **YES** | NO |
| `commitObligations` | NO | NO | NO | **YES** | NO |
| `addPayee` | NO | NO | NO | **YES** | NO |
| `startSession` | NO | NO | NO | **YES** | NO |
| `resume` (Unpause) | NO | NO | NO | **YES** | NO |
| `revokeSession` | NO | NO | NO | **YES** | NO |
| `approveAction` / `revokeActionApproval` | NO | NO | NO | **YES** | NO |
| `ownerPayObligation` | NO | NO | NO | **YES** | NO |
| `pause` | NO | **YES** | **YES** | **YES** | NO |
| `cancelQueuedAction` | NO | **YES** | **YES** | **YES** | NO |
| `cancelPayee` | NO | **YES** | **YES** | **YES** | NO |
| `emergencyUnwind` (Funds to Safe) | NO | **YES** | **YES** | **YES** | NO |
| `expireQueuedAction` (Post-Expiry) | **YES** | **YES** | **YES** | **YES** | **YES** |
| Normal Safe Transfer / Swap (`execTransaction`) | NO | NO | NO | **YES** | NO |
| `disableModule` | NO | NO | NO | **YES** | NO |

---

## 3. Smart Contract Verification: Controls → Tests → Evidence Mapping

The implementation of `KavachiModule` was verified using the Foundry test framework. As documented in `src/contracts/STATUS.md`:
* **Total Tests Reported:** **157 passed**, 0 failed, 0 skipped (`test-report.txt`).
* **Reproducible Seed:** `0x20261019`
* **Foundry Profile:** Solc `0.8.24`, Cancun EVM, `via_ir = true`, 200 optimizer runs.
* **Bytecode Footprint:** `KavachiModule` runtime size is **24,301 bytes** (275 bytes below the 24,576 byte EIP-170 limit). `KavachiPolicyLib` runtime is **5,854 bytes**.
* **PAY Gas Consumption:** **292,385 gas** for `KavachiModule.execute(PAY_OBLIGATION)` under an official Safe v1.4.1 proxy fixture.

---

### 3.1 Normative Guard Checks (§7.7): Controls to Tests to Evidence

The following 27 normative check families are verified across unit tests and confirmed with on-chain execution receipts:

| Step / Guard | Invariant / Revert Condition | Covering Test Function(s) in `test/` | On-Chain Evidence Row / Source |
|---|---|---|---|
| **Step 1** | Deployment chain validation (`WrongChain`) | `SharedChecksTest.test_commonWrongChainAllKinds` | `STATUS.md` line 26 |
| **Step 1** | Module enabled on Safe (`ModuleNotEnabled`) | `SharedChecksTest.test_commonDisabledAllKinds` | Row E2 (`0xd8d145e6...`) in `evidence/base-sepolia.json` |
| **Step 1** | Module pause status (`ModulePaused`) | `SharedChecksTest.test_commonPausedAllKinds`, `QueueTest.test_pausedQueueForbidden` | Row E14-APP (`0xb36b6d22...`) in `evidence/app/guardian_pause.json`, Arena A20 |
| **Step 1** | Caller is session key (`Unauthorized`) | `SharedChecksTest.test_commonUnauthorizedAllKinds`, `test_onlySafeAdministration` | Arena A15 in `evidence/results/anvil-A15.json` |
| **Step 1** | Live session validity (`SessionExpired`) | `SharedChecksTest.test_commonSessionExpiredAllKinds`, `test_ownerSessionKeyGuardianSeparation` | Row E6 (`0xb8927af2...`) in `evidence/base-sepolia.json` |
| **Step 1** | Permission tier check (`ReadOnly`) | `SharedChecksTest.test_commonReadOnlyAllKinds` | `STATUS.md` line 31 |
| **Step 2** | Existing queue excludes other digests (`QueueBusy`) | `QueueTest.test_commonQueueBusyAllKinds`, `QueueTest.test_queueBusyBeforeNonceAndPolicy` | `QueueTest.t.sol` |
| **Step 3** | Current nonce matching (`BadNonce`) | `SharedChecksTest.test_commonBadNonceAllKinds`, `QueueTest.test_queueCancelAndReplay` | Arena A21 in `evidence/results/anvil-A21.json` |
| **Step 3** | Policy epoch matching (`StalePolicy`) | `SharedChecksTest.test_commonStalePolicyAllKinds`, `test_pauseIdempotentAndSafeOnlyResume` | Arena A18 in `evidence/results/anvil-A18.json` |
| **Step 3** | Action deadline elapsed (`ActionExpired`) | `SharedChecksTest.test_commonDeadlineExpiredAllKinds` | `STATUS.md` line 35 |
| **Step 3** | Immediate deadline $\le 120$s (`InvalidDeadline`) | `SharedChecksTest.test_commonDeadlineTooFarAllKinds` | `STATUS.md` line 36 |
| **Step 3** | Approved deadline $\le 900$s | `CoreTest.test_approvedDeadlineBound`, `test_approvalValidAtExpiryAndConsumed` | `STATUS.md` line 37 |
| **Step 3** | Deadline $\le$ session expiry | `CoreTest.test_deadlineAfterSession` | `STATUS.md` line 38 |
| **Step 3** | Queue deadline bounds and expiration | `QueueTest.test_queueDeadlineLowerAndUpper`, `test_queueExpiresAndConsumesNonce` | Arena A22 in `evidence/results/anvil-A22.json` |
| **Step 3** | Non-zero decision hash (`MissingDecisionHash`) | `SharedChecksTest.test_commonMissingDecisionAllKinds` | Row E7-AGENT (`0x8bab4844...`) in `evidence/agent/golden.json` |
| **Step 4** | Action shape validation (`UnsupportedAction`) | `CoreTest.test_badDepositShape`, `test_badWithdrawShape`, `test_badPayShape`, `test_rebalanceUnsupported` | `STATUS.md` line 41 |
| **Step 4** | Non-zero asset amount (`InvalidAmount`) | `SharedChecksTest.test_commonZeroAmountAllKinds` | `STATUS.md` line 42 |
| **Step 4** | Single-action hard limit (`HardLimitExceeded`) | `SharedChecksTest.test_commonHardLimitAllKinds`, `BoundariesTest.test_tierEdgesMicroDelayedAndOwner` | Arena A23 in `evidence/results/anvil-A23.json` |
| **Step 5** | UTC-day gross limit (`DailyLimitExceeded`) | `SharedChecksTest.test_commonDailyLimitAllKinds`, `QueueTest.test_dailyRolloverAndSessionRestartCannotResetDay` | Arena A8 in `evidence/results/anvil-A8.json` |
| **Step 5** | Session gross limit (`SessionLimitExceeded`) | `SharedChecksTest.test_commonSessionLimitAllKinds`, `CoreTest.test_sessionLimit` | `STATUS.md` line 45 |
| **Step 6** | Tier routing boundaries | `BoundariesTest.test_tierEdgesMicroDelayedAndOwner`, `CoreTest.test_ownerApprovedInstant` | Row E9 (`0x97e5cd9c...`) in `evidence/base-sepolia.json` |
| **Step 6** | Missing delayed queue (`QueueRequired`) | `SharedChecksTest.test_commonDelayedRequiredAllKinds`, `QueueTest.test_delayedRequiresQueueEvenWithApproval` | Row E16 (`0xe1d5dba1...`) in `evidence/base-sepolia.json`, Arena A9 |
| **Step 6** | Queue not ready (`QueueNotReady`) | `SharedChecksTest.test_commonDelayedNotReadyAllKinds`, `QueueTest.test_queueReadyExecutionNoChargeUntilExecute` | Arena A24 in `evidence/results/anvil-A24.json` |
| **Step 6** | Missing Safe approval (`ApprovalRequired`) | `SharedChecksTest.test_commonApprovalRequiredAllKinds`, `CoreTest.test_revokeApproval` | Row E9 (`0x4fe8a794...`) in `evidence/base-sepolia.json` |
| **Step 6** | Expired Safe approval (`ApprovalExpired`) | `SharedChecksTest.test_commonApprovalExpiredAllKinds` | `STATUS.md` line 51 |
| **Step 7 (DEPOSIT)** | Solvency $C \ge R$ (`Underfunded`) | `CoreTest.test_depositUnderfunded`, `InvariantTest.invariant_I1_reservePreservedOnInvestment` | `STATUS.md` line 52 |
| **Step 7 (DEPOSIT)** | Reserve preservation $C - \text{assets} \ge R + B$ (`ReserveViolation`) | `CoreTest.test_depositReserve`, `QueueTest.test_queueRerunsReserve`, `CoreTest.testFuzz_depositPreservesReserve` | Row E16 (`0x55d44a2b...`) in `evidence/base-sepolia.json`, Arena A7 |
| **Step 7 (DEPOSIT)** | Allowed destination vault (`ForbiddenVault`) | `CoreTest.test_forbiddenVault`, `CoreTest.test_vaultNotInPolicy` | Arena A19 in `evidence/results/anvil-A19.json` |
| **Step 7 (DEPOSIT)** | Vault exposure ceiling (`ExposureExceeded`) | `CoreTest.test_exposureExceeded` | `STATUS.md` line 55 |
| **Step 7 (DEPOSIT)** | Share bounds and slippage (`ShareBoundViolation`) | `CoreTest.test_depositMinShares`, `CoreTest.test_depositLooseSlippage`, `BoundariesTest.test_depositObservedShareBound` | Row E10 (`0x35ff759c...`) in `evidence/base-sepolia.json` |
| **Step 7 (Vault)** | Pinned runtime code hash (`VaultCodeChanged`) | `CoreTest.test_vaultCodeChanged` | `STATUS.md` line 57 (Contrast with A17 proxy limitation) |
| **Step 7 (WITHDRAW)**| Vault liquidity sufficiency (`InsufficientLiquidity`) | `CoreTest.test_withdrawLiquidity` | `STATUS.md` line 58 |
| **Step 7 (WITHDRAW)**| Share burn slippage (`ShareBoundViolation`) | `CoreTest.test_withdrawShareBound`, `BoundariesTest.test_withdrawObservedShareBound` | Row E11 (`0x06530794...`) in `evidence/base-sepolia.json` |
| **Step 7 (PAY)** | Exact committed leaf proof (`BadObligationProof`) | `CoreTest.test_badProof`, `HashVectorsTest.test_commitments_single_leaf` | Arena A5 in `evidence/results/anvil-A5.json` |
| **Step 7 (PAY)** | Double-payment prevention (`ObligationAlreadyPaid`) | `CoreTest.test_duplicatePayAgentAndOwner`, `CoreTest.test_ownerThenAgentDuplicate` | Row E16 (`0x5af192e0...`) in `evidence/base-sepolia.json`, Arena A6, A14 |
| **Step 7 (PAY)** | Early payment window (`PayWindowNotOpen`) | `CoreTest.test_payWindow`, `BoundariesTest.test_payWindowExactBoundary` | `STATUS.md` line 62 |
| **Step 7 (PAY)** | Payee cooling / cancellation (`PayeeNotActive`) | `CoreTest.test_payeeCooling`, `CoreTest.test_payeeCancelled`, `CoreTest.test_payeeImmutable` | Row E16 (`0x8f9dde94...`), Row E16 (`0xc04b77c2...`) in `evidence/base-sepolia.json`, Arena A3 |
| **Step 7 (PAY)** | Recipient account match (`RecipientMismatch`) | `CoreTest.test_recipientMismatch` | Arena A2, A4 in `evidence/results/anvil-A2.json`, `anvil-A4.json` |
| **Step 7 (PAY)** | Sanctions screening (`PayeeSanctioned`) | `CoreTest.test_sanctionedAtPay`, `CoreTest.test_sanctionedAtAdd`, `QueueTest.test_queuePaySanctionsAtQueueAndExecution` | Arena A11 in `evidence/results/anvil-A11.json` |
| **Step 7 (PAY)** | Sanctions oracle failure closed (`SanctionsCheckFailed`)| `CoreTest.test_sanctionsFailClosed`, `CoreTest.test_sanctionsMalformedClosed` | `STATUS.md` line 66 |
| **Step 8** | Safe call boolean failure (`SafeCallFailed`) | `SafeIntegrationTest.test_safeCallFailedDeposit`, `test_safeCallFailedWithdraw`, `test_safeCallFailedPay` | `STATUS.md` line 67 |
| **Step 8** | Asset / share delta verification (`AccountingMismatch`) | `SafeIntegrationTest.test_accountingMismatchPay`, `test_accountingMismatchDeposit`, `test_feeOnTransferRejected` | `STATUS.md` line 69 |
| **Step 8** | Zero residual allowance (`ResidualAllowance`) | `SafeIntegrationTest.test_residualAllowanceRejected`, `InvariantTest.invariant_I5_noResidualAllowances` | Invariant I5 verification |
| **All Steps** | Reentrancy protection (`nonReentrant`) | `SafeIntegrationTest.test_reentrancyCannotExecuteAgain` | `STATUS.md` line 71 |

---

### 3.2 Formal Stateful Invariant Verification (I1–I7)

Stateful fuzzing was conducted via `TreasuryHandler` executing against live Safe and `KavachiModule` deployments (`src/contracts/STATUS.md` §7.13):

| Invariant ID | Formal Property Description | Covering Test Target | Verified Property in Campaign |
|---|---|---|---|
| **I1** | Reserve preservation: Liquid cash satisfies $C \ge R + B$ after any investment. Underfunded state ($C < R$) strictly blocks deposits. | `InvariantTest.invariant_I1_reservePreservedOnInvestment` | Verified across 2,048 handler operations including owner transfers and cash donations. |
| **I2** | Destination whitelist: Treasury assets leave Safe only to authorized vaults or active, non-sanctioned payees matching committed leaves. | `InvariantTest.invariant_I2_onlyTrustedDestinations` | Verified; zero funds leaked to untracked accounts. |
| **I3** | Velocity limits: Gross agent outflows never exceed daily or session limits. | `InvariantTest.invariant_I3_grossDailyAndSessionLimits` | Verified; ghost variables match module storage. |
| **I4** | Single disbursement: Every obligation ID is paid at most once across both agent and owner execution paths. | `InvariantTest.invariant_I4_paidAtMostOnce` | Verified; duplicate replay attempts unconditionally revert. |
| **I5** | Allowance hygiene: Safe allowance to any vault is reset to exactly 0 at transaction termination. | `InvariantTest.invariant_I5_noResidualAllowances` | Verified; Safe allowances remain strictly zero across successes and reverts. |
| **I6** | Pause integrity: While paused, all agent execution reverts. Only Safe 2-of-2 can resume. | `InvariantTest.invariant_I6_pauseStopsAgentAndOnlySafeResumes` | Verified; guardian key probes fail to resume. |
| **I7** | Nonce monotonicity: Action nonces increase monotonically. Consumed, cancelled, or expired digests cannot be replayed. | `InvariantTest.invariant_I7_nonceConsumptionAndDigestUniqueness` | Verified; unique digest execution asserted. |

---

## 4. Comprehensive Risk Register & Known Residuals

The risk register reflects actual engineering constraints documented in `system architecture/fusion_wallet_architecture.md` §21.1 and the empirical arena evaluation (`evidence/results/latest.json`).

### 4.1 Empirical Attack Arena Residuals & Known Risk Disclosures

In our 24-scenario adversarial arena evaluation, 21 scenarios were successfully stopped by defense layers, 2 escaped due to documented architectural boundaries, and 1 represents a planned feature not yet implemented:

```
Total Scenarios: 24 | Stopped: 21 | Escaped: 2 (A12, A17) | Not Implemented: 1 (A13)
```

#### 1. Risk A12: x402 Overcharge / Arbitrary Prepared Transaction (ESCAPED)
- **Attack Scenario:** Attacker crafts an arbitrary HTTP 402 payment request with an inflated amount (e.g., 99.00 USDC quoted vs 100.00 USDC charged).
- **Stopping Layer Status:** `signer` (Escaped in Arena).
- **Technical Root Cause:** A dedicated x402 authority/budget module is not yet implemented in `fusion-core`. The generic transaction signer accepts arbitrary prepared transactions. In the arena probe (`evidence/results/anvil-A12.json`), the signer generated a signed hash without enforcing an x402-specific budget policy.
- **Residual Risk & Mitigation:** No actual on-chain broadcast occurred in the arena. In production, x402 signing must remain disabled until a dedicated typed signing envelope and per-request budget caps are integrated into the `:signer` process.

#### 2. Risk A17: Proxy Implementation Swap Escapes Code-Hash Pin (ESCAPED)
- **Attack Scenario:** An underlying ERC-4626 yield vault deployed behind an ERC-1967 or Beacon proxy upgrades its implementation contract to malicious bytecode.
- **Stopping Layer Status:** `contract` (Escaped in Arena; documented limit §7.2 & §21.1).
- **Technical Root Cause:** `KavachiModule` validates `extcodehash(vault)` against `policy.vaultRuntimeCodeHash`. Under standard EVM proxy patterns, upgrading the implementation behind the proxy alters logic but leaves the proxy contract's own bytecode hash identical. Thus, `extcodehash` verification passes even though the underlying execution logic has changed. (Runtime code replacement of non-proxy vaults is stopped; see `test_vaultCodeChanged()`).
- **Residual Risk & Mitigation:** Fusion Wallet explicitly discloses that code-hash pinning protects only non-proxy vaults or the proxy shell itself. Production mitigation requires an off-chain implementation monitor service that reads the ERC-1967 implementation slot (`0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc`) and triggers an automated guardian pause upon unexpected implementation swaps.

#### 3. Risk A13: Unknown Broadcast Plus Client Crash / Restart (NOT IMPLEMENTED)
- **Attack Scenario:** The client crashes or loses power immediately after broadcasting an action to the network, before receiving a transaction receipt.
- **Stopping Layer Status:** `durable_journal` (Not Implemented in current core).
- **Technical Root Cause:** The current `fusion-core` implementation lacks a durable write-ahead journal or SQLite state machine to reconcile `UNKNOWN` broadcast states upon client restart.
- **Residual Risk & Mitigation:** In the event of a client restart during an in-flight broadcast, the on-chain contract invariant I7 (monotonic nonces) prevents double spending: any subsequent transaction with the same nonce reverts with `BadNonce`. However, client state recovery currently requires manual querying of on-chain event logs. A durable local transaction journal is slated for future development.

#### 4. High False Block Rate (FBR = 25/30 under AHEM-G)
- **Empirical Measurement:** Across the frozen 60-case benchmark corpus, AHEM-G achieved an **Attack Success Rate (ASR) of 0/30 (0.0%)** (zero malicious proposals admitted). However, it recorded a **False Block Rate (FBR) of 25/30 (83.3%)** across all benign scenarios (and **25/28 (89.3%) actionable FBR**, excluding 2 cases where HOLD is the financially optimal action).
- **Technical Root Cause:** LFM2.5-230M is a compact edge model (230M parameters). Under strict grammar constraints, when faced with complex accounting prompts, the model conservatively defaults to option `H` (`HOLD`) rather than proposing a payment.
- **Operational Impact & Mitigation:** High FBR represents an operational delay, **never a financial loss**. Treasury assets remain untouched in the Safe. Owners receive real-time notifications for overdue obligations and can execute payment directly via `ownerPayObligation`. Track K14 is actively optimizing prompt templates and letter constraints to lower FBR while strictly maintaining 0% ASR.

---

### 4.2 Comprehensive Risk Register Table

| Risk ID / Category | Inherent Threat | Architectural Mitigation | Residual Risk & Governance |
|---|---|---|---|
| **A12: x402 Budget** | Unchecked transaction prepared through x402 payment path. | Signer simulates call; amount bounds checked. | Dedicated x402 budget signer not built; x402 disabled by default. |
| **A17: Proxy Upgrade** | Underlying vault implementation changes behind proxy. | Proxy `extcodehash` pinned on-chain. | Proxy implementation upgrades preserve code hash; requires external monitoring. |
| **A13: Crash Recovery** | Client crashes mid-broadcast; unknown tx state. | Monotonic on-chain nonce prevents replay (`BadNonce`). | No durable local journal; requires chain sync upon reboot. |
| **Model FBR (83.3%)** | Model chooses HOLD on legitimate due bills. | Due date alerts in mobile UI; dual owners can bypass agent via `ownerPayObligation`. | Operational delay; financial safety strictly preserved. |
| **Smart Contract Defect** | Logic flaw in `KavachiModule` leading to asset lock. | Invariants I1–I7, 157 Foundry tests, formal handler campaign. Owners can disable module. | Unaudited testnet prototype. |
| **Dual Owner Collusion** | Both Mei and Wei keys compromised. | 2-of-2 Safe threshold; independent signing devices. | Sovereign multi-sig boundary. |
| **Signer Compromise** | Memory dump of mobile `:signer` process. | Session key bounded by on-chain daily/session limits. | Android OS application sandbox isolation. |
| **Prompt Injection** | Adversarial text tricks LLM into attacking treasury. | **AHEM-G Grammar Gate:** 0/200 illegal outputs, 0/30 ASR. Token-level sampler masking. | Model may default to HOLD. |
| **Invoice Redirection** | Attacker injects fraudulent vendor details. | Mandatory cooling delay (3,600s demo; 24h prod); guardian cancel. | Phishing both Safe owners during initial registration. |
| **Sanctions False Negatives** | Sanctioned entity receives payment. | `isSanctioned` oracle evaluated at add, queue, and execute. Fails closed. | Testnet oracle contains illustrative addresses. |
| **Vault Insolvency** | Underlying yield protocol halts or depegs. | Exposure ceilings (6,000 USDC), share bounds, one-tap `emergencyUnwind`. | Third-party smart contract insolvency risk remains with external protocol. |
| **Delayed Pause** | Network congestion delays guardian pause transaction. | 180s delayed tier provides on-chain cancellation window. | Base Sepolia gas fee market fluctuations. |

---

## 5. Explicit Limitations & Engineering Boundaries

In compliance with competition guidelines, Fusion Wallet makes the following explicit engineering disclosures:

1. **Testnet Only:** Deployed and evaluated strictly on the **Base Sepolia** testnet. The system manages testnet stablecoins and has not been evaluated for mainnet value.
2. **Unaudited Code:** The smart contracts and Rust libraries are prototype implementations developed for FinTechathon 2026 and have not received an independent security audit.
3. **Illustrative Sanctions List:** The on-chain `KavachiTestSanctionsList` contract (`0xd2716e08BCe6eb5B1f01329FF5E1a2dd0468B678`) contains static demonstration addresses. It does not interface with a real-time global compliance screening provider.
4. **Demonstration Payee Cooling Window:** The on-chain payee cooling period is configured to **3,600 seconds (1 hour)** on Base Sepolia to enable evaluation during judging sessions. Production deployments require a minimum 86,400-second (24-hour) cooling window.
5. **Simulated Vault Yields:** Vault B (`KavachiDemoVault`, `0xf052C0ED8404adEaAEA773A5F8Ae4628134aff6D`) utilizes scenario-based internal drip rates to demonstrate deposit and rebalance math. Yields are illustrative and do not reflect guaranteed financial returns.
6. **EIP-170 Bytecode Proximity:** The compiled runtime bytecode of `KavachiModule` is **24,301 bytes**, which is 275 bytes below the EVM runtime size limit of 24,576 bytes. Further feature expansion requires modular decomposition into additional external libraries.
7. **Mobile Hardware Execution Scope:** AHEM-G decode-time token gating is fully validated on host Linux aarch64 hardware (`evidence/planner/host_results.json`). Android phone cross-compilation via `cargo-ndk` is designated as next-phase engineering (`AHEM-G-07` reported as `NOT_RUN`).
8. **What the Module Does NOT Protect (§7.15):**
   * Actions executed by both Safe owners acting in concert (threshold-two sovereignty).
   * External smart contract vulnerabilities within third-party ERC-4626 vaults or the Aave protocol.
   * Gas expenditures incurred from reverted transactions.
   * The underlying real-world validity of an invoice before dual-owner commitment.
