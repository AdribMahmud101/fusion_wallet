# K29 — Legacy dashboard comparison and port

Inspected 2026-10-20. Archive: `/root/legacy/fusion_wallet_source.zip`, extracted
with `unzip -q /root/legacy/fusion_wallet_source.zip -d /root/legacy/src`.
Current baseline: submodule `89d1415257e547b147da922c625ca171da5d4f0c`
(K28 integration). Compared against both owner references:
[Home top](reference/home_top.png) and [Home bottom](reference/home_bottom.png).
Screenshots establish appearance, not timing; motion details below come from source.

## Source inventory

Legacy Android source root:
`/root/legacy/src/fusion_wallet/repos/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/`.
Kotlin paths in this inventory are relative to that root; the resource path is
relative to the legacy app's `src/main/` directory:

| File | Relevant implementation |
| --- | --- |
| `ui/treasury/TreasuryScreen.kt` | `TreasuryScreen`, `HeroTile`, `AllocTile`, `RunwayTile`, `TrendTile`, `SpendTile`, `RatesTile`, `AskTile`, `AttentionTile` (Worth knowing). Supplies snapshot/history values and labels to the Canvas charts. |
| `ui/charts/Charts.kt` | `rememberChartProgress`, `DonutChart`, `RingGauge`, `Sparkline`, `StackedBars`, `LabeledBar`, `SegmentBar`. The dashboard's hand-drawn chart and animation implementation. |
| `ui/common/Bento.kt` | `BentoTile`, `BentoRow`, `TileLabel`: 22 dp rounded tiles, quiet border, top-to-bottom tint gradient at 16% alpha, 10 dp bento gap. |
| `ui/common/Common.kt` | `Tone` chart colors and `Expander`; the latter uses Compose `AnimatedVisibility`. |
| `ui/theme/Color.kt`, `ui/theme/Theme.kt`, `ui/theme/Type.kt` | Obsidian surfaces, accessible text colors, typography and Material theme used by the tiles. |
| `res/values/strings.xml` | Dashboard titles, explanations and chart accessibility descriptions. |

Current source root is
`src/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/`.
The matching implementation is `ui/charts/ChartKit.kt`,
`ui/charts/BalanceAreaChart.kt`, `ui/charts/ChartMath.kt`,
`ui/charts/BucketSegments.kt`, `ui/charts/Charts.kt`,
`ui/charts/ChartTileHeader.kt`, `ui/treasury/HomeCharts.kt`,
`ui/treasury/TreasuryScreen.kt`, and `ui/theme/FusionTokens.kt`.
K29 adds the motion-only token file `ui/theme/ChartMotion.kt`.

## Side-by-side comparison

These are the same visual family: dark bordered bento tiles, green spare-cash
hero, cash/bills/earning donut, amber runway ring, sky area line, teal/coral
weekly bars, sky interest pills and dotted insights. They are not identical
implementations or identical token sets. K28 already retained the legacy
`Charts.kt` components, but Home primarily uses the newer typed `ChartKit.kt`
and `BalanceAreaChart.kt`. Restoring an entire legacy screen would discard the
current evidence contract and is unnecessary.

