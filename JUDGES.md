# Fusion Wallet — for judges
> **FinTechathon 2026 · International Track · Topic F — Agent Wallet**
> **Network:** Base Sepolia (chain 84532) · test money only

---

## Try the app in 2 minutes (no build needed)

1. On an arm64 Android phone (Android 10+), download and install
   **https://github.com/AdribMahmud101/fusion_wallet/releases/download/app-latest/fusion-wallet.apk**
   (allow installing from your browser when asked; SHA-256 in the `.sha256` file next to it).
2. Sign in with one tap: choose **Guest Judge** (view-only) or **Lin Mei** (owner) on the sign-in screen — the demo credentials fill in automatically.
3. First-run setup (6 steps, each with **Skip**): live treasury checks read from Base Sepolia, and the on-device AI model download (~131 MB, SHA-256 verified, about a minute on Wi-Fi). Skip the model to browse instantly in monitor mode.
4. Tour: **Home** (treasury, buckets) → **AI Agent** (*Run Agent* = the on-device model decides on live chain state) → **Activity** (every on-chain action, with BaseScan links) → **Safety** (23/24 attacks stopped, with evidence) → **Settings → Judge mode** (every evidence row).

Every number in the app is read from the chain or from the evidence files in this repository.

## Verify in 2 Minutes (Keyless & Read-Only)

Judges can independently verify every on-chain claim, receipt, state invariant, and passport proof in under 2 minutes with zero keys or secrets.

### Prerequisites
- `python3` (standard library only)
- `cast` (Foundry CLI; optional, standard fallback JSON-RPC included)

### The Command
```bash
cd /root/projects/fusion_wallet-wt/K19
bash scripts/judge/verify_all.sh
```

### What `verify_all.sh` Verifies
1. **Base Sepolia Evidence (`scripts/verify_evidence.py`):** Checks all 30 on-chain receipts in [`evidence/base-sepolia.json`](file:///root/projects/fusion_wallet-wt/K19/evidence/base-sepolia.json), validating Safe threshold-2 invariants, contract bytecode hashes, receipt statuses, log topics, and decoded event fields.
2. **Autonomous Agent Golden Path (`scripts/agent/check_golden.py`):** Verifies the 3 live autonomous agent transactions in [`evidence/agent/golden.json`](file:///root/projects/fusion_wallet-wt/K19/evidence/agent/golden.json), ensuring status `0x1`, session signer authorization, passport `keccak256` integrity, and `ActionExecuted`/`ActionQueued` event anchoring.
3. **App & Mobile Proofs (`evidence/app/*.json`):** Checks guardian pause and owner resume transactions ([`evidence/app/guardian_pause.json`](file:///root/projects/fusion_wallet-wt/K19/evidence/app/guardian_pause.json)), verifying hardware Keystore-backed guardian actions.
4. **Live Demo Runs (`evidence/demo/runs.json`):** Verifies every live demo execution row, confirming receipt inclusion, session signer identity, and passport hash alignment.
5. **Contract Source Verification (Sourcify):** Directly queries Sourcify's public API to verify exact compiler matches (`0.8.24`, Cancun, 200 runs, via-IR) for all deployed contracts in [`deployments/84532.json`](file:///root/projects/fusion_wallet-wt/K19/deployments/84532.json).

---

## Live End-to-End Demo (One Command)

To watch the complete autonomous system execute live on Base Sepolia:

```bash
cd /root/projects/fusion_wallet-wt/K19
bash scripts/demo/live_demo.sh
```

### Flow Narrated by `live_demo.sh` (~2–4 minutes):
1. **Snapshot:** Queries on-chain balances, reserve buckets, and `KavachiModule` state; verifies gas runway and unpaused lifecycle.
2. **Fresh Bill Commitment:** Owners (2-of-2 Safe) commit a fresh, unique instant-tier bill ($2.00 USDC) to an active verified payee via MultiSend.
3. **Agent Tick (MNN + AHEM-G):** The edge planner loads weights, enumerates candidates, applies token-level steering, and outputs candidate `A` with full decision passport.
4. **Autonomous Payment:** The agent runner signs with `SESSION_PRIVATE_KEY` and broadcasts `execute()` on Base Sepolia.
5. **On-Chain Verification:** Confirms receipt status `1`, signer == session key, `decisionHash` anchored in event logs, `isPaid` == `true`, and verifies that duplicate payment attempts revert on-chain (`ObligationAlreadyPaid`).
6. **Provenance Logging:** Appends run evidence to [`evidence/demo/runs.json`](file:///root/projects/fusion_wallet-wt/K19/evidence/demo/runs.json) adhering to the §19.5 schema.

---

## Evidence Provenance Legend

| Provenance | Meaning | Trust Boundary |
|:---|:---|:---|
| **`SCRIPT`** | Deterministic operations executed by owner scripts | Safe 2-of-2 multisig or deployer setup; baseline infrastructure. |
| **`AGENT`** | Autonomous actions decided by edge model with AHEM-G | Signed exclusively by ephemeral session key within mandate & policy limits. |
| **`APP`** | Operations initiated from the Android mobile client | Emergency guardian pause signed via Android Keystore hardware-backed keys. |

---

## Definition of Done Check

```bash
cd /root/projects/fusion_wallet-wt/K19 && bash scripts/judge/verify_all.sh && python3 -c "import json;r=json.load(open('evidence/demo/runs.json'));assert len(r)>=1 and r[-1]['txHash'];print('ok')"
```
