# Kavachi contracts — Card K1

Solidity 0.8.24, Cancun, via-IR, optimizer 200. Treasury amounts are USDC base units (6 decimals). No proxies or generic execution surface in the module. REBALANCE is deliberately unsupported; OPS_TOPUP is absent.

## Validation

```sh
forge build --sizes
forge test
python3 script/check_abi.py
forge fmt --check
FOUNDRY_PROFILE=ci forge test
```

Default: fuzz 1,000; invariants 64 runs × depth 32. CI: fuzz 10,000; invariants 256 × 64. Reproducible seed: `0x20261019` (configured in both profiles). Foundry 1.8.5 groups the seven invariants into one campaign and reports each I1–I7 check separately.

`STATUS.md` maps every §7.7 guard and I1–I7 to test names. `SPEC_ISSUES.md` records frozen-interface clarifications. Frozen `spec/**` files are never regenerated or changed.

## K2 deployment handoff

1. Use a real Safe v1.4.1 with two owners and threshold 2, treasury USDC with 6 decimals, and two distinct ERC4626 vaults whose `asset()` is that USDC.
2. Deploy `KavachiTestSanctionsList(safe)` (TEST LIST). Deploy `KavachiDemoVault(asset, name, symbol)` for each fallback vault required by the deployment profile (SIMULATED YIELD).
3. Deploy `KavachiPolicyLib` first. Link `src/KavachiPolicyLib.sol:KavachiPolicyLib` when deploying/verifying `KavachiModule`.
4. Constructor: `KavachiModule(safe, asset, vaultA, vaultB, sanctionsList)`. Constructor deploys its own immutable payee registry at CREATE nonce 1 of the module. The registry's `owner` is the Safe; all writes go through the module to atomically invalidate queues. No extra module getter was added to the frozen ABI.
5. Safe enables the module, configures a policy with sorted vault addresses, and adds payees **immediately**. Payee cooling is at least 3,600 seconds; this clock is independent of bill commitment/session activation.
6. Safe commits the obligations, starts a session (at most 24 hours), then calls `resume(getState().policyEpoch)`. Resume increments the epoch; construct actions from the post-resume state.

Safe operation is always Call with value 0. The official `execTransactionFromModuleReturnData` wrapper invokes `execTransactionFromModule` and lets the module reject false/malformed token returns. Stateless linked-library validation follows the §7.14 size split and never performs a financial operation.

Policy changes and commitments pause, clear the session and invalidate a queue. Pause preserves the current session; emergency unwind and revoke clear it. Daily usage survives session changes. Cancel/expire/invalidate consumes a queued nonce. Owner bill payment invalidates an existing queue because the mandate reserve changes.

Emergency unwind caps each Safe redemption leg at 350,000 gas and each share-balance read at 30,000, so one failing or gas-consuming vault cannot prevent attempting the other. A leg that needs more gas is reported failed; owners can use normal threshold-two Safe transactions for recovery. Vault runtime code is checked; proxy implementation monitoring remains off-chain as specified.

`MockUSDC` and the deliberately unprotected `NaiveAgentWallet` reject deployment outside Anvil chain ID 31337. Do not use either on Base Sepolia. `drip(assets)` is deployer-only and transfers the deployer's approved assets into the demo vault as **simulated yield**.

## Dependencies

Vendored upstream source snapshots (no source patches): OpenZeppelin Contracts **v5.0.2**, forge-std **v1.9.7**, Safe smart account **v1.4.1**. Each library includes `PINNED_VERSION` with the exact upstream commit and its original license. This avoids mutating the repository's root `.gitmodules` outside K1's ownership. Safe test artifacts alone compile without IR because unmodified upstream Safe assembly fails solc's IR compilation; production `src/**` is restricted to via-IR.
