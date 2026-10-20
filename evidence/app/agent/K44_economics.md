# K44 measured deposit economics

Read-only Base Sepolia capture at block **48,358,262**, hash and timestamps in
[K44_live_inputs.json](K44_live_inputs.json). No policy or chain state was changed.
The same production Rust formulas produced [K44_quant_report.json](K44_quant_report.json).

- Aave utilization: **20.69%** (display rounded upward); deposit guard permits <=85%.
- Available underlying / withdrawable now: **26,834,319.796431 test USDC**.
  This is shared pool liquidity, not a guarantee that the Safe owns that much.
- Spot compounded supply APY: **0.2055852%**. Realized APY from normalized
  liquidityIndex growth over 86,400 seconds: **0.2065335%**. Conservative APY:
  **0.2055852%**. Current and historical index reads succeeded on `https://sepolia.base.org`;
  Android also tries `https://base-sepolia-rpc.publicnode.com` if needed.
- K33 median actual fee: **1,342,800,000,259 wei**, from 17 indexed session
  receipts captured in `feeSamples` (same most recent 20 / 30-day filter,
  gasUsed × effectiveGasPrice + L1/operator fees, median rounded up to a wei).
- Chainlink ETH/USD: **2,387.78 USD**, pinned proxy
  `0x4aDC67696bA383F43DD60A9e78F2C97Fbbfc7cb1`, 8 decimals. Latest round,
  answeredInRound and updatedAt are captured. Maximum age **1,200 seconds**,
  matching the published heartbeat. Missing, stale, future, nonpositive or
  incomplete rounds yield **UNMEASURED**, never an assumed ETH price.
- Gas cost rounds upward to **0.003207 USD**. The real ERC-4626
  previewDeposit → convertToAssets round-trip loss is **0.000001 USDC**.
  The policy's 50 bps slippage tolerance is a bound, not a measured cost.

At **10 USDC**, current cash APY = 0, the deposit is
**UNPROFITABLE_AFTER_GAS**: break-even **56.96 days** (rounded upward).
It is excluded before planner inference. The report retains the rejected
measurement, so a HOLD can explain the economics.

At those measured costs and conservative APY, the mathematical threshold is
**40.682473 USDC**. The smallest whole-USDC size is **41 USDC**. We separately
re-quoted 41 USDC at the same block; the loss remained 0.000001 USDC and the
production report marked it **ACTIONABLE**, **13.90 days**. See
[K44_actionable_inputs.json](K44_actionable_inputs.json) and
[K44_actionable_report.json](K44_actionable_report.json).

**Supervisor decision:** consider an owner 2-of-2 proposal for a 41 USDC instant
per-move cap. This is a measured proposal, not an authorization: the current
10 USDC cap remains unchanged. Re-read rates, receipts, oracle and quote before
changing policy or executing; a threshold is valid only for its captured costs.
K38's share-quote binding work must also be integrated before live execution.
The current action preview and on-chain share guards still apply.

## Data and arithmetic

Deterministic Rust uses integer USD micro-units and APY fractions in parts per
billion. Gas USD and displayed break-even days round upward; APY fixed-point
compounding rounds downward. Classification compares the exact fraction with
14 days before display rounding. Zero/negative APY improvement is unprofitable.
Historical return compounds the index ratio over its actual elapsed time
(including a fractional exponent); a missing archive is explicitly `spot only`.
Invalid supplied historical data blocks yield moves instead of disguising it
as an archive failure. Rebalances remain unsupported by the on-chain module;
the shared formula handles their APY delta without enabling that route.

Addresses: [Aave address book](https://github.com/aave-dao/aave-address-book/blob/main/src/AaveV3BaseSepolia.sol).
Feed metadata: [Chainlink reference directory](https://reference-data-directory.vercel.app/feeds-ethereum-testnet-sepolia-base-1.json).

## Integration context

This card includes committed K39 Autopilot (root `62b26b4`, Android `2b09a30`)
as a prerequisite. K38's diagnosis was read from its active worktree at
`/root/projects/fusion_wallet-wt/K38/evidence/app/agent/K38_hold_diagnosis.md`;
its runtime edits were uncommitted and were not copied over. The requested
`docs/Perception & Ingestion.md` is absent from this checkout and the available
project worktrees; the explicit K44 brief and frozen WIN_PLAN supply the rules.
The K38 legacy 30-day / fixed-fee scenario filter is replaced for yield moves by
the requested measured 14-day break-even gate. No owner policy is relaxed.

## K47 live re-quote for 50 USDC (2026-10-20)

The requested fresh capture is [K47_50_live_inputs.json](K47_50_live_inputs.json),
with the production Rust result in [K47_50_quant_report.json](K47_50_quant_report.json).
At Base Sepolia block **48,364,253**, utilization was **20.32%** and the
conservative APY was **0.1983033%** (the lower of spot and the approximately
24-hour realized rate). The latest Chainlink ETH/USD answer was **2,385.018**;
the median actual receipt fee was **1,985,676,000,148 wei**. For a 50 USDC
deposit, the exact ERC-4626 preview round-trip loss was **0.000001 USDC**, the
gas estimate rounded upward to **0.004736 USD**, and break-even was **17.44
days**. The production report classifies it **UNPROFITABLE_AFTER_GAS**; the
measured minimum size for the 14-day ACTIONABLE gate was **62.278519 USDC**.

Therefore the proposed 50 USDC instant-tier change was not submitted and no
policy field, session, or epoch was changed. The live planner was not triggered:
the 50 USDC candidate is excluded before inference by the same economics gate.
This capture does not justify substituting a larger policy change.
