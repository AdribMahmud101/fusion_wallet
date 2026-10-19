# K1 contract status

Implementation and validation complete. K2/K9 can proceed with the deployment handoff in `README.md`. No SL1 cuts were applied. PAY, DEPOSIT, WITHDRAW, all permission tiers, approvals/revocation, queueing, pause and unwind are implemented. REBALANCE reverts `UnsupportedAction` (D16); no OPS_TOPUP (D3).

## Build and evidence

- Foundry 1.8.5; solc 0.8.24; Cancun; via-IR; optimizer 200.
- Default fuzz 1,000; invariant 64 × 32. CI fuzz 10,000; invariant 256 × 64.
- Reproducible fuzz/invariant seed: `0x20261019`.
- `forge build --sizes && forge test`: PASS — 157 tests reported, zero failures/skips; I1–I7 pass in a 64-run, depth-32 campaign (2,048 calls). See `size-report.txt` and `test-report.txt`.
- `FOUNDRY_PROFILE=ci forge test`: PASS — fuzz 10,000 and invariants 256 × 64 (16,384 handler calls, zero handler reverts), all 157 reported tests pass. See `ci-report.txt`.
- `KavachiModule` runtime: **24,301 bytes**, 275 bytes below EIP-170 (24,576). `KavachiPolicyLib`: 5,854 bytes. No split removed a frozen ABI method.
- `forge fmt --check`: PASS. `python3 script/check_abi.py`: PASS — all 93 frozen entries identical, excluding only the concrete constructor.
- `forge script script/GenVectors.s.sol:GenVectors`: PASS; computed action digests match the freeze. See `vectors-report.txt`.
- `python3 script/check_abi.py`: compares all frozen function, event and error entries, including names, tuple components, mutability and indexing. Constructor-only exception explained in `SPEC_ISSUES.md`.
- `CoreTest.test_realSafe_payDepositWithdraw` deploys the official unmodified Safe v1.4.1 singleton + proxy factory, a threshold-two proxy, enables the module through a transaction signed by both owners, and pays, deposits and withdraws through the actual module. `test_realSafe_signedPolicyAndOwnerPayment` additionally submits policy/owner-payment transactions with both signatures.
- PAY gas: **292,385 gas** for `KavachiModule.execute(PAY_OBLIGATION)` in the real Safe fixture (`forge test --gas-report --match-contract BoundariesTest --match-test test_payPreservesFreeCashAndMandateCommitsRemainingReserve`). See `gas-report.txt`.
- Exact upstream dependency commits: `lib/*/PINNED_VERSION`. No dependency source patches.

## §7.7 normative checks

Shared guards are tested separately against PAY, DEPOSIT and WITHDRAW in `SharedChecksTest`; kind-specific tests use real module instances. All names below are in `test/`.

