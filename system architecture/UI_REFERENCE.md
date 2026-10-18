# Fusion Wallet — Android UI/UX Design System & Architectural Reference

> **Living Technical Specification & Style Guide**  
> **Platform:** Android 10+ (API 29+) · **Toolkit:** Jetpack Compose & Material 3 (`androidx.compose.material3`)  
> **Aesthetic:** "Obsidian Institutional" with Bento Grid Layout & Progressive Disclosure  
> **Code Location:** `repos/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/ui/`

---

## 1. Executive Summary & Design Philosophy

**Fusion Wallet** is an autonomous AI treasury assistant built for small-and-medium enterprises (SMEs) running on Base Sepolia. The user interface bridges institutional fintech trust with autonomous on-device edge AI.

The UI is anchored by three overarching design tenets:

1. **Obsidian Institutional Aesthetic**  
   A high-density, flat, dark-canvas foundation (`#08090C`). Zero drop shadows, pure flat surfaces, hairline borders (`1.dp`), and restrained semantic accents deliver a serious financial-grade tool rather than a speculative consumer crypto app.

2. **Progressive Disclosure (`ui-ux-pro-max` Paradigm)**  
   *“Headline → Status → Single Action → Details on tap.”*  
   Financial operators need immediate answers in seconds, not cognitive overload:
   - **One dominant question answered per screen** (e.g., *“How much can the agent safely deploy right now?”*).
   - Plain-language terminology over crypto jargon (*“Spare cash”*, *“Set aside for bills”*, *“Earning interest”*).
   - Technical depths (exact calldata, cryptographical digests, verification layers, gas limits, raw hashes) are collapsed behind accessible `Expander` taps.

3. **Radical Honesty & Safety Badging**  
   - **No fake greens:** Unmeasured metrics display `not measured` (`--`), never a flattering `0%` or artificial `100%`.
   - **Explicit provenance:** Every screen highlights whether data is live on-chain or a `DEMO` synthetic fixture, paired with a permanent `TESTNET` badge in the top bar.
   - **Multi-layer failure attribution:** When an action is held or rejected, the UI explicitly shows which layer stopped it (*Edge Model*, *Deterministic Policy*, *Companion Signer*, or *On-Chain Contract*).

---

## 2. Directory Structure & File Map

All UI code resides in `repos/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/ui/`:

```
ui/
├── FusionWalletApp.kt        # Root shell: Session routing, Scaffold, TopBar, BottomBar, Crossfade
├── MainViewModel.kt          # Legacy state flow coordinator
├── Fmt.kt                    # Fixed US-locale financial & cryptographic formatters
├── DemoData.kt               # Synthetic preview fixtures
│
├── theme/                    # Design tokens & Material 3 setup
│   ├── Color.kt              # Obsidian & Accessible tokens, FusionSurfaces, FusionDims
│   ├── Theme.kt              # DarkColorScheme mapping & FusionWalletTheme wrapper
│   └── Type.kt               # Inter & JetBrains Mono typography scale
│
├── common/                   # Reusable Bento & Layout primitives
│   ├── Bento.kt              # BentoTile, BentoRow, BentoGap (10.dp), StatTile, TileLabel
│   └── Common.kt             # Tone object, ScreenHeader, CardTitle, Expander, Notice
│
├── charts/                   # Hand-drawn native Canvas charts (zero external libs)
│   └── Charts.kt             # DonutChart, RingGauge, Sparkline, StackedBars, LabeledBar, SegmentBar
│
├── components/               # Foundational UI building blocks
│   ├── Components.kt         # ObsidianCard, StatusPill, RuleRow, VelocityMeter, Monograms, Icons
│   ├── TopHealthAppBar.kt    # Legacy health bar & FusionBottomBar navigation
│   └── FusionTopBar.kt       # Slim institutional top bar (Brand, TESTNET, DEMO, Avatar)
│
├── treasury/                 # Tab 1: "What You Have" (Home)
│   ├── TreasuryScreen.kt     # Bento layout: Hero, Allocation, Runway, Trend, Spend, Bills
│   └── AskAccountantSheet.kt # Modal sheet for natural-language local/cloud analysis
│
├── agent/                    # Tab 2: "What It Does" (AI Agent)
│   └── AgentScreen.kt        # Mandate passport, daily limit gauge, decision timeline
│
├── proof/                    # Tab 3: "The Safety Evidence" (Safety)
│   ├── ProofScreen.kt        # Headline, Scorecard (ASR/FBR), 7-Scenario Arena grid, Receipts
│   └── PrivacySection.kt     # Cloud LLM egress toggle, Keystore key status, consent modal
│
├── auth/                     # Authentication & Profile management
│   ├── LoginScreen.kt        # Salted PBKDF2 sign-in, lockout timer, quick-fill demo cards
│   ├── ProfileSheet.kt       # User info, role capabilities, avatar customize, language switch
│   ├── AvatarView.kt         # 6-color × 7-glyph WCAG AA accessible avatar generator
│   └── AuthViewModel.kt      # Authentication state machine & lockout handler
│
├── chat/                     # Guardrail pre-flight inspection
│   ├── PreFlightGuardrailCard.kt # Decoded action card, parameter check, tier approvals
│   ├── ChatFeed.kt           # Legacy intent execution feed
│   └── PromptComposer.kt     # Input bar with MNN execution pill
│
├── i18n/                     # Internationalization
│   └── AppLocale.kt          # Seamless English / 中文 runtime configuration wrapper
│
└── splash/                   # Cold start splash
    └── FusionSplashScreen.kt # Animated brand splash with load-time floor
```

