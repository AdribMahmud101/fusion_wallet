# Card K26 — Device-found polish: icons, Home tiles, history, load speed

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). The supervisor ran the merged app on the demo phone (Tecno CL6, 1080×2436, Android) and found these defects. Judges will see these screens first. Fix each; no dummy data.

## Where you work
Umbrella worktree `/root/projects/fusion_wallet-wt/K26` (branch `card/K26`); submodule: `git submodule update --init src/Fintech_Wallet_Agent`, inside `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K26 FETCH_HEAD`. **Native libs are gitignored: run `bash /root/projects/fusion_wallet/scripts/android_planner/build.sh && bash /root/projects/fusion_wallet/scripts/android_planner/build_core.sh` from your umbrella worktree root first** (they write into your submodule's `jniLibs`), then `source /root/projects/fusion_wallet/scripts/edge_android_env.sh; ./gradlew testDebugUnitTest assembleDebug`. Open links only with `context.startActivitySafe(...)`. Design system: `ui/theme/FusionTokens.kt`, `ui/components/`, `docs/ux/UX_PLAN.md`.

## Defects (fix all; commit after each)
1. **Broken icon on the Home hero:** a solid blue square renders under "9,399.00 USDC" (likely the Base network logo vector failing). Also the USDC coin icon in the hero renders as a white blob on blue — check both vector drawables against the official artwork (paths, viewport, fill rules) and render them in a JVM/Paparazzi-free sanity check (e.g. parse the XML and assert path count > 0 and viewport sizes). Fix or replace with correct vectors; record sources in `res/drawable/ICONS.md`.
2. **Duplicate network badge:** "Base Sepolia · TEST" appears in the top bar AND inside the hero tile. Keep only the top bar one.
3. **Bucket bar renders as striped noise** (many thin green slices). It must be a clean 100 % stacked bar: one solid segment per bucket (Owed / Buffer / Surplus), proportional, labelled with values beside/below; a zero bucket gets no segment but keeps its label "0.00".
4. **Balance history says "0 verified points in 7 days"** although the Safe had dozens of USDC transfers in the last day. Build the series from USDC `Transfer` events to/from the Safe (asset `0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f`, Safe `0x029869910714D9e09b854a2688311AFd42fBb44E`) plus module events, reconstructing the running balance backwards from the current balance; cache incrementally (do not rescan from deployment on every launch). With ≥ 4 points draw the area chart; otherwise the stat tile.
5. **Slow history scans:** the onboarding policy step's vendor list and Bills' "receipt not in loaded history" both wait on full log scans from the deployment block over the public RPC (20 s+). Share one incremental, cached event index (persisted, resumed from the last block) used by Home history, Bills receipts, vendor lists and Activity; chunk `eth_getLogs` ranges; parallel requests; per-request timeout.
6. **Safety tab shows a raw ISO timestamp** ("Arena measured 2026-10-19T19:15:51.279410+00:00"): format as local date/time ("Measured 19 Oct, 19:15 UTC"). Sweep all screens for raw ISO/epoch timestamps and raw enum codes.

## Owned paths (inside the submodule)
`ui/**` (except `ui/onboarding/**` logic beyond consuming the shared index), `data/**`, `chain/**`, `res/**`. Do not edit `inference/**`, `agent/**`, `signer/**`.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K26/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && unzip -l app/build/outputs/apk/debug/app-debug.apk | grep -q libfusion_planner_android.so
```
exits 0; JVM tests for the bucket-segment math, balance reconstruction from transfers, timestamp formatting and the incremental index resume; `ui/K26_FIXES.md` lists each defect → change → test. Measure on a real device if reachable (`adb -s 119703846S004282` is the Tecno over USB on the laptop — not reachable from here; use the OnePlus at `adb -s eb3cea7b` if `adb` works in your environment) and record timings.

## Acceptance
No fabricated data; no Float/Double money; no test weakened. Commit inside the submodule on `card/K26`. Do not merge, do not push.
