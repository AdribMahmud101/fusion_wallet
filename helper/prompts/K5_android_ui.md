# Card K5 — Android UI on real state (Phase 3)

You are the Android UI engineer for Fusion Wallet (FinTechathon 2026, Topic F; hard deadline 2026-10-20 15:59 UTC).
Innovation & interaction is 30% of the score and judged on what people see in the video and screenshots. Make it clear, honest and beautiful.

## Where you work
- Umbrella worktree: `/root/projects/fusion_wallet-wt/K5` (branch `card/K5`). Android app = submodule `src/Fintech_Wallet_Agent` (own repo). In your worktree run `git submodule update --init src/Fintech_Wallet_Agent`, then inside the submodule `git fetch /root/projects/fusion_wallet-wt/K4/src/Fintech_Wallet_Agent card/K4 && git checkout -b card/K5 FETCH_HEAD` so you start from card K4's data contract. **Commit inside the submodule on `card/K5`.**
- App module: `src/Fintech_Wallet_Agent/fusion-core-android/app`, package `app.fusion.wallet`. Existing UI: Obsidian bento design, tabs, canvas charts, EN/中文 (`ui/i18n`), PBKDF2 login. Keep the existing visual language (`docs/UI_REFERENCE.md`, `system architecture/UI_REFERENCE.md`).
- Data contract (owned by card K4, read-only for you): `app/fusion/wallet/data/WalletState.kt` + `WalletRepository`. Render ONLY from `WalletState`. If you need a field that is missing, write it into `ui/NEEDS_FROM_K4.md` and render a clear "—" until it exists. Never fabricate numbers.
- Build (verified on this aarch64 machine): `source /root/projects/fusion_wallet/scripts/edge_android_env.sh` then `./gradlew assembleDebug` in `fusion-core-android`.
- Spec: `system architecture/fusion_wallet_architecture.md` §17 (UX spec: every screen, states, copy, accessibility, Judge mode), §6 tiers, §18.4 claims to avoid.

## Owned paths (exclusive; inside the submodule)
`.../app/src/main/java/app/fusion/wallet/ui/**` EXCEPT `ui/MainViewModel.kt` (K4 owns it), and `.../app/src/main/res/**`. Do not touch `data/**` or `chain/**`.

## Screens (priority order; commit after each)
1. **Treasury (Home)**: total balance, the four buckets (reserve / owed / buffer / surplus) as a bar, vault positions, network badge "Base Sepolia · TEST", last-refresh block number.
2. **Agent**: feed of agent actions as decoded cards (kind, amount, payee label, tier, status, decisionHash short form, "View on BaseScan" link `https://sepolia.basescan.org/tx/<hash>`); queued actions with a live countdown to `readyAt`; big **Stop** button (guardian pause) — UI + confirmation dialog; wire it to a callback the ViewModel exposes, show "not wired" if K4 has not provided it.
3. **Bills & Vendors**: payees with ACTIVE/COOLING badge and cooling countdown; a "new bank details" payee shown as COOLING with the warning copy from §17.
4. **Safety**: attacks blocked, generated from `WalletState` evidence rows of kind INTENTIONAL_REVERT (each with its tx link) — never hard-coded "7 of 7".
5. **Judge mode**: a list of every evidence row (id, what it proves, tx link, block) — this is what a judge taps through.
- Every screen: loading, empty and error states; EN + 中文 strings in `res/values` and `res/values-zh`; content descriptions for accessibility; works on a 6" phone.
- DEMO data only in debug builds and visibly labelled DEMO.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K5/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew assembleDebug && ! grep -rn "7 of 7\|7/7" app/src/main/java/app/fusion/wallet/ui app/src/main/res
```
exits 0, and `ui/SCREENS.md` lists each screen, which `WalletState` fields it renders, and its states.

## Acceptance
- No hard-coded balances, hashes or counts in release paths. Claims to avoid (§18.4) absent from strings.
- Commit inside the submodule on `card/K5`. Do not merge, do not push.