---

## 3. Color System & Accessibility Tokens

The color architecture is split into the baseline **Obsidian** design specification and the measured **Accessible** tokens that ensure strict **WCAG 2.1 AA** compliance against dark canvas surfaces.

### 3.1 Surface & Border Tokens

```kotlin
// Main Brand Palette (Obsidian)
val CanvasBackground = Color(0xFF08090C) // Main screen canvas (Deep Obsidian)
val SurfaceLevel1    = Color(0xFF101217) // Primary cards, bento tiles, dialogs
val SurfaceLevel2    = Color(0xFF181B22) // Input fields, chip backgrounds, nested containers
val SurfaceLevel3    = Color(0xFF232733) // Track backgrounds, hover states, secondary borders

val BorderSubtle     = Color(0xFF232733) // 1.dp standard container outline
```

### 3.2 WCAG AA Measured Contrast Substitutions

| Token | Spec Hex | Accessible Hex | Measured Ratio | WCAG Compliance Justification |
|---|---|---|---|---|
| **TextMuted** | `#6B7280` | `Color(0xFF7A8194)` | 4.81:1 on Surface1<br>5.12:1 on Canvas | Original `#6B7280` was 3.88:1 (failed AA 4.5:1). Replacement passes AA body text. |
| **ButtonFill** | `#6366F1` | `Color(0xFF5F62EE)` | 4.70:1 with White | Spec `#6366F1` was 4.47:1 (missed AA by 0.03). Replacement clears 4.5:1. |
| **BorderFocused**| `#374151` | `Color(0xFF5A6478)` | 3.15:1 on Surface1 | Spec was 1.82:1. Clears WCAG 1.4.11 (3.0:1 non-text contrast for focus rings). |
| **AccentText** | `#6366F1` | `Color(0xFF5A5DE8)` | 3.72:1 on Surface1 | Clears AA-large text threshold (3.0:1). |

### 3.3 CompositionLocal Surface Elevation

Material 3's `ColorScheme` only allows a single surface and surface variant. Fusion Wallet injects a multi-tiered surface hierarchy via `LocalFusionSurfaces`:

```kotlin
data class FusionSurfaces(
    val canvas: Color = Obsidian.CanvasBackground,
    val level1: Color = Obsidian.SurfaceLevel1,
    val level2: Color = Obsidian.SurfaceLevel2,
    val level3: Color = Obsidian.SurfaceLevel3,
    val border: Color = Obsidian.BorderSubtle,
    val borderFocused: Color = Accessible.BorderFocused,
)

val LocalFusionSurfaces = staticCompositionLocalOf { FusionSurfaces() }

// Convenient accessor in Composables:
val surfaces = FusionTheme.surfaces
```

### 3.4 Semantic Tone System (`Tone`)

Meaning-bearing colors are strictly mapped. **Rule:** Color must never be the only indicator; always pair with an icon, badge, or plain text label.

