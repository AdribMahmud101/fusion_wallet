# Fusion Wallet — Security Self-Assessment & Risk Register

**Evaluation Track:** FinTechathon 2026 International Track — Topic F: Agent Wallet  
**Target Reviewers:** WeBank Industry Experts and Tsinghua PBCSF Scholars  
**Primary Standards & Decision References:** `system architecture/fusion_wallet_architecture.md` (§6, §7.7, §7.13, §7.15, §21), `src/contracts/STATUS.md`, `spec/DECISIONS.md`.

---

## 1. Executive Summary & Security Philosophy

Autonomous financial agents that rely solely on system prompt instructions or post-hoc text filters are fundamentally vulnerable to jailbreaking, prompt injection, and operational drift. Fusion Wallet adheres to a **defense-in-depth, fail-closed architecture** where the on-device Large Language Model (LLM) is strictly bounded:

1. **Inference Bounded by Construction:** The AI inference engine can only select among pre-computed, deterministic candidate actions (`A`–`G`, or `H` for HOLD) through an in-engine decode-time token grammar gate (**AHEM-G**). The model cannot write an address, calldata, or an amount.
2. **On-Chain Enforcement (Kavachi Guard Layer):** Policy, velocity caps, reserve math, and maker-checker authorizations are enforced natively on-chain via `KavachiModule` attached to a Gnosis Safe v1.4.1 multi-signature account.
3. **Four-Eyes Principle:** Any policy loosening, bill commitment, or large transaction requires dual-signature authorization from both Safe owners (Mei & Wei).
4. **Asymmetric Stopping Power:** Any single human owner or automated guardian key can immediately pause the system, cancel pending queues, or unwind investments back into the Safe.

---

## 2. Authority Model & Permission Tiers

### 2.1 Role Definitions & Separation of Duties (§6.1)

| Role | Key Location & Storage | Authorized Capabilities | Prohibited Capabilities |
|---|---|---|---|
| **Owner 1 (Mei)** | Android `:signer` process, biometric Android Keystore unwrap. | Sign Safe multi-sig transactions; trigger unilateral emergency pause, queue cancellation, or emergency unwind. | Cannot act alone on threshold-2 operations (policy update, session creation, large payments). |
| **Owner 2 / Approver (Wei)** | Safe{Wallet} / laptop / secondary device. | Provide second signature for threshold-2 operations; trigger unilateral emergency pause or unwind. | Cannot act alone on threshold-2 operations. |
| **Safe (2-of-2 Threshold)** | On-chain multi-signature account (`0x029869910714D9e09b854a2688311AFd42fBb44E`). | Configure policy, commit bill Merkle roots, register new payees, start/resume/revoke sessions, approve owner-tier actions. | Bound by contract bytecode invariants. |
| **Agent Session Key** | Android `:signer` process, memory-only unwrap during live session. | Call `execute` and `queueAction` strictly within active policy limits, session budget, and time window. | Cannot alter policy, cannot approve its own actions, cannot unpause, cannot pay uncommitted or cooling payees. |
| **Guardian** | Isolated key in `:signer` process or monitoring service. | Unilateral `pause`, `cancelQueuedAction`, `cancelPayee`, and `emergencyUnwind`. | Cannot resume, cannot configure policy, cannot move funds anywhere except into the Safe. |
| **Auditor** | No keys (public read-only interface). | Inspect Judge Mode, export audit trails, verify on-chain decision hashes. | Cannot sign or alter any state. |

**The Stopping Principle:** A single human or guardian can only **stop operations or pull funds back into the Safe**. Starting, loosening parameters, or paying external accounts requires full 2-of-2 Safe authorization or policy-bounded agent execution.

---

### 2.2 Amount Tiers & Transaction Velocity Controls (§6.2 & §6.3)

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

### 2.3 Comprehensive Permission Matrix (§6.5)

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

### 2.4 Institutional Banking Controls Mapping (§2.8)

