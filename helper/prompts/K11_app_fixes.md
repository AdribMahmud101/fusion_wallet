# Card K11 — App fixes found on the real device (Phase 4, demo-blocking)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). The supervisor installed the merged APK on the demo phone (Tecno, 1080x2436) and found these defects on the Home screen. The video is recorded from this app, so each defect costs points.

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K11` (branch `card/K11`). `git submodule update --init src/Fintech_Wallet_Agent`, then inside the submodule `git checkout -b card/K11 origin/fusion-v3-integration || git checkout -b card/K11 fusion-v3-integration` — it MUST include the merged K4 + K5 commits (`git log --oneline | grep -E "card K4|card K5"`; if missing, `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K11 FETCH_HEAD`). Commit inside the submodule.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh` then `./gradlew testDebugUnitTest assembleDebug` in `fusion-core-android`.

## Owned paths (inside the submodule)
`.../app/fusion/wallet/data/**`, `.../wallet/chain/**`, `.../wallet/ui/**` (including `MainViewModel.kt`), `res/**`, `assets/**`. (K12 owns `.../wallet/signer/**` — do not touch.)

## Fix (commit after each; add a unit test where it is logic)
1. **Red banner "ACCOUNTING_MISMATCH: on-chain commitment differs from bundled bills"** — the app compares the on-chain obligations Merkle root with a bill list bundled in assets. Owners re-commit bills on-chain (other cards do this live), so a bundled list will always go stale. Fix: load the bill list from the newest source that MATCHES the on-chain root — try the bundled file, then `obligations-84532.json` fetched from the public GitHub raw URL `https://raw.githubusercontent.com/AdribMahmud101/fusion_wallet/main/evidence/obligations-84532.json` (and any `evidence/agent/obligations*.json` there). Show ACCOUNTING_MISMATCH only when NO candidate list reproduces the on-chain root, and then as a clear amber "Bill details unavailable for commitment 0x…; totals from chain" state, not a red error. Totals that the chain itself provides must still render.
2. **"Stale data" banner after 30 s** — on a public RPC with ~2 s blocks this fires constantly. Refresh every 15 s; show stale only if the last successful snapshot is older than 90 s, as a small amber chip, not a banner.
3. **Duplicate "Base Sepolia · TEST" badge** (header and next to the Home title) — keep only the header one.
4. **"Block ##48322201"** — double `#`. Render `Block #48322201`.
5. **Bottom nav label "Bills &" truncated** — use a short label ("Bills") in EN and the 中文 equivalent; labels must fit at 1080 px width.
6. Check the other tabs (Bills, AI Agent, Safety, Judge mode) for the same classes of defect (double symbols, truncation, duplicate badges, red errors for normal states) and fix them.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K11/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && ! grep -rn '"##\|##\$' app/src/main/java/app/fusion/wallet/ui
```
exits 0, and `ui/FIXES_K11.md` lists each defect with the file/line changed and the test that covers it.

## Acceptance
- No fabricated numbers; the amber state must say what is unknown. No Float/Double money. No test weakened.
- Commit inside the submodule on `card/K11`. Do not merge, do not push.