```kotlin
object Tone {
    val Cash    = Color(0xFF10B981) // Emerald: Free, unallocated, sweepable cash
    val Bills   = Color(0xFFF59E0B) // Amber: Committed operating reserves / obligations
    val Earning = Color(0xFF38BDF8) // Sky Blue: Deployed in vaults/Aave, yield-bearing, info
    val Good    = Color(0xFF10B981) // StatusSuccess: All checks passed, armed, healthy
    val Warn    = Color(0xFFF59E0B) // StatusWarning: Caution, in-queue, approaching cap
    val Bad     = Color(0xFFEF4444) // StatusError: Blocked, reverted, shortfall, critical
}
```

---

## 4. Typography Scale & Font Rules

Typography enforces a strict cognitive boundary: **Inter** for human UI copy; **JetBrains Mono** for all technical and engine data.

```
                    ┌────────────────────────────────────────────────────────┐
                    │                      TYPOGRAPHY                        │
                    ├───────────────────────────┬────────────────────────────┤
                    │   Inter (Variable)        │  JetBrains Mono (Variable) │
                    │   - UI Headings & Copy    │  - Hashes & Addresses      │
                    │   - Labels & Explanations │  - Financial Amounts       │
                    │   - Dialogs & Sheets      │  - FFI / Terminal Logs     │
                    └───────────────────────────┴────────────────────────────┘
```

### 4.1 Type Scale Definition

| Token / Style | Family | Weight | Size / LineHeight | Target Usage |
|---|---|---|---|---|
| `displaySmall` | Inter | SemiBold (600) | 30sp / 36sp | Page heroes, large sign-in headers |
| `headlineMedium` | Inter | SemiBold (600) | 22sp / 28sp | Screen titles, profile name headers |
| `titleLarge` | Inter | SemiBold (600) | 16sp / 22sp | Section titles, modal sheet titles |
| `titleMedium` | Inter | Medium (500) | 14sp / 20sp | Bento card titles, subsection headers |
| `bodyLarge` | Inter | Normal (400) | 14sp / 20sp | Primary body text, explanatory subtitles |
| `bodyMedium` | Inter | Normal (400) | 14sp / 20sp | Standard descriptions, list items |
| `bodySmall` | Inter | Normal (400) | 12sp / 16sp | Secondary notes, tile captions, hints |
| `labelLarge` | Inter | Medium (500) | 12sp / 16sp | Toggle button text, action chips |
| `labelMedium` | Inter | Medium (500) | 11sp / 16sp | Chart axis text, bar subgroup labels |
| `labelSmall` | JetBrains Mono | Medium (500) | 11sp / 16sp | Status tags (`PASS`, `WARN`), micro-badges |
| `MonoStyle` | JetBrains Mono | Normal (400) | 12sp / 16sp | Currency numbers, nonces, addresses |
| `MonoTerminalStyle`| JetBrains Mono | Normal (400) | 11sp / 16sp | FFI system logs, IPC json stream |

### 4.2 Tabular Figures Rule

All numbers and monospaced text employ **tabular figures** (`fontFeatureSettings = "tnum"`). This prevents horizontal jitter and UI shaking when countdown timers tick or live balances stream in:

```kotlin
private val TabularNumbers = TextStyle(
    fontFeatureSettings = "tnum",
    platformStyle = PlatformTextStyle(includeFontPadding = false),
)
```

---

## 5. Spatial Design & Bento Grid Architecture

The entire visual layout follows a **Bento Grid** architecture. Information is organized into compartmentalized, rounded tiles that create a calm, unified rhythm across heterogeneous data (gauges, numbers, lists, charts).

```
+-------------------------------------------------------------------+
| ScreenHeader: Title + Subtitle                   [Profile Avatar] |
+-------------------------------------------------------------------+
| BentoTile (Hero - 100% Width)                                     |
| "Spare cash" $3,000.00 USDC                     [Covered Status]  |
+---------------------------------+---------------------------------+
| BentoTile (50% Width)           | BentoTile (50% Width)           |
| Allocation Donut Chart          | Runway Gauge (4.2 Months)       |
+---------------------------------+---------------------------------+
| BentoTile (100% Width)                                            |
| 30-Day Asset Trend Sparkline                                      |
+-----------------------------------+-------------------------------+
| BentoTile (55% Width)             | BentoTile (45% Width)         |
| Weekly Spend Stacked Bars         | Vault Yield Labeled Bars      |
+-----------------------------------+-------------------------------+
```