| Component | Legacy source: appearance and motion | K28 current baseline | K29 decision / resulting behavior |
| --- | --- | --- | --- |
| Shared entrance | `Animatable` 0→1, `tween(700, FastOutSlowInEasing)`. One entrance per composition. Reads Android animator scale to skip at zero; Compose scales playback. No spring, infinite pulse or number-counting animation. | Same helper, shortened to **200 ms**. Some callers start the animation while only an unavailable message is visible. | Restore **700 ms** with the same easing in `FusionChartMotion`. Key allocation, area, weekly and interest entrances by drawable-data availability so late evidence gets its entrance. Repeated refreshes with available data do not replay it. Values and accessibility text remain immediately accurate. |
| Allocation donut | 108 dp ring / 15 dp stroke; static empty track; positive segments sweep simultaneously clockwise from −90°. Butt caps; 3° gaps only with multiple positive buckets. Legacy forces a minimum tiny arc. | 122 dp / 17 dp; butt caps; 2° gaps even for a single bucket; no background track. Exact integer bucket boundaries and largest-remainder percentages are better than legacy floats. | Keep current dimensions, exact arithmetic and semantics. Port the track and 3° multi-bucket gaps; a single bucket closes completely. Shrink gaps for tiny buckets instead of adding fake visible value. Keep the current zero-total text fallback. |
| Cash runway | `RingGauge`: full 360° track and 360° × fraction, round progress caps. Arc grows with shared entrance. Legacy scales days to 60 and uses different health thresholds. | Full track, but **270°** × fraction; 90-day scale and evidence-based observed outflow, with explicit minimum history. | Port the full **360°** sweep and retain round caps. Keep the current 90-day denominator, uncapped numeric label, thresholds, and text-only insufficient-history states. Info sheet now explicitly says “Full ring: 90 days.” |
| Area line | `Sparkline`: midpoint-control cubic Beziers through each point, 2.5 dp round stroke, 28%-to-transparent vertical fill; reveal by left-to-right clipping. Static 6/4 dp endpoint backing/dot appears at progress ≥0.98. Points are equally spaced. No line glow in this source. | Already smooth: weighted harmonic monotone cubics with bounded controls, actual timestamp spacing, padded real extrema, 28% fill, 2.5 dp round stroke and endpoint glow. It is **not** a straight-segment chart. Requires four verified points. | Retain the stronger current interpolation: it preserves flat runs, reversals and funding cliffs without overshoot. Restore the legacy reveal rhythm, add a subtle 7 dp / 8% line halo and round joins, anchor the gradient to plot padding, and fade the final verified dot/backing/glow after the reveal reaches it. Draw the marker outside the reveal clip to avoid slicing its halo. No points or historical values are added. |
| Weekly spend | `StackedBars`: stack segments inside a rounded silhouette; all heights grow from baseline with shared entrance. Width is 56% of slot, 6 dp corners; zero totals draw 3 dp stubs. Real labels below. | Already has 24 dp pills, radius half width, stacked clipping, 4 dp zero stubs and compact vendor totals. Shapes are closer to the owner screenshot than legacy's fixed 6 dp corners. | Retain current pill shapes, clipping and narrow-screen spacing; restore 700 ms baseline growth, including zero→positive evidence. Preserve local Monday buckets, real vendor labels, partial-history disclosures and empty-week stubs. |
| Interest bars | `LabeledBar`: label/value above 8 dp track, 4 dp rounded fill; width grows using entrance. Legacy clamps fraction to at least 2%, which can paint a false positive value. | Same useful silhouette; nullable decimal fractions, true zero width, live Aave APY and explicitly simulated vault track. Entrance may already be complete when APY arrives. | Keep current shape and honest zero/null behavior. Animate each row when a positive verified fraction arrives; retain 700 ms width growth. No legacy rate constants or minimum positive fills. |
| Spare-cash hero | `HeroTile` in a green/red tinted `BentoTile`; static formatted amount, coverage icon/status and helper text. Tint is a vertical alpha gradient. **No hero entrance/count-up animation.** | Static exact surplus, verified coverage, diagonal green gradient and provenance/freshness info sheet. More accurate current wording and accounting checks. | Keep current implementation and stable immediately readable values. No reason to port a weaker data/status model or invent hero motion absent from legacy. |
| Worth knowing | `AttentionTile` ranks `LocalFinding`s by severity; colored dots and text. Shows two important findings, then “See all”. `Expander` owns `AnimatedVisibility`, but this call passes empty content: the actual finding rows change without an explicit row transition. | `WorthKnowing` renders exact live runway, verified next bill, surplus and vault statements in dotted rows. No collapse/expand or decorative animation. | Keep current statements and accessibility. Legacy does not provide a meaningful findings animation to transplant; importing its local-finding/demo pipeline would violate scope. |

## Data and accessibility boundary

Only chart rendering, entrance lifecycle, chart motion tokens and runway scale
explanation changed. `HomeCharts.kt` continues to take `WalletState` and `TxIndex`:

- Allocation uses exact current surplus, funded reserve + buffer and vault assets.
- Runway uses the current pinned cash/outflow evidence, requires seven observed
  days, excludes self-transfers and vault deposits, and retains current thresholds.
- The area line uses reconstructed Safe USDC samples, with original timestamps
  and the four-sample minimum. Historical vault values remain unavailable. Its
  existing info sheet discloses actual observation dates and sample count.
- Weekly bars use indexed bill payments and registered/indexed vendor labels.
- Interest uses live Aave supply APY. The existing **Demo vault** row continues
  to say **simulated**, with an empty track: no annual rate is fabricated.
- Worth-knowing statements still use the current evidence and accounting checks.

No `DemoData`, legacy snapshot/analytics provider, seeder, fixture balance,
synthetic history or screenshot value is imported. Money remains `BigInteger` /
`BigDecimal`; only dimensionless canvas ratios use floats. Existing info-sheet
actions, 48 dp targets, chart descriptions, numeric labels and large-font stacked
layout remain. Animation affects geometry only, never accessible numeric values.

At animator scale zero, each newly keyed entrance initializes at its final state.
For nonzero scales, `Animatable` runs in Compose's existing coroutine context,
which applies the live system `MotionDurationScale` (including a change to zero
during playback). The 700 ms token is not manually multiplied by system scale.
This integration was checked against the locally cached Compose 1.7.6 classes:
`WindowRecomposer_androidKt` observes `animator_duration_scale`, and
`SuspendAnimationKt` reads `MotionDurationScale` during playback.

## Validation

Completed successfully from the K29 worktree:

```bash
bash scripts/android_planner/build.sh && bash scripts/android_planner/build_core.sh
cd src/Fintech_Wallet_Agent/fusion-core-android
source /root/projects/fusion_wallet/scripts/edge_android_env.sh
./gradlew testDebugUnitTest assembleDebug && \
  unzip -l app/build/outputs/apk/debug/app-debug.apk | grep -q libfusion_planner_android.so && \
  test -s /root/projects/fusion_wallet-wt/K29/docs/ux/LEGACY_DASHBOARD.md
```

Both native scripts exited 0. The exact definition-of-done chain exited 0:
**BUILD SUCCESSFUL**, 202 tests across 37 suites, zero failures/errors and three
existing opt-in skips. No tests were changed. The 29 MB debug APK contains
`lib/arm64-v8a/libfusion_planner_android.so` and
`lib/arm64-v8a/libfusion_android_core.so`. `git diff --check` passes in both repos.
The native scripts left no tracked changes outside the owned UI files.

Both attached ADB targets
(`127.0.0.1:5555`, `emulator-5554`) report **unauthorized**, so device screenshots
and playback at different system animator scales cannot be claimed. Source
comparison used the actual extracted implementation and both owner screenshots.