| Banking Control Standard | Fusion Wallet Technical Mechanism | Enforcement Point |
|---|---|---|
| **Four-Eyes Principle (Maker-Checker)** | 2-of-2 Safe signature threshold required for all policy changes, session grants, payee additions, and bill commitments. | On-chain Safe threshold verification. |
| **Segregation of Duties** | Agent key executes within policy; guardian key pauses/cancels; owners manage configuration; auditor is strictly read-only. | `KavachiModule` caller assertions (`onlySafe`, `onlySessionKey`, `onlyGuardianOrOwner`). |
| **Confirmation of Payee (Payee Verification)** | Payees registered via immutable tuple (`payeeIdHash`, `account`, `labelHash`). Mandatory cooling period before activation. | `KavachiModule` payee registry + client-side lookalike analysis. |
| **Sanctions Screening (KYT)** | Evaluates `isSanctioned(address)` at payee registration, action queuing, and final execution. Reverting oracle fails closed. | `ISanctionsList` interface calls (`SanctionsCheckFailed` on revert). |
| **Transaction Velocity Limits** | Per-transaction tiers, rolling UTC daily limits, and lifetime session caps. | Smart contract state (`dailyUsageRaw`, `sessionUsageRaw`). |
| **Cooling-Off Period** | New vendor accounts cannot receive disbursements until `activeAt` timestamp matures. | `KavachiModule.addPayee` timelock check. |
| **Model Risk Governance & Explainability** | Deterministic integer candidate generation, AHEM-G token grammar mask, RFC 8785 canonical passport JSON with model hash. | Rust `candidates.rs`, `fusion-planner`, and on-chain `decisionHash`. |
| **Audit Trail & Non-Repudiation** | Every executed action emits `decisionHash` in on-chain events. Independent read-only verification script re-hashes passports. | Event `ActionExecuted(..., decisionHash)` + `scripts/verify_evidence.py`. |
| **Data Minimization & Privacy** | Inference runs locally on-device. Prompt contains no Ethereum addresses, private keys, or raw balances. | Process boundary between `:main` and `:inference`. |
| **Business Continuity & Kill Switch** | Emergency stop button, guardian pause, and atomic multi-vault redemption into the Safe treasury. | `KavachiModule.emergencyUnwind`. |

*Regulatory Note:* This table represents an architectural mapping against common financial controls; it does not constitute a regulatory compliance certification or legal opinion.

---

## 3. Smart Contract Verification: Controls → Tests Mapping

The implementation of `KavachiModule` was verified using the Foundry test framework. As documented in `src/contracts/STATUS.md`:
* **Total Tests Reported:** **157 passed**, 0 failed, 0 skipped.
* **Reproducible Seed:** `0x20261019`
* **Foundry Profile:** Solc `0.8.24`, Cancun EVM, `via-ir = true`, 200 optimizer runs.
* **Bytecode Footprint:** `KavachiModule` runtime size is **24,301 bytes** (275 bytes below the 24,576 byte EIP-170 limit). `KavachiPolicyLib` runtime is **5,854 bytes**.
* **PAY Gas Consumption:** **292,385 gas** for `KavachiModule.execute(PAY_OBLIGATION)` under an official Safe v1.4.1 proxy fixture.

---

### 3.1 Normative Guard Checks (§7.7) Mapping

The following 27 normative check families are verified across dedicated unit test suites in `src/contracts/test/`:

