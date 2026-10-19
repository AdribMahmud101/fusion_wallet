# Card K23 — Design system + Home bento + Activity tab + charts + crypto icons

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). Innovation & interaction = 30%. Implement the UX plan `docs/ux/UX_PLAN.md` (READ IT FULLY FIRST) §1 design system, §2 navigation, §4 Home and Activity. **Every component functional, fed by real data; no dummy UI; honest empty states.**

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K23` (branch `card/K23`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside it `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K23 FETCH_HEAD` (includes K17 in-app agent + K21 fast paint/UX). Commit inside the submodule.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh`; `./gradlew testDebugUnitTest assembleDebug` in `fusion-core-android` (shared Gradle daemon; do not start extra daemons).
- Data: `data/` (WalletState, cached snapshot, history), evidence files on GitHub raw (already used by the app).

## Owned paths (inside the submodule)
`ui/theme/**`, `ui/components/**`, `ui/charts/**`, `ui/shell/**` (nav, top bar), `ui/home/**` (or the Treasury/Home screen package), NEW `ui/activity/**`, `res/drawable/**` (icons), `res/font/**`, `res/values*/strings*.xml` entries you add. Do NOT touch `ui/agent/**`, `ui/safety/**`, `ui/bills/**`, `ui/onboarding/**`, `ui/settings/**`, `inference/**`, `agent/**`, `signer/**` (other cards).

## FIRST (within 30 min, commit alone): shared design tokens + icons
- `ui/theme/FusionTokens.kt`: `object FusionColors { val Bg = Color(0xFF0B0F14); val Surface = Color(0xFF141B22); val SurfaceRaised = Color(0xFF1A232C); val Border = Color(0xFF26313B); val Text = Color(0xFFF3F1EA); val TextMuted = Color(0xFF8C9A94); val Green = Color(0xFF19C37D); val Indigo = Color(0xFF7C83F7); val Amber = Color(0xFFE8A33D); val Red = Color(0xFFE5484D) }`, `object FusionType` (Space Grotesk headings, IBM Plex Sans body, JetBrains Mono numbers; scale 32/24/18/15/13 sp; bundle the fonts as `res/font` — all three are OFL), `object FusionShape { tile = RoundedCornerShape(20.dp) }`, spacing 16/20 dp.
- Components: `BentoTile`, `StatusChip(text, tone)` (always text + color), `AmountText(raw: BigInteger, decimals=6, exact: Boolean)` (`2,153.00 USDC` / exact 6-dp), `ProvenanceChip(Agent|Owner|App|Script)`, `TxLink(hash)`, `SkeletonTile`.
- Icons: vector drawables for USDC, ETH (spothq `cryptocurrency-icons`, CC0), Base, Aave, Safe (official brand kits); Material Symbols Rounded for UI. `res/drawable/ICONS.md` lists source URL + license per icon. No emoji.
Other cards (K22, K24) merge this commit and build on it — keep names stable after it lands.

## Then
1. **Navigation:** bottom nav Home · Bills · Agent · Activity · Safety (≤ 5); avatar → Settings route (K22 provides the screen; add the route + a placeholder-free entry that opens it — if K22's screen is not merged yet, the avatar opens the existing profile/settings screen). Typed routes; ViewModel-driven navigation events.
2. **Home bento** per plan §4: hero treasury tile (USDC icon, network badge, "Updated N s ago", cached/live), 100 % stacked bucket bar with labels+values, **balance-over-time area chart** from on-chain history (7 d / 30 d toggle + one-line accessible summary; if fewer than 4 points, show a stat tile instead), agent tile (status, last decision one line, Run agent + Stop using existing ViewModel actions), vault tiles (Aave waUSDC with Aave logo; demo vault labelled "simulated yield"), next-bill tile.
3. **Activity tab (new):** one timeline of every on-chain action (module events + evidence rows), filters All / Agent / Owner / App / Blocked, rows with human title, amount, counterparty name, time, ProvenanceChip, TxLink; tap → bottom sheet with passport hash, block, gas.
4. Charts implemented with Compose Canvas (no web views); accessible content descriptions; series distinguished by label/pattern, not hue alone.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K23/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && test -s app/src/main/res/drawable/ICONS.md
```
exits 0, plus `ui/K23_SCREENS.md` listing each tile/screen → its data source.

## Acceptance
- No hard-coded balances/hashes/counts in release paths. No Float/Double money. No test weakened. Commit inside the submodule on `card/K23`. Do not merge, do not push.

## Supervisor device finding (after K21 merge)
On the Tecno (1080 px) the Home title row breaks: the "Updated 2s ago · refreshing" chip sits in the same row as the screen title, so "Home" wraps to "Hom/e" and the subtitle is squeezed into a narrow column. Put freshness status on its own line under the subtitle (or in the hero tile), never in the title row. Check every screen title row the same way.
