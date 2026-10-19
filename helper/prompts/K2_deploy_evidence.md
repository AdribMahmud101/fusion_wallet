# Card K2 — Base Sepolia deployment + evidence scripts (Phase 3, CRITICAL PATH)

You are the deployment engineer for Fusion Wallet (FinTechathon 2026, Topic F, hard deadline 2026-10-20 15:59 UTC).
Topic F scores **real on-chain transaction hashes**. Your output is the judges' proof.

## Where you work
- Worktree: `/root/projects/fusion_wallet-wt/K2` (branch `card/K2`). `cd` there first. Work ONLY there.
- Contracts are merged and green (card K1): `src/contracts/` (forge test 157/157). Read `src/contracts/STATUS.md` and `src/contracts/SPEC_ISSUES.md` if present.
- Spec (read-only): `system architecture/fusion_wallet_architecture.md` — §0.3 (D1, D2, D11, D17, D20), §0.5 (verified addresses, Safe v1.4.1 addresses UNVERIFIED), §6 (tiers), §7.1–7.2, §19 (evidence list E1–E16 and row format §19.5), §4.1c SL2/SL5.
- Toolchain: forge/cast/anvil 1.8.5. RPC: `https://sepolia.base.org` (chain 84532).

## Keys (SECRETS — read this twice)
- Testnet keys live in `/root/projects/fusion_wallet/.env.local` (outside your worktree; DEPLOYER/OWNER/APPROVER/GUARDIAN/SESSION/OPS `_ADDRESS` and `_PRIVATE_KEY`, plus `RPC_URL`). Load with `set -a; . /root/projects/fusion_wallet/.env.local; set +a`.
- NEVER print, echo, log, commit or copy a private key into any file. Pass keys only as `--private-key $VAR` / env. Scripts read them from env. Add `scripts/audit_secrets.sh` (grep for `0x[0-9a-fA-F]{64}`, `PRIVATE_KEY=`) and run it before every commit.
- Only DEPLOYER is funded (~0.13 ETH on Base Sepolia). Fund the others from it: OWNER, APPROVER, GUARDIAN 0.01 each; SESSION 0.02. Keep the rest for deploys.

## Owned paths (exclusive)
`src/contracts/script/**`, `scripts/**`, `evidence/**`, `deployments/**`. Do NOT edit `src/contracts/src/**` or tests; if a contract bug blocks you, write `evidence/BLOCKERS.md` and stop that path.

## Do, in order (commit after each step)
1. **Preflight**: `cast code` each Safe v1.4.1 address in §0.5 on 84532 (non-empty?). Aave: `cast code` USDC/waUSDC from §0.5; look for the Aave faucet (§0.5) and try minting ≥ 10,000 USDC to the treasury. If Aave USDC cannot be obtained, apply D2/SL2: Circle USDC `0x036CbD53842c5426634e7929541eC2318f3dCF7e` + SCALED profile + two `KavachiDemoVault`s. Record the choice + evidence in `deployments/PREFLIGHT.md`. If Safe is missing, apply SL5 (deploy official v1.4.1 artifacts from `src/contracts/lib/safe-smart-account`).
2. **Rehearse on Anvil** first: `script/Deploy.s.sol` + setup on a local anvil (chain 31337) end to end; write `deployments/31337.json`.
3. **Deploy to Base Sepolia**: Safe 2-of-2 [OWNER, APPROVER], `KavachiDemoVault`, `KavachiTestSanctionsList`, `KavachiModule` (enabled on the Safe), vaults per profile. **Source-verify** every contract (`forge verify-contract`; BaseScan key may be absent — then use Sourcify `--verifier sourcify`, which needs no key). Write `deployments/84532.json` (addresses, runtime code hashes, block numbers, compiler settings, profile).
4. **Setup batch (D17)** via Safe MultiSendCallOnly (2-of-2 signatures from OWNER + APPROVER keys): configurePolicy + commitObligations + startSession + resume. Then `addPayee` ×2 immediately — **payee cooling (D11, 3600 s) is the critical path; start it as early as possible**.
5. **Evidence**: `scripts/collect_evidence.py` appends rows to `evidence/base-sepolia.json` in the §19.5 format (tx hash, block, kind, decisionHash, expected event, source=SCRIPT). Produce E1–E5 now (deploy, verify, Safe setup, module enable, payees). After cooling ends, produce the PAY/DEPOSIT/WITHDRAW/queue/cancel/pause rows that you can produce with forge scripts (`script/Golden*.s.sol` belongs to K9 — do not write it; use your own `script/Evidence*.s.sol`).
6. `scripts/verify_evidence.py` (D20): read-only, no keys; fetches every receipt from the public RPC and checks status + expected event + decisionHash. Must pass on what exists.

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K2 && bash scripts/audit_secrets.sh && python3 scripts/verify_evidence.py evidence/base-sepolia.json
```
exits 0, `deployments/84532.json` lists every deployed address, each contract is source-verified (link in `deployments/84532.json`), and `evidence/base-sepolia.json` has at least E1–E5 with real tx hashes.

## Acceptance checks
- Every tx hash in evidence is real and fetchable on https://sepolia.basescan.org.
- No private key anywhere in git history of `card/K2`. No secrets in logs you commit.
- Commit on `card/K2` after each step. Do not merge, do not push.

## Unblocks
K9 (golden path + verifier), K4/K5 (app reads real state from `deployments/84532.json`), K7 (docs 06 evidence).
