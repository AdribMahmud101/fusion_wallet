# Assets, currencies and Aave — verified facts and plain-English glossary

Verified on-chain against **Base Sepolia (chain 84532)**, public RPC `https://sepolia.base.org`, block 48,238,481, on 2026-10-17. Addresses come from the official `bgd-labs/aave-address-book` (`AaveV3BaseSepolia.sol`) and were confirmed by read-only `cast call`s. Re-verify before the final demo: testnet reserves can be paused or reconfigured.

## Decision (owner, 2026-10-17)
Treasury unit = **Aave's Base Sepolia test USDC**. Vault A = **real Aave USDC market via its ERC-4626 wrapper**. Vault B = **our own OpenZeppelin ERC-4626 demo vault** on the same USDC (also useful as the "attacker vault" in the arena). This replaces the spec's DemoUSD + two custom vaults.

## Verified contracts (Base Sepolia)
| Thing | Address | Notes |
|---|---|---|
| Aave V3 Pool | `0x8bAB6d1b75f19e9eD9fCe8b9BD338844fF79aE27` | `getReserveData(USDC)` returns live data |
| USDC (Aave test token) | `0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f` | symbol `USDC`, 6 decimals |
| aUSDC (`aBasSepUSDC`) | `0x10F1A9D11CDf50041f3f8cB7191CBE2f31750ACC` | rebasing ERC-20 receipt, 6 decimals |
| waUSDC (`waBasSepUSDC`) | `0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a` | **ERC-4626** wrapper: `asset()` = USDC, `totalAssets`, `convertToAssets`, `previewDeposit`, `maxDeposit` all respond |
| Other reserves | USDT `0x0a215D8b…E54a`, WBTC, WETH `0x4200…0006`, cbETH, LINK | volatile / not managed by the treasury |

## Verified market state (read 2026-10-17)
- USDC reserve: **active = 1, frozen = 0, paused = 0**, supply cap 0 (unlimited), borrowing enabled.
- `currentLiquidityRate` = `2074789914389810119812431` ray → **0.2075 % APR** (≈ 0.2077 % APY). Borrow APR ≈ 1.11 %.
- Liquidity index 1.244772; waUSDC share price (`convertToAssets(1e6)`) = 1.244772 — they match, as they should.
- **Yield is tiny on a testnet.** Do not present APY as the value proposition. The story is safe automation.

## Known gaps / risks
- We have **not** yet confirmed how to obtain test USDC (Aave faucet) or that the owner/agent can `deposit` into waUSDC. Confirm on a fork/Anvil before building the contract against it.
- Our contract's vault universe is immutable; waUSDC must be pinned by address and code hash in the deployment manifest.
- Aave could pause/freeze the testnet reserve before judging; keep our own vault B + a recorded replay as fallback.
- Organizer acceptance of third-party testnet assets is unverified (spec says check separately).

## Plain-English glossary
| Word | Meaning |
|---|---|
| **Gas / gas token (ETH)** | The fee for every transaction, paid in the chain's native coin. **Not an ERC-20.** The agent's own account must hold some or it cannot act. Not part of treasury value. |
| **ERC-20** | The standard for fungible tokens (USDC, WETH, aTokens). ETH itself is *not* one; WETH is ETH wrapped into one. |
| **Stablecoin** | An ERC-20 meant to stay ≈ $1 (USDC). Our unit of account. |
| **aToken** | Aave's receipt token for a deposit; its balance grows by itself as interest accrues (rebasing). |
| **ERC-4626 vault / waUSDC** | A standard "deposit → get shares" wrapper. Share count stays fixed; the *price per share* rises. waUSDC is Aave's aToken wrapped this way. |
| **Supply / lending** | Depositing a stablecoin into Aave so borrowers pay you interest. |
| **APR vs APY** | APR = simple yearly rate (what Aave reports). APY includes compounding. We show **APR**. |
| **Bills / obligations** | Payments the owner approved and wants cash set aside for (the "reserved" bucket). |
| **Safety buffer** | Extra cash kept on top of bills. |
| **Safe to deploy (sweepable)** | Cash − bills − buffer. The only money the agent may move. |
| **Mandate** | The owner's delegation to the agent: spend tiers, caps, expiry. Enforced by the contract, only *displayed* by the app. |
| **Tier** | ≤ 10 USDC acts instantly; ≤ 100 waits (cancellable); above needs the owner. |
| **Rebasing vs share-price** | aToken balance grows; waUSDC shares stay constant while their price grows. |

## What counts toward "total assets"
| Asset | ERC-20? | In treasury total? | Shown as |
|---|---|---|---|
| USDC | yes | yes | Cash |
| aUSDC / waUSDC | yes | yes | Earning |
| ETH (gas) | **no** | **no** | Gas fuel (+ runway in agent actions) |
| WETH, WBTC, cbETH, LINK, USDT | yes | no | "Other tokens · not managed" |
