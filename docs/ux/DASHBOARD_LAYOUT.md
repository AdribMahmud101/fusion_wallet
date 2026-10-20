# Fusion Wallet — dashboard layout

What every screen of the Android app shows, where each number comes from, and what it looks like when data is missing. Verified on a Tecno CL6 (1080×2436) against the live Base Sepolia deployment.

**Visual system:** dark obsidian bento grid. Tiles are rounded (18 dp), `#111418` fill with a thin `#1F242B` border, and 12 dp gaps. Titles are small and muted; big numbers use a sans face; amounts read `9,399.00 USDC`. Chart palette: green (cash), amber (reserved/bills), sky (earning/lines), teal and coral (spend categories). Every status chip carries text, never color alone, and every chart has an ⓘ button whose sheet explains its method. Charts animate in once (shared 700 ms ease-out entrance) and respect the system animation setting.

**Rule zero:** every number is read from the chain or from evidence files in this repository. When data is insufficient, the tile says so ("Early estimate", "Not enough history yet"); it never draws a fake shape.

---

## Shell (every screen)

| Element | Shows | Source |
|---|---|---|
| Top bar | FUSION logo · `● Base Sepolia · TEST` badge · Settings (gear) · avatar | network from `deployments/84532.json`; avatar = signed-in demo user |
| Bottom nav | Home · Bills · AI Agent · Activity · Safety | — |
| Avatar menu | Profile, language (English / 中文), sign out, Judge mode | local |

## Sign-in

One-tap demo accounts: **Lin Mei** (Owner), **Chen Wei** (Finance approver), **Guest Judge** (Auditor, view-only). Login is a UI convenience; authority lives in keys and the Safe 2-of-2.

## First-run setup (6 steps; Back / Skip / Next on each)

| Step | What it does (really) |
|---|---|
| 1 Welcome & role | Owner / Finance approver / Auditor; preselects the signed-in role |
| 2 Network & currency | Base Sepolia TESTNET; treasury currency USDC read from the deployment; language |
| 3 Connect treasury | Live checks at one block: Safe has code, threshold 2 of 2, KavachiModule enabled, Sourcify links |
| 4 On-device AI model | Downloads LFM2.5-230M (~131 MB, resumable, Wi-Fi-only option), SHA-256 verifies 5 files, then a warm-up self-test decision (letter + decode ms) |
| 5 Keys | Authority table (owners = Safe 2-of-2; agent session key = module only; guardian = Stop only); import keys into Android Keystore or stay view-only |
| 6 Policy review | Read-only from chain: permission tier, daily and session limits, reserve buffer, verified vendors (vendors load after the rest) |

---

## Home — "Your money at a glance"

1. **Spare cash** (hero, green gradient): surplus bucket in USDC; ✓ "Your bills are covered" or ⚠ shortfall; one line: "What is left after bills and the safety cushion. Your AI agent can only use this." Source: module buckets at the snapshot block.
2. **Where it sits** (donut, half width): Cash · Reserved for bills · Earning (vault assets), as % with the total in the center. Source: Safe USDC balance, reserved = owed + buffer, vault positions.
3. **Cash runway** (ring gauge, half width): days = Safe cash ÷ average daily outflow (bill payments + owner sends) over the observed last 30 days. Status: Healthy (≥ 60 d) · Keep an eye on it (21–59) · Low (< 21) · **Early estimate** (under 7 days of data) · Not enough history (< 12 h).
4. **Total, last 30 days** (area line): current Safe cash, ▲/▼ % change, a smooth line with a glowing end dot. Series reconstructed from Safe USDC `Transfer` events and module checkpoints via a persisted, incremental event index.
5. **Weekly spend** (pill bars, half width): 3w · 2w · 1w · Now; bars stacked by vendor (top vendors + Other), legend with totals. Source: paid bills from the tx index.
6. **Interest per year** (thin bars, half width): Aave lending = live supply APY from Aave V3 Pool `getReserveData(USDC).currentLiquidityRate`; Demo vault = "simulated" (no live rate). Bars scale to the highest rate shown.
7. **Worth knowing**: 2–3 sentences computed from the numbers above (runway at current pace, owed vs funded, idle surplus vs vaults).
8. **AI Agent** tile: Active / Paused; last decision with its source chip (App / Agent) and planner (`mnn+ahem-g@android`); **Run Agent** (dry run: live policy preview, no broadcast) and **Stop**.
9. **Vault** tiles: Aave waUSDC (Aave logo, on-chain position, withdrawable, exposure cap, shares) and the demo vault (labelled simulated yield).

