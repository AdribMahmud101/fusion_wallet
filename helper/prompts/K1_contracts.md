# Card K1 — Kavachi contracts (Phase 2–3, CRITICAL PATH)

You are the Solidity engineer for Fusion Wallet (FinTechathon 2026, Topic F, hard deadline 2026-10-20 15:59 UTC).
Deployment to Base Sepolia (card K2) is blocked on you. Ship a correct, tested core first; polish later.

## Where you work
- Worktree: `/root/projects/fusion_wallet-wt/K1` (branch `card/K1`). `cd` there first. Work ONLY there.
- Spec (read-only): `system architecture/fusion_wallet_architecture.md` — §6 (tiers, limits), §7 (all of it), §8.3 payees, §8.5 sanctions, §9 hashing, §24 Appendix A (the interface).
- Frozen inputs (read-only, already merged): `spec/abi/*.json`, `spec/vectors/v4_1.json`, `spec/reference/src/IKavachiModule.sol`, `spec/reference/src/HashV41.sol`, `spec/DECISIONS.md`. Your ABI MUST equal `spec/abi/IKavachiModule.json`. Read `spec/DECISIONS.md` before you start.
- Toolchain: forge/cast/anvil 1.8.5, solc 0.8.24.

## Owned paths (exclusive)
`src/contracts/**` only. Never edit `spec/**` or the architecture doc. If the frozen ABI is wrong, write the problem into `src/contracts/SPEC_ISSUES.md` and work around it — do not change the freeze.

## Build
1. Foundry project at `src/contracts/`: solc 0.8.24 pinned, `evm_version = "cancun"`, `via_ir = true`, optimizer 200 (D15). OpenZeppelin Contracts v5 (pinned tag) + forge-std under `src/contracts/lib` (`forge install --no-git` is fine).
   Profiles: `default` = fuzz 1000 runs, invariant 64 runs × depth 32 (fast, this is the DoD). `ci` = fuzz 10000, invariant 256 × 64 (§7.1).
2. Contracts (§7.1): `KavachiModule` (implements the frozen interface; Safe module calling `execTransactionFromModule`; NO delegatecall, NO generic execute), `KavachiDemoVault` (OZ ERC4626 + deployer `drip()`, labelled simulated yield), `KavachiTestSanctionsList` (owned by the Safe, TEST LIST), `MockUSDC` (6 dp, EIP-3009, Anvil only), `NaiveAgentWallet` (deliberately unprotected comparator, Anvil only, labelled).
3. Scope cuts already decided: REBALANCE reverts `UnsupportedAction` (D16). No OPS_TOPUP (D3). Apply the SL1 cut order only if you are not green by 14:00 UTC.
4. Hashing: use the frozen §9 formulas. Port `spec/reference/src/HashV41.sol` logic; a test MUST re-check every vector in `spec/vectors/v4_1.json` against the module's own hashing (D18).
5. Tests (`src/contracts/test/`): one unit test per §7.7 revert for the S kinds (PAY, DEPOSIT, WITHDRAW), queue/cancel/expire (§7.8), tiers (§6.2), cumulative limits (§6.3), payee cooling/immutability/sanctions (§8.3, §8.5), duplicate pay (I4), pause/unwind (§7.10), decisionHash event (§7.11). Handler-based invariant tests I1–I7 (§7.13). Use a real Safe v1.4.1 on the test chain (deploy the official contracts in setUp, e.g. via `safe-smart-account` as a lib) — not a mock Safe — for at least one end-to-end test of pay + deposit through the module.
6. Size gate (§7.14): `forge build --sizes` → `KavachiModule` runtime < 24,576 bytes. If over, apply the §7.14 split order.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K1/src/contracts && forge build --sizes && forge test
```
exits 0, AND `KavachiModule` runtime size < 24,576 in that output, AND `src/contracts/STATUS.md` lists every §7.7 check and I1–I7 with the test name that covers it (or "NOT COVERED" + why).

## Acceptance checks
- No test weakened, skipped, or deleted to go green. No `vm.assume(false)` tricks. No mocks of the module itself.
- Money is integers only. No private keys or secrets in files (use `makeAddr`/`vm.addr`).
- Commit on `card/K1` at each green milestone (e.g. module compiles; unit tests green; invariants green; size gate). Do not merge, do not push.

## Unblocks
K2 (deploy + evidence) and K9 (golden path). Speed matters: a green PAY + DEPOSIT + WITHDRAW + pause core by 13:00 UTC beats a perfect module at 18:00.
