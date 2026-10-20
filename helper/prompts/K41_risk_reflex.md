# K41 (Card D) — AI risk control: detect an anomaly and stop the money, on-chain

Read `docs/WIN_PLAN.md`, `evidence/results/ARENA.md` (A10, A17), and the Android agent README first.
Android: submodule `src/Fintech_Wallet_Agent/fusion-core-android` (commit there, then the pointer). You own `agent/risk/*`
and the guardian auto path. Do not touch `agent/autopilot/*` or `advisor/*`.

## Why
Topic F lists risk control. Today the only on-chain guardian pause was a human tap. A17's automatic reflex exists only on Anvil.

## Build
1. Risk monitor (runs at each refresh and each agent tick): checks with real chain reads, e.g. vault implementation slot
   changed vs pinned (A17), unexpected payee/obligation root change, Safe cash below committed obligations, abnormal outflow
   rate. Each produces a typed RiskSignal with evidence (block, values).
2. "Auto-protect" setting (off by default; on for the demo phone): on a critical signal, the app's guardian signer (existing
   separate envelope) sends `pause` to KavachiModule automatically; UI shows a red banner with the signal, block and tx hash.
   Resume stays 2-of-2 owners only.
3. Demonstrate on Base Sepolia with an owner-controlled, clearly-labelled DEMO trigger that is safe (e.g. the owner upgrades the
   demo vault proxy implementation, or another real but reversible condition you find in the contracts). Record: trigger tx
   (OWNER/SCRIPT), auto pause tx (APP, guardian), owner resume tx(s) (OWNER). Index all with honest sources, add verify_all checks,
   rebuild tx_index.
4. Update the A17 row in `ARENA.md` via the generator only if the app path now covers it, and say exactly what is covered.
5. Tests + `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`), `cargo test`, `python3 scripts/judge/verify_all.py`.
Never print or commit keys. Base Sepolia only. Leave the module resumed (live) at the end and prove it.
