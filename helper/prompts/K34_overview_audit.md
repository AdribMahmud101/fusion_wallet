# K34 — Make docs/SYSTEM_OVERVIEW.md accurate and complete

`docs/SYSTEM_OVERVIEW.md` is a features list plus architecture overview with Mermaid flow diagrams, for hackathon judges
(FinTechathon 2026 Topic F, Agent Wallet). It was drafted quickly and may contain wrong claims. Judges punish overclaiming.

## Do
1. Check every claim against the code and evidence in this repo: `src/contracts`, `src/fusion-core`, `src/fusion-planner`,
   `src/Fintech_Wallet_Agent/fusion-core-android`, `evidence/`, `scripts/`, `README.md`, `JUDGES.md`, `docs/ux/DASHBOARD_LAYOUT.md`.
   Known suspects: "Earn moves spare cash into a vault" (Home Earn is a dry-run preview only), "Vault withdraw",
   permission tier names, allowance numbers, "Accountant chat grounded in on-chain data", onboarding steps, Safety/Settings contents,
   evidence counts (tx_index rows, verify_all check count), arena numbers, contract names and addresses, which component calls which.
2. Fix wrong claims; add real features that are missing (sweep the app screens and contracts: events, guardian actions, queue/timelock,
   obligations commitment, x402/arena, reproducible build, release APK). Add the deployed addresses table (Base Sepolia).
3. Keep it short and scannable. Mermaid diagrams must render on GitHub (validate syntax). Diagrams must match real call paths.
4. Every number must be traceable; put the source file next to it in parentheses where useful.
5. Docs only: do not change code. Commit `docs/SYSTEM_OVERVIEW.md` and link it from `README.md`.