| Step / Guard | Invariant / Revert Condition | Covering Test Function(s) in `test/` |
|---|---|---|
| **Step 1** | Deployment chain validation (`WrongChain`) | `SharedChecksTest.test_commonWrongChainAllKinds` |
| **Step 1** | Module enabled on Safe (`ModuleNotEnabled`) | `SharedChecksTest.test_commonDisabledAllKinds` |
| **Step 1** | Module pause status (`ModulePaused`) | `SharedChecksTest.test_commonPausedAllKinds`, `QueueTest.test_pausedQueueForbidden` |
| **Step 1** | Caller is session key (`Unauthorized`) | `SharedChecksTest.test_commonUnauthorizedAllKinds`, `test_onlySafeAdministration` |
| **Step 1** | Live session validity (`SessionExpired`) | `SharedChecksTest.test_commonSessionExpiredAllKinds`, `test_ownerSessionKeyGuardianSeparation` |
| **Step 1** | Permission tier check (`ReadOnly`) | `SharedChecksTest.test_commonReadOnlyAllKinds` |
| **Step 2** | Existing queue excludes other digests (`QueueBusy`) | `QueueTest.test_commonQueueBusyAllKinds`, `QueueTest.test_queueBusyBeforeNonceAndPolicy` |
| **Step 3** | Current nonce matching (`BadNonce`) | `SharedChecksTest.test_commonBadNonceAllKinds`, `QueueTest.test_queueCancelAndReplay` |
| **Step 3** | Policy epoch matching (`StalePolicy`) | `SharedChecksTest.test_commonStalePolicyAllKinds`, `test_pauseIdempotentAndSafeOnlyResume` |
| **Step 3** | Action deadline elapsed (`ActionExpired`) | `SharedChecksTest.test_commonDeadlineExpiredAllKinds` |
| **Step 3** | Immediate deadline $\le 120$s (`InvalidDeadline`) | `SharedChecksTest.test_commonDeadlineTooFarAllKinds` |
| **Step 3** | Approved deadline $\le 900$s | `CoreTest.test_approvedDeadlineBound`, `test_approvalValidAtExpiryAndConsumed` |
| **Step 3** | Deadline $\le$ session expiry | `CoreTest.test_deadlineAfterSession` |
| **Step 3** | Queue deadline bounds and expiration | `QueueTest.test_queueDeadlineLowerAndUpper`, `test_queueExpiresAndConsumesNonce` |
| **Step 3** | Non-zero decision hash (`MissingDecisionHash`) | `SharedChecksTest.test_commonMissingDecisionAllKinds` |
| **Step 4** | Action shape validation (`UnsupportedAction`) | `CoreTest.test_badDepositShape`, `test_badWithdrawShape`, `test_badPayShape`, `test_rebalanceUnsupported` |
| **Step 4** | Non-zero asset amount (`InvalidAmount`) | `SharedChecksTest.test_commonZeroAmountAllKinds` |
| **Step 4** | Single-action hard limit (`HardLimitExceeded`) | `SharedChecksTest.test_commonHardLimitAllKinds`, `BoundariesTest.test_tierEdgesMicroDelayedAndOwner` |
| **Step 5** | UTC-day gross limit (`DailyLimitExceeded`) | `SharedChecksTest.test_commonDailyLimitAllKinds`, `QueueTest.test_dailyRolloverAndSessionRestartCannotResetDay` |
| **Step 5** | Session gross limit (`SessionLimitExceeded`) | `SharedChecksTest.test_commonSessionLimitAllKinds`, `CoreTest.test_sessionLimit` |
| **Step 6** | Tier routing boundaries | `BoundariesTest.test_tierEdgesMicroDelayedAndOwner`, `CoreTest.test_ownerApprovedInstant` |
| **Step 6** | Missing delayed queue (`QueueRequired`) | `SharedChecksTest.test_commonDelayedRequiredAllKinds`, `QueueTest.test_delayedRequiresQueueEvenWithApproval` |
| **Step 6** | Queue not ready (`QueueNotReady`) | `SharedChecksTest.test_commonDelayedNotReadyAllKinds`, `QueueTest.test_queueReadyExecutionNoChargeUntilExecute` |
| **Step 6** | Missing Safe approval (`ApprovalRequired`) | `SharedChecksTest.test_commonApprovalRequiredAllKinds`, `CoreTest.test_revokeApproval` |
| **Step 6** | Expired Safe approval (`ApprovalExpired`) | `SharedChecksTest.test_commonApprovalExpiredAllKinds` |
| **Step 7 (DEPOSIT)** | Solvency $C \ge R$ (`Underfunded`) | `CoreTest.test_depositUnderfunded`, `InvariantTest.invariant_I1_reservePreservedOnInvestment` |
| **Step 7 (DEPOSIT)** | Reserve preservation $C - \text{assets} \ge R + B$ (`ReserveViolation`) | `CoreTest.test_depositReserve`, `QueueTest.test_queueRerunsReserve`, `CoreTest.testFuzz_depositPreservesReserve` |
| **Step 7 (DEPOSIT)** | Allowed destination vault (`ForbiddenVault`) | `CoreTest.test_forbiddenVault`, `CoreTest.test_vaultNotInPolicy` |
| **Step 7 (DEPOSIT)** | Vault exposure ceiling (`ExposureExceeded`) | `CoreTest.test_exposureExceeded` |
| **Step 7 (DEPOSIT)** | Share bounds and slippage (`ShareBoundViolation`) | `CoreTest.test_depositMinShares`, `CoreTest.test_depositLooseSlippage`, `BoundariesTest.test_depositObservedShareBound` |
| **Step 7 (Vault)** | Pinned runtime code hash (`VaultCodeChanged`) | `CoreTest.test_vaultCodeChanged` |
| **Step 7 (WITHDRAW)**| Vault liquidity sufficiency (`InsufficientLiquidity`) | `CoreTest.test_withdrawLiquidity` |
| **Step 7 (WITHDRAW)**| Share burn slippage (`ShareBoundViolation`) | `CoreTest.test_withdrawShareBound`, `BoundariesTest.test_withdrawObservedShareBound` |
| **Step 7 (PAY)** | Exact committed leaf proof (`BadObligationProof`) | `CoreTest.test_badProof`, `HashVectorsTest.test_commitments_single_leaf`, `test_commitments_three_leaves` |
| **Step 7 (PAY)** | Double-payment prevention (`ObligationAlreadyPaid`) | `CoreTest.test_duplicatePayAgentAndOwner`, `CoreTest.test_ownerThenAgentDuplicate` |
| **Step 7 (PAY)** | Early payment window (`PayWindowNotOpen`) | `CoreTest.test_payWindow`, `BoundariesTest.test_payWindowExactBoundary` |
| **Step 7 (PAY)** | Payee cooling / cancellation (`PayeeNotActive`) | `CoreTest.test_payeeCooling`, `CoreTest.test_payeeCancelled`, `CoreTest.test_payeeImmutable` |
| **Step 7 (PAY)** | Recipient account match (`RecipientMismatch`) | `CoreTest.test_recipientMismatch` |
| **Step 7 (PAY)** | Sanctions screening (`PayeeSanctioned`) | `CoreTest.test_sanctionedAtPay`, `CoreTest.test_sanctionedAtAdd`, `QueueTest.test_queuePaySanctionsAtQueueAndExecution` |
| **Step 7 (PAY)** | Sanctions oracle failure closed (`SanctionsCheckFailed`)| `CoreTest.test_sanctionsFailClosed`, `CoreTest.test_sanctionsMalformedClosed` |
| **Step 8** | Safe call boolean failure (`SafeCallFailed`) | `SafeIntegrationTest.test_safeCallFailedDeposit`, `test_safeCallFailedWithdraw`, `test_safeCallFailedPay` |
| **Step 8** | Non-standard ERC20 return verification | `SafeIntegrationTest.test_falseApproveRejected`, `test_malformedTokenReturnsRejected`, `test_noReturnTokenSupported` |
| **Step 8** | Asset / share delta verification (`AccountingMismatch`) | `SafeIntegrationTest.test_accountingMismatchPay`, `test_accountingMismatchDeposit`, `test_feeOnTransferRejected` |
| **Step 8** | Zero residual allowance (`ResidualAllowance`) | `SafeIntegrationTest.test_residualAllowanceRejected`, `InvariantTest.invariant_I5_noResidualAllowances` |
| **All Steps** | Reentrancy protection (`nonReentrant`) | `SafeIntegrationTest.test_reentrancyCannotExecuteAgain` |

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