| Step | Required check / revert | Covering test(s) |
|---|---|---|
| 1 | Deployment chain / `WrongChain` | `test_commonWrongChainAllKinds` |
| 1 | Module enabled / `ModuleNotEnabled` | `test_commonDisabledAllKinds` |
| 1 | Paused / `ModulePaused` | `test_commonPausedAllKinds`; `test_pausedQueueForbidden` |
| 1 | Caller is session key / `Unauthorized` | `test_commonUnauthorizedAllKinds`; `test_onlySafeAdministration` |
| 1 | Live session / `SessionExpired` | `test_commonSessionExpiredAllKinds`; `test_ownerSessionKeyGuardianSeparation` |
| 1 | Permission tier / `ReadOnly` | `test_commonReadOnlyAllKinds` |
| 2 | Existing queue excludes other digests / `QueueBusy` | `test_commonQueueBusyAllKinds`; `test_queueBusyBeforeNonceAndPolicy` (also proves ordering) |
| 3 | Current nonce / `BadNonce` | `test_commonBadNonceAllKinds`; `test_queueCancelAndReplay` |
| 3 | Current policy / `StalePolicy` | `test_commonStalePolicyAllKinds`; `test_pauseIdempotentAndSafeOnlyResume` |
| 3 | Expired action / `ActionExpired` | `test_commonDeadlineExpiredAllKinds` |
| 3 | Immediate deadline ≤120 s / `InvalidDeadline` | `test_commonDeadlineTooFarAllKinds` |
| 3 | Approved deadline ≤900 s | `test_approvedDeadlineBound`; `test_approvalValidAtExpiryAndConsumed` |
| 3 | Deadline ≤session expiry | `test_deadlineAfterSession` |
| 3 | Queued deadline bounds and expiration | `test_queueDeadlineLowerAndUpper`; `test_queueExpiresAndConsumesNonce`; `test_queueExpiryBoundary` |
| 3 | Decision hash nonzero / `MissingDecisionHash` | `test_commonMissingDecisionAllKinds` |
| 4 | Kind-specific field shape / `UnsupportedAction` | `test_badDepositShape`; `test_badWithdrawShape`; `test_badPayShape`; `test_rebalanceUnsupported` |
| 4 | Nonzero amount / `InvalidAmount` | `test_commonZeroAmountAllKinds` |
| 4 | Hard cap / `HardLimitExceeded` | `test_commonHardLimitAllKinds`; `test_tierEdgesMicroDelayedAndOwner` |
| 5 | UTC-day gross / `DailyLimitExceeded` | `test_commonDailyLimitAllKinds`; `test_dailyRolloverAndSessionRestartCannotResetDay` |
| 5 | Session gross / `SessionLimitExceeded` | `test_commonSessionLimitAllKinds`; `test_sessionLimit` |
| 6 | Instant/delayed/owner tier boundaries | `test_tierEdgesMicroDelayedAndOwner`; `test_ownerApprovedInstant`; `test_ownerApprovedDelayedNeedsApprovalAndQueue` |
| 6 | Missing delayed queue / `QueueRequired` | `test_commonDelayedRequiredAllKinds`; `test_delayedRequiresQueueEvenWithApproval` |
| 6 | Queue not ready / `QueueNotReady` | `test_commonDelayedNotReadyAllKinds`; `test_queueReadyExecutionNoChargeUntilExecute` |
| 6 | Queue session identity / `QueueMismatch` | NOT COVERED directly: unreachable through the public lifecycle. Session replacement always invalidates the queue (`test_revokeSessionInvalidatesQueue`, `test_commitInvalidatesQueueAndSession`); explicit digest mismatch is tested by `test_queueMismatchCancel`. A stale-session queue cannot be created through the public lifecycle. |
| 6 | Missing approval / `ApprovalRequired` | `test_commonApprovalRequiredAllKinds`; `test_revokeApproval` |
| 6 | Expired approval / `ApprovalExpired` | `test_commonApprovalExpiredAllKinds` |
| 7 DEPOSIT | C≥R / `Underfunded` | `test_depositUnderfunded`; handler I1 with `ownerMoveCash` |
| 7 DEPOSIT | C−assets≥R+B / `ReserveViolation` | `test_depositReserve`; `test_queueRerunsReserve`; `testFuzz_depositPreservesReserve` |
| 7 DEPOSIT | Allowed destination / `ForbiddenVault` | `test_forbiddenVault`; `test_vaultNotInPolicy` |
| 7 DEPOSIT | Exposure cap / `ExposureExceeded` | `test_exposureExceeded` |
| 7 DEPOSIT | Preview and min-share slippage / `ShareBoundViolation` | `test_depositMinShares`; `test_depositLooseSlippage`; `test_depositObservedShareBound` |
| 7 vault | Pinned runtime / `VaultCodeChanged` | `test_vaultCodeChanged` |
| 7 WITHDRAW | Max liquidity / `InsufficientLiquidity` | `test_withdrawLiquidity` |
| 7 WITHDRAW | Preview and observed burn ≤max / `ShareBoundViolation` | `test_withdrawShareBound`; `test_withdrawLooseSlippage`; `test_withdrawObservedShareBound` |
| 7 underfunded | WITHDRAW/PAY remain available | `test_withdrawAllowedUnderfunded`; `test_payAllowedUnderfunded` |
| 7 REBALANCE | Unsupported at S | `test_rebalanceUnsupported` |
| 7 PAY | Exact committed leaf / `BadObligationProof` | `test_badProof`; `test_commitments_single_leaf`; `test_commitments_three_leaves` |
| 7 PAY | Immortal paid marker / `ObligationAlreadyPaid` | `test_duplicatePayAgentAndOwner` (includes recommit); `test_ownerThenAgentDuplicate` |
| 7 PAY | Early-pay window / `PayWindowNotOpen` | `test_payWindow`; `test_payWindowExactBoundary` |
| 7 PAY | Cooling and terminal cancellation / `PayeeNotActive` | `test_payeeCooling`; `test_payeeCancelled`; `test_payeeImmutable` |
| 7 PAY | Registry recipient / `RecipientMismatch` | `test_recipientMismatch` |
| 7 PAY | Sanctions / `PayeeSanctioned` | `test_sanctionedAtPay`; `test_sanctionedAtAdd`; `test_queuePaySanctionsAtQueueAndExecution` |
| 7 PAY | Reverting/malformed oracle / `SanctionsCheckFailed` | `test_sanctionsFailClosed`; `test_sanctionsMalformedClosed` |
| 7 PAY | Cash / `Underfunded` | `test_payInsufficientCash` |
| 7 PAY | Declared reserve / `ReserveViolation` | `test_payExceedsReserve` |
| 8 | Safe boolean failure / `SafeCallFailed` | `test_safeCallFailedDeposit`; `test_safeCallFailedWithdraw`; `test_safeCallFailedPay` |
| 8 | ERC20 false/malformed return / `SafeCallFailed` | `test_falseApproveRejected`; `test_falseTransferWithBalanceChangeRejectedAtomically`; `test_malformedTokenReturnsRejected`; `test_noReturnTokenSupported` |
| 8 | Cash/share deltas / `AccountingMismatch`, `ShareBoundViolation` | `test_accountingMismatchPay`; `test_accountingMismatchDeposit`; `test_accountingMismatchWithdraw`; `test_feeOnTransferRejected`; observed-share tests above |
| 8 | Zero allowance / `ResidualAllowance` | `test_residualAllowanceRejected`; I5 |
| 8 | Nonce, usage, queue/approval consumption, paid/R effects | `test_realSafe_payDepositWithdraw`; `test_queueReadyExecutionNoChargeUntilExecute`; `test_approvalValidAtExpiryAndConsumed`; `test_payPreservesFreeCashAndMandateCommitsRemainingReserve`; I3–I7 |
| 8 | Decision event | `test_decisionHashEventAndBinding`; `test_queueDecisionHashEvent` |
| all | Reentrancy on state changes | `test_reentrancyCannotExecuteAgain` |

