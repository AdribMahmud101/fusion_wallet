# Fusion Wallet — UX plan v1 (judge-facing redesign)

Source: supervisor review of the app on the demo phone (Tecno CL6, 1080×2436) plus ui-ux-pro-max guidance (pattern "Trust & Authority": security first, accent only for actions; onboarding must allow Skip/Back; confirm destructive actions and confirm success; charts never rely on color alone; Compose: typed routes, ViewModel-driven navigation events).
Rule zero: **every component is functional and fed by real data (chain, evidence files, device). No dummy UI.** If data is unavailable, show an honest empty/unknown state that says what is missing.

## 1. Design system

**Style:** dark obsidian bento grid — rounded tiles (20 dp radius, 1 dp border), one idea per tile, generous spacing (16 dp gutters, 20 dp tile padding), numbers big, labels small.

| Token | Hex | Use |
|---|---|---|
| bg | `#0B0F14` | screen background |
| surface | `#141B22` | tiles |
| surface-raised | `#1A232C` | dialogs, pressed tiles |
| border | `#26313B` | tile stroke |
| text | `#F3F1EA` | primary text |
| text-muted | `#8C9A94` | labels, captions (≥ 4.5:1 on surface) |
| green | `#19C37D` | positive, primary action, "Active" |
| indigo | `#7C83F7` | agent / information |
| amber | `#E8A33D` | queued, cooling, warnings |
| red | `#E5484D` | Stop, destructive, failures |

Status is never color-only: every status chip has text ("Active", "Cooling · 42 min", "Queued · 2:31").
**Type:** Space Grotesk (headings), IBM Plex Sans (body), JetBrains Mono (amounts, hashes, addresses). Scale: 32 / 24 / 18 / 15 / 13 sp. Amounts in lists `2,153.00 USDC`; exact 6-dp value on the detail view.
**Icons:** UI icons = Material Symbols Rounded (one family, outlined, 24 dp). Crypto/brand marks as vector drawables from official sources: USDC + ETH (spothq `cryptocurrency-icons`, CC0), Base, Aave, Safe (each project's brand kit). Record source + license in `res/drawable/ICONS.md`. No emoji anywhere.
**Motion:** 150–250 ms ease-out; respect system "remove animations".
**Touch:** targets ≥ 48 dp; every tap gives feedback; destructive actions confirm, then show a success snackbar with the tx link.

## 2. Information architecture

Bottom nav (5): **Home · Bills · Agent · Activity · Safety**. Avatar (top right) opens **Settings**. Judge mode lives in Settings and as a shortcut tile on Safety.

## 3. First-run onboarding (stepper, Skip/Back on every step, progress "Step 2 of 6")

1. **Welcome & role** — Owner · Finance approver · Auditor (view-only, for judges). One sentence on what each can do.
2. **Network & currency** — network: Base Sepolia (TEST badge). Treasury currency read from the deployment manifest (Aave test USDC, with USDC icon); if the manifest offers a fallback profile (Circle USDC) show it as an option, else a locked chip with "set by deployment". Language: English / 中文.
3. **Connect treasury** — load `deployments/84532.json`; live checks with ticks: Safe has code · threshold 2 of 2 · KavachiModule enabled · contracts source-verified (Sourcify link). Each check shows pass/fail and the block number.
4. **On-device AI model** — LFM2.5-230M (MNN 4-bit, ~128 MB): storage check, Wi-Fi-only toggle, download with progress and resume (Hugging Face `taobao-mnn/LFM2.5-230M-MNN` or the project's release asset), SHA-256 verified against the pinned hash, then a warm-up self-test decision (prints chosen letter + load/decode ms, AHEM-G ON). "Skip — monitor only" allowed.
5. **Keys** — what each key can do (authority table: Owner / Approver = Safe 2-of-2, Agent session key = module only, Guardian = Stop only). Import guardian and/or session key into Android Keystore (`:signer`), or "View-only".
6. **Policy review** — read-only from chain: tiers (instant / queued / needs approval), daily + session limits, reserve buffer, verified vendors. "Changes need both owners (Safe 2-of-2)." → Finish → Home.

Settings later lets you redo any step (model re-download, key import, language).

## 4. Screens

**Home (bento)**
- Hero tile: total treasury (USDC icon), network badge, "Updated 12 s ago", live/cached state.
- Buckets tile: 100 % stacked bar Owed · Buffer · Surplus with labels + values.
- Balance-over-time tile: area chart from on-chain history (Transfer / module events), 7-day and 30-day toggle; accessible summary line ("+12 % this week").
- Agent tile: status (Active / Paused), last decision in one line, "Run agent" (in-app planner) + Stop.
- Vault tiles: Aave waUSDC (Aave logo, live balance) · Demo vault (labelled "simulated yield").
- Next bill tile: vendor, amount, due in N days, tier.

**Bills** — segmented control *Bills | Vendors*.
- Bill card: vendor name + avatar initial, amount, due date, status (Due / Queued with countdown / Paid with tx link).
- Vendor card: name, short address, status ACTIVE or COOLING with a countdown ring, sanctions check result, BaseScan.
- "Add vendor / commit bills" explains it is an owner 2-of-2 action and links to the Safe — no dead form.

**Agent**
- "How it decides" strip (collapsible): Facts → candidates A–G + H → on-device model picks a letter (AHEM-G) → policy preview → signer → contract.
- Latest decision card (expanded): candidate list with the chosen one highlighted, each policy check pass/fail, planner kind, latency, passport hash (copy) + BaseScan.
- Controls: Run agent now (dry run / live), Stop (guardian).
- Daily limit ring gauge (used / limit) and queue with countdowns.
- Decision history list (human titles, provenance chip Agent / Owner / App / Script).

**Activity (new)** — one timeline of every on-chain action with filters (All / Agent / Owner / App / Blocked), each row: human title, amount, counterparty, time, provenance chip, tx link. Tap → detail sheet with passport, block, gas.

**Safety**
- Score tile: "21 / 24 attacks stopped" + "0 attacks succeeded (frozen 0/30, held-out 0/20)".
- Five-layer diagram (AHEM-G → validator → policy → signer → contract) with counts stopped at each layer.
- Blocked attacks list: what was tried, which layer stopped it, evidence link (Base Sepolia revert tx or test name).
- Known limits (escaped scenarios) listed honestly.
- Stop + Emergency unwind (unwind = owner action: explain and link if not signable here).
- Judge mode shortcut.

**Settings** — Network & deployment (addresses, verification links) · On-device model (hash, size, last latency, re-download) · Keys (which are imported, remove) · Language · Judge mode (every evidence row, verify commands) · About.

## 5. Performance (from K21)
Cached state paints instantly; live snapshot ≤ 3 s; history and GitHub evidence load in the background; skeleton tiles keep layout stable (no jumping).
