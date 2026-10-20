# K47C — fresh build live HOLD diagnosis

## Run and decision

- Built the current worktree app with `scripts/android_planner/build.sh`,
  `scripts/android_planner/build_core.sh`, and the locked Gradle
  `assembleDebug` task. Installed the debug APK as an update on the OnePlus 11
  (CPH2447, Android 16); its existing model, obligations file, and encrypted
  session envelope remained present.
- Ran exactly one live tick through
  `python3 scripts/android_planner/run_tick.py --local-host` at 2026-10-20
  13:40 UTC. The report is `locked_dry_run.json`.
- This was a real app decision (`dryRun: false`): the on-device
  `mnn+ahem-g@android` planner in `app.fusion.wallet:inference` returned
  `model.raw: {"choice":"H"}`. `decisionExplanation` is
  “Nothing eligible to move within the current policy”; the chosen candidate
  and candidate-set hold both have `reason: HOLD`, and
  `candidateSet.candidates` is empty.
- The report has no `allocationMandate`, `execution`, or `txHash`. No
  transaction was sent and the tick was not repeated.

## Why it held

The snapshot and K44 quant section are present, so this run did not reproduce
K47B's missing quant section or a stale-snapshot HOLD:

- Snapshot block **48,367,663**, timestamp **2026-10-20 13:40:14 UTC**. The
  report completed at **13:40:26.922 UTC**, making the snapshot **12.922
  seconds old** at completion. The report does not serialize a block-lag
  field; an independent Base Sepolia head lookup at the report completion
  time returned block **48,367,669**, a **6-block lag**.
- `quant.blockNumber` and `rateEvidence.block` both match the snapshot.
  `rateEvidence.status` is `LIVE` (19 bps), and `quant.depositAllowed` is
  `true`.
- `quant.candidates` contains two **UNMEASURED** entries. For the first
  vault, the 75 USDC size and 2-raw-unit slippage quote are present, but
  `gasUsdRaw` is null and `gasStatus` is `UNMEASURED`; the second vault also
  has no usable slippage quote. Thus neither candidate has a measured
  break-even or actionable size. `vaultQuotes` is empty, and native candidate
  enumeration produced no candidates.
- The module snapshot says `paused: false`; its session was live and had not
  expired. Safe cash remained **9,399.001234 USDC**. No on-chain policy was
  changed.

The new report therefore has **no break-even value** to report. The earlier
K47B preview in `K47B_75_quant_report.json` measured **11.69 days** for 75
USDC; that quote does not fill in the missing gas measurement in this live
tick.
