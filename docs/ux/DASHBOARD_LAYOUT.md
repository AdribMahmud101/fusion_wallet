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

1. **Spare cash** (hero, green gradient): surplus bucket in USDC; ✓ "Your bills are covered" or ⚠ shortfall; one line: "What is left after bills and the safety cushion. Your AI agent can only use this." Source: module buckets at the snapshot block. The bottom action bar contains exactly **↓ Receive** and **+ Earn**. Receive opens an offline-generated QR of the Safe address, copy confirmation, the warning "Send USDC on Base Sepolia (test network) only", and a BaseScan address link. Earn runs `runAgentTick(dryRun = true)` and opens a plain-language decision sheet with the actual action amount, vault/vendor, snapshot block, policy checks, and any queue or owner-approval requirement. Bill coverage in the preview is computed from that decision's snapshot; no transaction is broadcast.
2. **Where it sits** (donut, half width): **Spare cash** (green surplus) · **For bills** (amber funded reserve + buffer; owed + buffer when fully funded) · **Earning** (sky vault assets). Each legend row includes a compact USDC amount and percentage, with total assets in the center. An unfunded shortfall is explained in ⓘ rather than counted as an asset. Source: module buckets and vault positions at the snapshot block. Footer chip: **Agent gas: X ETH (~N moves)**, from session-key ETH at that same block divided by median actual gas cost from up to 20 of its indexed receipts in the last 30 days (including recorded L1/operator fees). Receipt sender/hash, block height and canonical block hash are checked. Unknown history omits N; unavailable balance omits the chip.
3. **Cash runway** (ring gauge, half width): days = Safe cash ÷ average daily outflow (bill payments + owner sends) over the observed last 30 days. Status: Healthy (≥ 60 d) · Keep an eye on it (21–59) · Low (< 21) · **Early estimate** (under 7 days of data) · Not enough history (< 12 h).
4. **Total, last 30 days** (area line): current Safe cash, ▲/▼ % change, a smooth line with a glowing end dot. Series reconstructed from Safe USDC `Transfer` events and module checkpoints via a persisted, incremental event index.
5. **Weekly spend** (pill bars, half width): 3w · 2w · 1w · Now; bars stacked by vendor (top vendors + Other), legend with totals. Source: paid bills from the tx index.
6. **Interest per year** (thin bars, half width): Aave lending = live supply APY from Aave V3 Pool `getReserveData(USDC).currentLiquidityRate`; Demo vault = "simulated" (no live rate). Bars scale to the highest rate shown.
7. **Worth knowing**: 2–3 sentences computed from the numbers above (runway at current pace, owed vs funded, idle surplus vs vaults).
8. **AI Agent** tile: **Autopilot on · next in 3m** when enabled (live countdown from the foreground controller; deciding/waiting states replace the countdown), then Active / Paused; last decision with its recorded source chip (App / Agent); **On-device AI · Private**; **Preview next move** (opens the same dry-run decision sheet) and **Stop** (turns Autopilot off immediately, then opens the separate guardian-pause dialog). An allowance bar shows "Today's AI allowance: X of Y USDC left · up to Z per move", from snapshot daily usage, daily policy limit, and instant-tier maximum; overspent or zero limits clamp remaining to zero. The detailed planner (`mnn+ahem-g@android`) stays on the AI Agent tab.
9. **Vault** tiles: Aave waUSDC (on-chain assets) and the demo vault (labelled simulated yield). Each shows **Available anytime: X USDC** from `maxWithdraw`; shares, exposure cap, and vault address are in ⓘ. There are no Add/Withdraw buttons.

**Queue banner, immediately under the hero:** **Waiting for your approval** appears only when the live module queue contains an item (including an expired item that is still present on-chain). It shows the actual action, amount and vendor label when recorded, "Above the AI's auto-limit" only above the instant maximum, "Your bills stay covered" only when the buckets support it, and a second-by-second countdown to `readyAt` (or ready/expired status). **Approve in Safe (2 of 2)** opens an explanatory sheet with the action digest and Safe link; opening it does not submit an approval.

**K30 cancellation dependency:** the current `GuardianSignerClient` exposes only `requestPause`; its `:signer` service accepts only empty PAUSE IPC. There is no public guardian queue-cancellation API. In this build the banner states that cancellation is unavailable and offers guardian-import instructions when absent. A transaction-producing Cancel button is deliberately omitted until the signer owner supplies a digest-bound cancellation API or scope permission allows that API to be added. Pause is never presented as queue cancellation.

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
- **Autopilot card, first on screen**: toggle, **1m / 5m / 15m** interval (default 5m), foreground status, next-tick countdown, last decision, **Run now**, and **Stop**. Enabling runs immediately; the next tick is scheduled after the previous one completes. Run now also works with periodic ticks off. Stop turns scheduling off immediately and cancels the active coroutine. A submitted transaction may still confirm.
- **Execution mode**: **Preview only on this device: no session key** when no key is imported; **Preview only on this device: broadcast lease unavailable** without a matching live relay lease. With both available, ticks call the existing runtime in live mode and use its signer/receipt verification. Every tick refreshes guards; paused/unconfigured modules, expired/inactive sessions, read-only tier or role, guardian stop, offline/stale snapshots and unverified bill accounting record **H · Hold** with reasons and checks. No policy or signer bypass is added.
- **Autopilot timeline**: latest 50 actual ticks, newest first, persisted in app-private storage with atomic replacement. Each row shows local time, decision letter and plain title, reason, passed/failed checks, preview/hold/execution status, and a BaseScan link only for a verified execution hash. Interrupted live attempts say **Execution not confirmed**. Journal failure turns automatic execution off and shows an error.
- **Foreground lifecycle**: an in-process coroutine is attached to the root Activity lifecycle. Leaving the foreground cancels it; returning runs one tick without replaying missed intervals. It needs no background service, foreground-service policy exception, alarm or wake lock. Signing out disables scheduling; auditor access holds. Guardian pause disables Autopilot before the signer request.
- **Manual agent controls**: Dry run / **Run agent now** (on-device model on live chain state; live mode signs with the Keystore session key). **Guardian pause** remains the separate on-chain transaction from the app's `:signer` process.
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