The risk register reflects actual engineering constraints documented in `system architecture/fusion_wallet_architecture.md` §21.1:

| Risk Category | Inherent Threat | Architectural Mitigation | Residual Risk & Governance |
|---|---|---|---|
| **Smart Contract Defect** | Logic flaw in `KavachiModule` leading to asset lock or unauthorized withdrawal. | Allowlisted call targets, value strictly 0, invariants I1–I7, 157 unit tests. Owners can disable the module at any time. | Contracts have not undergone external commercial audit. |
| **Dual Owner Collusion / Compromise** | Both Mei and Wei keys compromised, draining Safe. | Dual signatures required on independent devices. Hardware wallets recommended in production. | Inherent to account owner sovereignty. |
| **Signer Process Compromise** | Attacker compromises `:signer` process memory. | Signer restricted to active session key limits and on-chain policy bounds. | Android OS application process isolation at S (multi-APK sandbox planned for F). |
| **Prompt Injection / Jailbreak** | Adversarial invoice text tricks LLM into attacking treasury. | **AHEM-G Grammar Gate:** Token sampler physically blocks illegal tokens. Output validator rejects non-conforming responses. | Model may still choose a poor *legal* candidate (e.g. HOLD instead of paying a due bill). |
| **Invoice Redirection / Fake Vendor** | Malicious vendor details injected into database. | Mandatory payee cooling delay (3,600s demo; 24h prod), immutable payee record, guardian cancellation. | Social engineering tricking both Safe owners during initial registration. |
| **Sanctions Screening False Negatives** | Sanctioned entity receives payment. | `isSanctioned` oracle checked at payee registration, queuing, and execution. Oracle revert fails closed. | Illustrative testnet oracle list; not a real-time global sanctions service. |
| **Vault Insolvency / Testnet Depeg** | Underlying lending protocol halts or suffers bad debt. | Per-vault exposure caps (6,000 USDC), share slippage bounds, one-tap `emergencyUnwind`. | Third-party smart contract insolvency risk remains with the external protocol. |
| **Delayed Pause Inclusion** | Network congestion delays guardian pause transaction. | Signer maintains local panic latch; delayed tier provides 180s on-chain cancellation window. | Blockchain transaction inclusion latency is subject to Base Sepolia fee market. |
| **Agent Gas Starvation** | Session key runs out of native ETH gas, stalling payments. | Gas runway monitored in UI; owner notifications on low gas balance. | Liveness hazard only; financial safety is unaffected. |
| **AHEM Fork Divergence** | Patched MNN fork diverges from upstream releases. | Pinned submodule commit, tracked patches in `mnn_patches/`, SHA-256 binary validation. | Maintenance overhead tracking upstream MNN security fixes. |
| **Vault Proxy Upgraded Mid-Session** | Underlying ERC-4626 vault implementation changes logic. | Off-chain implementation code hash pinned in manifest and checked on-chain. | Short exposure window between snapshot block and transaction inclusion. |
| **Sub-Optimal Model Reasoning** | Model chooses HOLD when a legitimate bill is due. | Due date alerts in mobile UI; owners can bypass the agent and execute `ownerPayObligation`. | Operational delay; financial safety remains preserved. |

