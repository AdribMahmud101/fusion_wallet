# Card K29 — Port the legacy dashboard's smooth charts and animations (real data only)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). The owner has the LEGACY app source at `/root/legacy/fusion_wallet_source.zip`; its Home dashboard had the polished look in `docs/ux/reference/home_top.png` / `home_bottom.png` (smooth animations, donut, runway gauge, smooth area line, weekly pill bars, interest bars). The current app (card K28) approximates it. Goal: reuse the legacy implementation where it is better.

## Steps
1. `unzip -q /root/legacy/fusion_wallet_source.zip -d /root/legacy/src` (if not already). Find the dashboard/Home screen and its chart/animation code (Compose Canvas, animate*AsState, Animatable, spline/path code, gradients). Write `docs/ux/LEGACY_DASHBOARD.md`: file list, each component (donut, gauge, area line, bars, interest bars, hero, worth-knowing), how it animates, and a **side-by-side comparison** with the current `ui/charts/ChartKit.kt` / `ui/treasury/HomeCharts.kt`: is it the same visual system? what is smoother/better there?
2. Port the better pieces into the current app: animation (enter sweep/grow, easing, durations), line smoothing/gradient/glow, donut caps/gaps, bar shapes. Keep the CURRENT data wiring — every value stays real chain/evidence data (no legacy demo/fixture data, no DemoData). Keep honest fallbacks (insufficient history), info sheets, and accessibility labels.

## Where you work
Umbrella worktree `/root/projects/fusion_wallet-wt/K29` (branch `card/K29`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K29 FETCH_HEAD`. From the umbrella worktree root: `bash scripts/android_planner/build.sh && bash scripts/android_planner/build_core.sh`; then `source /root/projects/fusion_wallet/scripts/edge_android_env.sh; ./gradlew testDebugUnitTest assembleDebug` in `src/Fintech_Wallet_Agent/fusion-core-android`.

## Owned paths
Submodule: `ui/charts/**`, `ui/treasury/**` (Home), `ui/theme/**` (only motion/chart tokens). Umbrella: `docs/ux/LEGACY_DASHBOARD.md`. Do not touch data/inference/agent/signer.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K29/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && unzip -l app/build/outputs/apk/debug/app-debug.apk | grep -q libfusion_planner_android.so && test -s /root/projects/fusion_wallet-wt/K29/docs/ux/LEGACY_DASHBOARD.md
```
exits 0. No fabricated data; no test weakened; animations respect the system animator scale. Commit on `card/K29` (both repos). Do not merge, do not push.
