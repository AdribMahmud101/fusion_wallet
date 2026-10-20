# Yield Scout

Screens where spare cash could earn, with fixed-code rules. Screening, not financial advice.

- **Data:** DeFiLlama yield pools for USDC on Base (mainnet), through `proxy/`. Cached on the phone; the age shows on screen.
- **Seven rules per pool** (`scout/ScoutRules.kt`): TVL of at least $10M; not a rate spike (within 50% of the 30-day mean); yield mostly organic (base yield at least 50%); audited lending protocol on an allowlist; single-asset stablecoin; liquidity (not published by the feed, so "not measured"); pays for its gas within 14 days (measured agent gas cost and spare cash).
- **Eligible:** no rule fails and at least 5 pass. Ranking is deterministic: eligible first, then fewer failures, then the lower of spot and 30-day APY, then pool size.
- **The cloud advisor only explains the top pick** ("Why this pick?"). Its reply must cite the numbers it was given or it is rejected. It cannot change the ranking.
- **Acting on a pick:** only Aave v3 is executable here (Base Sepolia). "Ask the AI agent to act on this" starts an Autopilot tick: the agent re-checks every rule and the on-chain limits, then executes, previews (no session key on the phone) or holds with the reason. Other pools produce a "Share proposal with owners" message; adding a venue needs the Safe 2-of-2.
- **Honest limits:** pool data is mainnet and this app runs on a test network; the audit allowlist is a short fixed list; no utilization for mainnet pools.