### 5.1 Grid Tokens & Dimensions (`FusionDims`)

```kotlin
object FusionDims {
    val radiusSmall  = 6.dp   // Tags, chips, status pills
    val radiusMedium = 12.dp  // Legacy cards, text input outlines
    val radiusLarge  = 16.dp  // Bottom sheets, system dialogs
    val hairline     = 1.dp   // Container borders
    val gutter       = 16.dp  // Outer screen horizontal padding
}

val BentoGap  = 10.dp                   // Uniform spacing between all bento tiles
val TileShape = RoundedCornerShape(22.dp) // Bento tile corner curvature
```

### 5.2 Bento Primitives (`ui/common/Bento.kt`)

1. **`BentoTile`**  
   The atomic unit of the interface. Provides `surfaces.level1` background, 22.dp corner radius clipping, 1.dp subtle border, and an optional **vertical gradient tint** fading from 16% alpha to transparent. This allows a tile to carry emotional valence (success/good or warning/bad) without harsh visual weight.
   
2. **`BentoRow`**  
   An intrinsic-height row container (`Modifier.height(IntrinsicSize.Min)`). By applying `Modifier.weight(w).fillMaxHeight()` to each child tile, cards placed side-by-side are mathematically guaranteed to match heights regardless of internal content differences.

3. **`StatTile`**  
   Pre-composed high-impact KPI tile displaying a large `22.sp` `MonoStyle` value anchored over a subtle, muted caption.

---

## 6. Bespoke Canvas Visualization System (`Charts.kt`)

To maintain ultra-low runtime binary size (APK ~17 MB) and avoid third-party library bloat or licensing entanglements, **all charts are drawn directly using Compose `Canvas`**.

### 6.1 Universal Charting Rules

1. **TalkBack Spoken Accessibility:** Every chart container requires an explicit semantic `contentDescription` conveying the data in full sentences. The drawing canvas clears internal semantics.
2. **Redundant Numeric Text:** Charts never convey information through color alone; every chart is paired with textual numbers adjacent to it.
3. **Reduced Motion Adaptation:** Draw-in animations check `Settings.Global.ANIMATOR_DURATION_SCALE`. If the user has disabled animations in Android system settings, the chart immediately renders in its terminal state (`progress = 1.0f`).

### 6.2 Chart Catalog

```
   DonutChart               RingGauge               Sparkline
    ╭──────╮                 ╭──────╮              ╭───────────●
   │   ●    │               │  85%   │            ╱             │
    ╰──────╯                 ╰──────╯            ╱              │
Proportional buckets     Daily limit meter    30-day balance curve

   StackedBars              LabeledBar              SegmentBar
   █   █   █            Aave [██████░░] 0.21%    [█████░░░░░░░░░░]
   █   █   █            Demo [████░░░░] 3.10%    Bill timing timeline
 Weekly spend categories  Horizontal progress      Due dates distribution
```

| Composable | Description | Key Parameters | Where Used |
|---|---|---|---|
| **`DonutChart`** | Proportional ring with center composable slot. Zero totals render a hollow track. | `values: List<Float>`, `colors: List<Color>`, `size: Dp`, `stroke: Dp` | Home Allocation Tile (Cash vs Bills vs Earning) |
| **`RingGauge`** | Single 0..1 value arc with rounded stroke caps. | `progress: Float`, `color: Color`, `size: Dp`, `stroke: Dp` | Home Runway Tile, Agent Daily Limit Tile |
| **`Sparkline`** | Smooth cubic Bezier spline with vertical gradient area fill and endpoint indicator dot. | `values: List<Float>`, `color: Color`, `description: String` | Home 30-Day Asset Trend Tile |
| **`StackedBars`** | Multi-series vertical stacked bars with rounded tops and bottom category text labels. | `groups: List<List<Float>>`, `colors: List<Color>`, `labels: List<String>` | Home Weekly Spend Breakdown (Payroll, Suppliers, Other) |
| **`LabeledBar`** | Horizontal bar with left label, right formatted value, and background level3 track. | `label: String`, `valueText: String`, `fraction: Float`, `color: Color` | Home Earning Rates (Aave waUSDC vs Demo Vault) |
| **`SegmentBar`** | Single horizontal segmented progress bar for continuous linear distributions. | `values: List<Float>`, `colors: List<Color>`, `description: String` | Home Obligations & Bills Due-Date Timeline |