---

## 5. Explicit Limitations & Engineering Boundaries

In compliance with competition guidelines, Fusion Wallet makes the following explicit engineering disclosures:

1. **Testnet Only:** Deployed and evaluated strictly on the **Base Sepolia** testnet. The system manages testnet stablecoins and has not been evaluated for mainnet value.
2. **Unaudited Code:** The smart contracts and Rust libraries are prototype implementations developed for FinTechathon 2026 and have not received an independent security audit.
3. **Illustrative Sanctions List:** The on-chain `KavachiTestSanctionsList` contract (`0x429E2E2088f117c093aC3c50058eE5f3Fcecb46a`) contains static demonstration addresses. It does not interface with a real-time global compliance screening provider.
4. **Demonstration Payee Cooling Window:** The on-chain payee cooling period is configured to **3,600 seconds (1 hour)** on Base Sepolia to enable evaluation during judging sessions. Production deployments require a minimum 86,400-second (24-hour) cooling window.
5. **Simulated Vault Yields:** Vault B (`KavachiDemoVault`) utilizes scenario-based internal drip rates to demonstrate deposit and rebalance math. Yields are illustrative and do not reflect guaranteed financial returns.
6. **EIP-170 Bytecode Proximity:** The compiled runtime bytecode of `KavachiModule` is **24,301 bytes**, which is 275 bytes below the EVM runtime size limit of 24,576 bytes. Further feature expansion requires modular decomposition into additional external libraries.
7. **Mobile Hardware Execution Scope:** AHEM-G decode-time token gating is fully validated on host Linux aarch64 hardware (`evidence/planner/host_results.json`). Android phone cross-compilation via `cargo-ndk` is designated as next-phase engineering (`AHEM-G-07` reported as `NOT_RUN`).
8. **What the Module Does NOT Protect (§7.15):**
   * Actions executed by both Safe owners acting in concert (threshold-two sovereignty).
   * External smart contract vulnerabilities within third-party ERC-4626 vaults or the Aave protocol.
   * Gas expenditures incurred from reverted transactions.
   * The underlying real-world validity of an invoice before dual-owner commitment.