Freshness: cached snapshot paints instantly; "Updated N s ago · refreshing" until the live read lands (about 2 s).

## Bills — "Verified vendors and obligations protected on-chain"

- Live block chip + Refresh.
- Segmented **Bills | Vendors**.
- **Bill cards**: vendor, amount, due date, status Due / Queued (countdown) / Paid (BaseScan link). Bill details resolve from the on-chain commitment root (`evidence/obligations/by-root/<root>.json`).
- **Vendor cards**: name, short address, ACTIVE or COOLING (countdown ring), sanctions result, BaseScan.
- **Paid per vendor** bars.
- **Add vendor / commit bills**: explains this needs both Safe owners (2-of-2) and links to the Safe; no dead form.

## AI Agent — "It works inside rules you set"

- **How it decides** (expandable diagram): facts → candidates A–G + H (hold) → on-device model picks one letter under the AHEM-G grammar gate → policy preview → signer → contract.
- **Agent controls**: Dry run / **Run agent now** (on-device model on live chain state; live mode signs with the Keystore session key), **Stop** = guardian pause transaction from the app's `:signer` process.
- **Latest decision**: candidates with the chosen one highlighted, each policy check pass/fail, planner kind, load/decode ms, passport hash + BaseScan.
- **Decision mix** donut (Pay / Deposit / Withdraw / Hold) and **daily limit** ring (used / limit, remaining).
- Queue with live countdowns; decision history with source chips.

## Activity — "On-chain transaction & module timeline"

- Filters: All · Agent · Owner · App · Blocked.
- One card per transaction: human title ("Bill paid", "Bills committed by both owners", "Agent resumed by both owners", "Guardian pause"…), amount, vendor name, local time, **source chip** (App / Agent / Owner / Script), planner chip, BaseScan link. Tap → detail sheet (passport hash, block, gas).
- Provenance comes from `evidence/tx_index.json` (37 txs): built from evidence files plus on-chain proof (every `SafeMultiSigTransaction` from the treasury Safe = OWNER). Unknown transactions say "Source not recorded"; they never default to a guess.

## Safety — "Attacks blocked by contract and policy guards"

- **Score**: 23 / 24 attacks stopped · 0 attacks succeeded (frozen 0/30 · held-out 0/20), measured time, links to arena evidence and the held-out evaluation.
- **Stopped at each layer** (bars): AHEM-G → validator → policy → signer → contract, counts from `evidence/results/latest.json`.
- **ASR / FBR** bars: frozen vs held-out (0/30, 0/20; 9/30, 13/20).
- **Blocked attempts**: what was tried, which layer stopped it, evidence link (Base Sepolia revert tx or test name).
- **Known limits**: the escaped scenario (A12, x402 not built) stated plainly.
- **Stop** and **Emergency unwind** (unwind is an owner 2-of-2 action, explained and linked).

## Settings and Judge mode

- Network & deployment (addresses, Sourcify verification links), on-device model (hash, size, last latency, re-download/verify), keys (imported or not; remove), language, redo setup, about.
- **Judge mode**: every evidence row with its transaction link, plus the two verify commands (`bash scripts/judge/verify_all.sh` → 48/48 with no keys).