---

## 7. Navigation & Screen Specifications

The app navigation was restructured from a legacy developer drawer into **three dedicated bottom navigation tabs** representing the operator's mental model:

```
                    ┌────────────────────────────────────────────────────────┐
                    │                   FUSION WALLET APP                    │
                    └────────────────────────────────────────────────────────┘
                                                 │
                   ┌─────────────────────────────┼────────────────────────────┐
                   ▼                             ▼                            ▼
         ┌───────────────────┐         ┌───────────────────┐        ┌───────────────────┐
         │     TREASURY      │         │     AI AGENT      │        │      SAFETY       │
         │      (Home)       │         │    (Execution)    │        │      (Proof)      │
         ├───────────────────┤         ├───────────────────┤        ├───────────────────┤
         │ • Safe to Deploy  │         │ • Mandate Details │        │ • Overall Shield  │
         │ • Asset Donut     │         │ • Velocity Ring   │        │ • ASR/FBR Scores  │
         │ • Runway Gauge    │         │ • Done/Wait/Block │        │ • 7 Attack Tiles  │
         │ • 30-Day Trend    │         │ • Rules Breakdown │        │ • Egress Guard    │
         │ • Spend Breakdown │         │ • Decision Feed   │        │ • Audit Receipts  │
         │ • Ask Accountant  │         │ • Layer Checklists│        │ • Consent Dialog  │
         └───────────────────┘         └───────────────────┘        └───────────────────┘
```

### 7.1 Screen 1: Home (`TreasuryScreen.kt`)
*Answering: "How much capital do I have, and what is safe for the agent to touch?"*

- **Hero Tile:** Displays **"Spare Cash"** (`b.sweepable`). If there is a funding shortfall (`b.shortfall > 0`), the tile tints `Tone.Bad` and alerts: *"Short by X USDC. Nothing will be invested."*
- **Allocation & Runway BentoRow:**
  - *Left:* `DonutChart` breaking down Cash (`#10B981`), Reserved Bills (`#F59E0B`), and Deployed (`#38BDF8`).
  - *Right:* `RingGauge` showing operating cash runway in months/days based on average 30-day burn rate.
- **Trend Tile:** 30-day historical net-worth `Sparkline`.
- **Spending & Rates BentoRow:**
  - *SpendTile:* `StackedBars` showing 7-day outflows (Payroll, Suppliers, General).
  - *RatesTile:* `LabeledBar` comparing real Base Sepolia Aave supply APR (0.21%) vs OZ Demo Vault APR.
- **Ask Accountant Tile:** Banner tapping into `AskAccountantSheet` to run on-device deterministic analytics or cloud LLM inquiries.
- **Holdings & Bills:** Full list of vault token balances and upcoming payment liabilities.

### 7.2 Screen 2: AI Agent (`AgentScreen.kt`)
*Answering: "What is the agent authorized to do, and what decisions has it made?"*

- **Identity & Limit BentoRow:**
  - *IdentityTile:* Edge AI passport (Model: `LFM2.5-230M-MNN`, authority scope, status: `ARMED`, `PAUSED`, or `HALTED`).
  - *LimitTile:* `RingGauge` visualizing current 24-hour spent vs. daily cap ($2,500 USDC limit).
- **Stats Row:** Compact `StatTile` trio: `Done` (green), `Waiting` (amber), `Blocked` (muted).
- **Rules Tile:** Mandate parameters: allowed destination whitelist, max transaction size ($100), required slippage bounds.
- **Decision Timeline:** Chronological feed of candidate actions picked by the model. Each card includes:
  - Timestamp & target recipient.
  - Action summary (*"Deploy 50.00 USDC to Vault A"*).
  - Multi-layer check breakdown:
    - 🛡️ **Policy Engine:** Whitelist check, balance sufficiency.
    - 🤖 **Edge Inference:** Candidate ID validity, prompt evaluation.
    - ✍️ **Signer Companion:** Digest agreement, owner consent.
    - ⛓️ **On-Chain Contract:** Rate-limit check, balance reservation.

