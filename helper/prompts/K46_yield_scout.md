# K46 — Yield Scout: the AI screens where money can earn, recommends, owners decide

Read `docs/WIN_PLAN.md` (hard security invariants), `docs/Perception & Ingestion.md` (B: protocol collector, 1.C break-even,
rate mean-reversion), `docs/ux/UX_PLAN.md`, `docs/ux/DASHBOARD_LAYOUT.md`, and the merged K40 (`advisor/*`) and K44 (quant)
code if present. Android: submodule `src/Fintech_Wallet_Agent/fusion-core-android` (commit there, then the pointer).
You own new `scout/*` and `ui/scout/*`. Do not edit `advisor/*`, `agent/autopilot/*`, `agent/risk/*` beyond calling their APIs.

## Rules
- Screening, not advice. Recommendation only. Nothing here moves money or signs.
- Off-chain data is untrusted: only typed numbers pass; raw text never reaches the planner or a signer.
- Feed down or stale → "Not measured", never cached-as-fresh or guessed.

## Data (open sources, no keys)
1. DeFiLlama yields API (`https://yields.llama.fi/pools`, and `/chart/{pool}` for history): USDC single-asset lending pools on Base
   (chain "Base", stablecoin, no IL). Fields: project, symbol, tvlUsd, apy, apyBase, apyReward, apyMean30d, pool id.
2. On-chain cross-check for Aave v3 USDC on Base Sepolia (the venue we can execute): utilization, available liquidity, paused/frozen,
   spot supply rate (reuse K44 readers if merged). Show where mainnet data and testnet reads disagree.
3. Cache last good response with its fetch time; show age.

## Deterministic checklist (code, versioned, shown to the user)
Each pool gets PASS/FAIL/NOT MEASURED per rule with the source value:
TVL ≥ $10M · utilization < 85% (where measurable) · |apy − apyMean30d| ≤ 50% of apyMean30d (no bait spike) · reward share of
APY ≤ 30% (sustainable yield) · protocol on our allowlist of audited lending protocols (list it in code with audit links) ·
stablecoin peg ok (Chainlink USDC/USD if K41/K44 reader exists) · gas-adjusted break-even ≤ 14 days for the user's spare cash
(K44 formula; NOT MEASURED if K44 absent). Score = passes / measured. Only all-PASS pools are "Eligible".

## AI layer
- Ranking among Eligible pools is deterministic (conservative APY = min(apy, apyMean30d), then TVL).
- If the K40 advisor is on: "Why this pick?" plain-English rationale citing the numbers, labelled "Cloud explanation · <model>".
  The advisor never changes the ranking or the checklist result.
- Action button per Eligible pool:
  - If the venue is already allowlisted and deployed on Base Sepolia (Aave today): "Preview a move" → existing dry-run Preview sheet.
  - Else: "Propose to owners" → exact Safe tx payload (to, calldata) to allowlist the vault in KavachiModule (read `src/contracts`
    for the real function), with "Needs owner signatures — the AI cannot apply this" and copy/share. If the venue has no
    Base Sepolia deployment: "Mainnet data · not available on test network" and no payload.

## UI/UX (dark obsidian bento, real data only)
1. New "Earn" destination: add to the bottom nav only if it stays ≤5 items cleanly; otherwise a Home bento tile "Yield Scout"
   (top pick, its conservative APY, "3 of 12 pools eligible") opening the full screen. Pick what reads best and say why.
2. Scout screen: header "Where your spare cash could earn" + spare cash amount + data age chip ("DeFiLlama · 4 min ago").
   Filter chips: Eligible / All / Executable on testnet.
3. Pool cards: protocol icon (use bundled vector icons already in the app or simple monogram; no remote images), name, conservative
   APY large, spot vs 30-day mean mini sparkline (from `/chart/{pool}`, last 30 points), TVL, utilization bar, checklist summary
   ring "6/7", badge Eligible / Not eligible / Not measured, badge "Executable on testnet" when true.
4. Pool detail sheet: full checklist table (rule, value, threshold, PASS/FAIL, source), break-even line ("pays for its gas in 2.3 days
   on 8,899 USDC"), the action button, and the disclaimer "Screening, not financial advice. Test network."
5. Empty/error states designed, not blank. Motion consistent with K29 chart animations. Accessible contrast, 48dp touch targets.
6. Update `docs/ux/DASHBOARD_LAYOUT.md`, and add Yield Scout to `docs/SYSTEM_OVERVIEW.md` (features + one flow diagram).

## Tests and build
Unit tests: checklist rules with edge values, ranking, parser on a recorded API fixture (commit a small fixture), stale/down feed,
payload encoding against the contract ABI. `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`
(`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`; native libs via `scripts/android_planner/build.sh` +
`build_core.sh` from worktree root). `python3 scripts/judge/verify_all.py` still passes. Commit message lists the live pool counts seen.