## §7.13 handler invariants

`TreasuryHandler` targets real Safe/module/vault/token deployments. Calls include deposits, withdrawals, queued actions, cancellation, expiry, pause/resume, session renewal, owner/agent bill payments and replay attempts, sanctions changes, time changes, cash donations, owner cash withdrawals (creating shortfalls), and unwind. Expected module reverts are caught; successful-call assertions are not caught. `fail_on_revert=true` ensures a failing handler assertion fails the campaign. `test_handlerExerciseAllRoutes` independently proves that the handler reaches successful financial and queue paths.

| ID | Test name | Independent evidence |
|---|---|---|
| I1 | `invariant_I1_reservePreservedOnInvestment` | Every successful handler deposit asserts C≥R+B before/after; conservation includes owner outflows, donations and withdrawals. |
| I2 | `invariant_I2_onlyTrustedDestinations` | Actual vault/payee/owner cash balances match independent flow totals; successful payments independently recheck active status, sanctions and committed leaf. |
| I3 | `invariant_I3_grossDailyAndSessionLimits` | Ghost sums keyed by UTC day/session; compared with current state and checked against limits after every success. |
| I4 | `invariant_I4_paidAtMostOnce` | Separate per-obligation payment count, asserted on agent and owner successes; retries include both paths. |
| I5 | `invariant_I5_noResidualAllowances` | Both Safe-to-vault allowances equal zero after every handler transaction, including reverts. |
| I6 | `invariant_I6_pauseStopsAgentAndOnlySafeResumes` | Paused probes attempt execute/queue as agent and resume as guardian; all must fail. |
| I7 | `invariant_I7_nonceConsumptionAndDigestUniqueness` | Independent expected nonce with exact increments for execution/cancel/expiry/invalidation; per-digest execution counts. |

## Remaining submission checks

`HashVectorsTest` imports the module's production `src/HashV41.sol` and rechecks every frozen vector category (tags, sorted vault limits, policy, session/no-session, mandate, all four action shapes, candidate, empty/single/three-leaf commitments and proofs, and decision JSON). Each frozen action also passes through the deployed module's `hashAction` with its live domain. `test_hashActionUsesProductionDomainAndState` and `test_policyHashSessionIdAndMandateFollowProductionHashing` check actual module state/domain bindings. `script/GenVectors.s.sol` prints read-only regeneration evidence and cannot overwrite the freeze.

`test_emergencyUnwindBothVaultsUnderMillionGas`, `test_emergencyUnwindFailedLegContinues`, `test_unwindGasConsumingLegCannotBlockOther`, and `test_unwindChecksObservedBalancesAndReturn` cover unwind success, failure isolation, observed cash/share/return-data accounting, and the 1,000,000-gas envelope. The TEST sanctions list, simulated-yield drip, Anvil deployment guards and EIP-3009 authorization/cancellation/replay/domain checks have dedicated tests.

Every reachable §7.7 guard and all I1–I7 have coverage. The unreachable defensive queue-session mismatch is marked NOT COVERED directly above; lifecycle invalidation tests establish why it cannot occur without corrupting module storage. Base Sepolia deployment and source verification belong to K2.