### 7.3 Screen 3: Safety (`ProofScreen.kt`)
*Answering: "What is the concrete evidence that this system cannot be exploited?"*

- **Headline Banner:** Large shield glyph displaying total blocked attacks (*"7 of 7 attacks blocked"*).
- **Scorecard Row:**
  - **ASR (Attack Success Rate):** `0/7` (Good / Green).
  - **FBR (False Block Rate):** `--` (*"Not measured"* — honestly disclosed because benign test corpora are pending).
  - **Availability:** `7/7` (System liveness).
- **Attack Arena Grid (2 Columns):** Visual tiles for the 7 standard test attacks:
  1. *Prompt Injection:* Rebuffed at inference layer.
  2. *Depeg Attack (0.97):* Local oracle latch triggered before model execution.
  3. *Cap Breach:* Reverted by `HardLimitExceeded` on-chain.
  4. *Reserve Breach:* Reverted by `ReserveViolation` on-chain.
  5. *Drip Drain:* Blocked by cumulative daily limit tracking.
  6. *Queue Bypass:* Reverted by contract time-lock enforcement.
  7. *RPC Drop / Network Restart:* Reconciled safely via idempotent transaction journal.
- **Cloud Egress & Privacy Tile:** Displays status of DeepInfra cloud analyst. Fail-closed toggle; Keystore status; egress redaction counters.

---

## 8. Multi-Tier Action Guardrails & Decoded Sign Cards

The UI enforces a **three-tier security hierarchy** based on transaction value:

```
                    ┌────────────────────────────────────────────────────────┐
                    │               THREE AMOUNT ACTION TIERS                │
                    ├────────────────────────────────────────────────────────┤
                    │ Tier 1: Micro Action (≤ $10.00 USDC)                   │
                    │         • Immediate autonomous on-chain execution      │
                    │         • Zero delay; instant receipt logged           │
                    ├────────────────────────────────────────────────────────┤
                    │ Tier 2: Standard Action ($10.01 – $100.00 USDC)        │
                    │         • Autonomous execution behind 180s Time-Lock   │
                    │         • Live countdown ticker in UI                  │
                    │         • On-chain Cancel button active during window  │
                    ├────────────────────────────────────────────────────────┤
                    │ Tier 3: High-Value Action (> $100.00 USDC)             │
                    │         • Autonomous execution strictly FORBIDDEN      │
                    │         • Decoded confirmation card prompted           │
                    │         • Cryptographic Owner biometric/key signature  │
                    └────────────────────────────────────────────────────────┘
```

### 8.1 Decoded Action Card Spec (`PreFlightGuardrailCard.kt`)

Before any Tier 3 transaction or queue confirmation occurs, a **Decoded Intent Card** is generated. The UI adheres to strict anti-spoofing requirements:

- **Plain-English Narrative:** *"Move 50.00 USDC from Vault A to Vault B. Keep 2,000.00 USDC protected for bills."*
- **Exact Bounds:** Displays exact minimum shares received and maximum shares burned calculated with asset decimals (never loose floating-point math).
- **Full Address Inspection:** Addresses expand on tap to show full 42-character hexadecimal strings; abbreviations (`0x12...34`) alone are forbidden for signing consent.
- **Independent Signer Digest:** The Companion Signer app calculates and presents its own digest. If UI state and Signer state diverge by even 1 bit, execution is hard-blocked.

---

## 9. Authentication, RBAC & Profile Customization

The application features a built-in role-based access control (RBAC) model operating on three public evaluation profiles:

| Account | Role | Color Accent | Permissions & Capabilities |
|---|---|---|---|
| `mei@fusion.demo` | **Owner** | Indigo (`#4F52D6`) | Full control: Toggle cloud LLM, pause/resume agent, approve >$100 Tier 3 operations. |
| `wei@fusion.demo` | **Finance Manager** | Teal (`#0F766E`) | Operational control: Pause/resume agent, execute within pre-approved mandate. |
| `judge@fusion.demo` | **Auditor** | Amber (`#B45309`) | Read-only inspection: View treasury, decision logs, receipts, and attack arena. |

### 9.1 Authentication Implementation (`LoginScreen.kt`)
- Salted PBKDF2 credential validation.
- Brute-force protection: **5 failed attempts enforce a 30-second lockout timer** with live on-screen countdown.
- 1-tap quick fill buttons for demo reviewers.

### 9.2 Accessible Avatar System (`AvatarPalette`)
Avatars are dynamically computed using 6 accessible colors (ensuring white glyph contrast > 4.5:1) and 7 distinct icons:
- **Colors:** Indigo (`#4F52D6`), Teal (`#0F766E`), Amber (`#B45309`), Rose (`#BE123C`), Sky (`#0369A1`), Green (`#15803D`).
- **Glyphs:** Initials, Person, Briefcase, Shield, Star, Rocket, Leaf.

---

## 10. Internationalization (i18n) & Locale Switching

The app features full runtime localization between **English (`en`)** and **Simplified Chinese (`zh`)** without restarting the activity or resetting view states.

```kotlin
// AppLocale.kt runtime context wrapper
CompositionLocalProvider(
    LocalContext provides localizedContext,
    LocalConfiguration provides localizedContext.resources.configuration,
) {
    FusionWalletTheme {
        FusionWalletApp()
    }
}
```

### 10.1 Formatting Rules (`Fmt.kt`)
To eliminate ambiguity across languages:
- **Financial Currency:** Enforces fixed US decimal formatting (`#,##0.00`) via `DecimalFormatSymbols(Locale.US)` across all locales.
- **Short Numbers:** Compresses large figures for compact cards (e.g., `10000` $\rightarrow$ `10K`).
- **Hex Hashes:** Truncates safely using 8-character prefix and 4-character suffix (`0x8f3b…a109`).
- **Duration Format:** Translates seconds to human strings (`95s` $\rightarrow$ `1:35`, `3700s` $\rightarrow$ `1h 1m`).

---

## 11. Core Reusable Composable Snippets

### 11.1 Standard Bento Row with Balanced Tiles

```kotlin
BentoRow {
    // 50% width tile
    BentoTile(
        modifier = Modifier.weight(1f).fillMaxHeight(),
        tint = Tone.Good
    ) {
        TileLabel("ACTIVE STATUS")
        Text("Protected", style = MaterialTheme.typography.titleMedium)
    }

    // 50% width tile
    StatTile(
        modifier = Modifier.weight(1f).fillMaxHeight(),
        label = "RUNWAY",
        value = "4.2 mo",
        color = Tone.Cash
    )
}
```

### 11.2 Progressive Disclosure Expander

```kotlin
var expanded by rememberSaveable { mutableStateOf(false) }

Expander(
    label = "View Verification Checks",
    expanded = expanded,
    onToggle = { expanded = !expanded }
) {
    Column(Modifier.padding(top = 8.dp)) {
        RuleRow(label = "Whitelist Target", detail = "0xba50...4D5f", state = RuleState.Pass)
        RuleRow(label = "Daily Cap", detail = "$50.00 / $2,500.00", state = RuleState.Pass)
    }
}
```

### 11.3 Status Pill Badge

```kotlin
StatusPill(
    label = "TESTNET",
    tone = Tone.Warn,
    leading = {
        Icon(Icons.Filled.Shield, contentDescription = null, tint = Tone.Warn)
    }
)
```

---

## 12. Accessibility & System Integration Checklist

Before shipping any new Compose screen in `fusion_wallet`, verify against this checklist:

- [ ] **Contrast:** All body text meets WCAG AA 4.5:1 on its underlying surface level.
- [ ] **Touch Targets:** All interactive rows, buttons, and expanders have `Modifier.heightIn(min = 48.dp)` or `size(48.dp)`.
- [ ] **Semantic Roles:** Custom clickables declare `Modifier.semantics { role = Role.Button }` or `Role.RadioButton`.
- [ ] **No Color-Only State:** Errors or warnings pair color with an icon (`Warning`, `ErrorOutline`) and clear text labels.
- [ ] **System Insets:** Containers respect edge-to-edge display:
  - Top bars use `Modifier.statusBarsPadding()`.
  - Bottom navigation bars use `Modifier.navigationBarsPadding()`.
  - Input forms apply `Modifier.imePadding()`.
- [ ] **Hardware Acceleration:** Canvas charts skip infinite or entrance animations if system animator scale is 0.
- [ ] **String Extraction:** Zero hard-coded UI strings; all copy referenced from `R.string.*` with both EN and `values-zh` entries.
