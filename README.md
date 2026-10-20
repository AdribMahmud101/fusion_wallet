# Fusion Wallet

> **Autonomous AI Treasury Agent for Small Businesses**  
> *FinTechathon 2026 International Track — Topic F: Agent Wallet*  
> Reference Implementation & Submission Dossier (Phase 4)

---

## 1. Executive Overview

**Fusion Wallet** is an autonomous AI treasury agent designed for small cross-border enterprises holding stablecoins (USDC). It solves a fundamental problem in autonomous Web3 finance: enabling an AI model to actively manage operational liquidity—paying verified bills, reserving funds for liabilities, and sweeping surplus cash into yield vaults—without granting the model unbounded authority to transfer funds or leak assets.

### The Problem
* **Business Email Compromise (BEC) & Invoice Fraud:** FBI IC3 recorded ~24,768 BEC complaints and ~$3.05B in losses in 2025. Attackers manipulate invoice accounts or inject malicious payee data.
* **Address Poisoning & Vanity Confusion:** High-profile losses of ~$50M (Dec 2025) and ~$12.25M (Jan 2026) highlight the risks of manual address handling.
* **Unconstrained AI Agent Vulnerability:** The May 2026 Grok/Bankr incident demonstrated how an indirect prompt injection drained ~$150K–$200K from an AI agent wallet with open transfer authority.
* **Idle Operating Cash:** Cash sitting idle earns 0%, but naive automated sweeps risk touching money already owed for upcoming payroll and vendor obligations.

### The Solution: Bounded Decision Autonomy
Fusion Wallet replaces open-ended agent autonomy with a **bounded decision paradigm**:
1. **The Model Chooses, It Never Writes:** Prompts present discrete options (`A`, `B`, … `G`, or `H` for HOLD). The inference engine is physically incapable of emitting an Ethereum address, calldata, or raw amounts.
2. **Decode-Time Grammar Gating (AHEM-G):** An in-engine token grammar gate inside the native MNN inference loop restricts sampling to valid option letters, blocking prompt injection attacks at the sampler level.
3. **Kavachi On-Chain Guard Layer:** A dedicated Gnosis Safe v1.4.1 module enforcing cryptographic reserve accounting ($C - \text{assets} \ge R + B$), 180-second cancellation queues, payee cooling delays, and two-person maker-checker controls.
4. **On-Chain Decision Passports:** Every executed transaction embeds a cryptographic `decisionHash` corresponding to an RFC 8785 canonical JSON passport published for external auditability.

---

## 2. Repository Layout

```
.
├── system architecture/               # Full architecture v5.0 specification
├── spec/
│   ├── abi/                           # Frozen contract ABIs (IKavachiModule.json, etc.)
│   ├── vectors/                       # Frozen cryptographic test vectors (v4_1.json)
│   └── DECISIONS.md                   # Formal specification and hashing decisions
├── src/
│   ├── contracts/                     # Foundry smart contract project (Kavachi guard layer)
│   │   ├── src/                       # KavachiModule.sol, KavachiPolicyLib.sol, HashV41.sol, etc.
│   │   ├── test/                      # 157 unit, integration, and invariant tests
│   │   └── script/                    # Deployment and vector generation scripts
│   ├── fusion-core/                   # Pure Rust financial engine (candidates, economics, hashing)
│   │   └── src/                       # Candidate enumerator, preview checks, passport builder
│   ├── fusion-planner/                # AHEM-gated inference runner (MNN + AHEM-G + UniFFI)
│   ├── Fintech_Wallet_Agent/          # Android application (Jetpack Compose, multi-process)
│   ├── mnn-android-sdk/               # Pinned MNN mobile framework
│   └── ahem/                          # Pinned AHEM token-level grammar gating submodule
├── deployments/                       # Chain manifests (84532.json for Base Sepolia)
├── evidence/                          # Execution artifacts, host results, and run logs
│   ├── base-sepolia.json              # 30 operator script execution receipts (source: SCRIPT)
│   ├── agent/                         # Autonomous golden path txs + passports (source: AGENT)
│   ├── app/                           # In-app guardian pause + resume (source: APP / OWNER)
│   ├── results/                       # 24 attack arena scenarios & latest.json benchmarks
│   └── planner/                       # host_results.json (200 AHEM-G adversarial evaluations)
├── docs/                              # Auxiliary documentation (UI, assets, accountant)
│   ├── EDGE_ANDROID_BUILD.md          # Complete Android edge build toolchain documentation
│   └── submission/                    # Official FinTechathon submission deliverables
│       ├── 01_TECHNICAL.md            # System architecture, core algorithm, security design
│       ├── 02_DECK.md                 # 10-slide outline with speaker notes (≤ 10 min)
│       ├── 03_VIDEO_SCRIPT.md         # 5-minute timed demonstration script and shot list
│       ├── 05_SECURITY_SELF_ASSESSMENT.md # Permission matrix, controls mapping, known limits
│       └── 06_EVIDENCE.md             # Complete evidence dossier generated from repo files
└── scripts/                           # Independent verifiers and build automation scripts
    ├── verify_evidence.py             # Keyless Base Sepolia receipt and revert verifier
    ├── edge_android_build.sh          # Android edge compilation script
    ├── edge_android_env.sh            # Android toolchain environment setup
    └── agent/check_golden.py          # Keyless autonomous agent passport verifier
```

---

## 3. Provenance Framework

To maintain absolute evaluation integrity, all execution records are categorized by their explicit provenance:

| Provenance Tag | Execution Origin | Description & Boundaries | Keyless Verifier Tool |
|---|---|---|---|
| **`SCRIPT`** | Operator / Test Scripts | Deterministic deployment and verification scripts (`src/contracts/script/EvidenceActions.s.sol`). **Never designated as autonomous.** | `python3 scripts/verify_evidence.py evidence/base-sepolia.json` |
| **`AGENT`** | Autonomous Agent Runner | Transactions broadcast autonomously by `agent-runner` with the on-device model under AHEM-G (`planner.kind = mnn+ahem-g`). Each `decisionHash` equals `keccak256(canonical_passport)`. | `python3 scripts/agent/check_golden.py` |
| **`APP`** | Physical Android Phone | In-app actions triggered from the mobile user interface and broadcast by the isolated `:signer` process using an Android Keystore-wrapped key on real hardware (Tecno CL6). | Keyless receipt & event verification (`evidence/app/guardian_pause.json`) |
| **`OWNER`** | Safe Multi-Signature (2-of-2) | Dual-signature administrative actions (Mei & Wei) configuring policy, committing bill roots, or resuming execution after an in-app pause. | `python3 scripts/verify_evidence.py evidence/base-sepolia.json` |

---

## 4. Keyless Verification: Run It Yourself

Judges and independent auditors can verify all on-chain receipts, event logs, and passport cryptographic bindings keylessly without keys or specialized credentials:

```bash
# 1. Verify 30 operator script receipts against Base Sepolia RPC
python3 scripts/verify_evidence.py evidence/base-sepolia.json

# 2. Verify 3 autonomous agent golden path transactions and passport hashes
python3 scripts/agent/check_golden.py

# 3. Regenerate and verify the evidence dossier
python3 docs/submission/gen_evidence.py
```

---

## 5. Build, Test, and Verification Instructions

### 5.1 Smart Contracts (`src/contracts`)
The on-chain guard layer is developed with Foundry (Solidity `0.8.24`, Cancun EVM, `via_ir = true`, 200 optimizer runs).

```bash
cd src/contracts

# Format check and ABI verification
forge fmt --check
python3 script/check_abi.py

# Build contracts and verify EIP-170 bytecode sizes
forge build --sizes

# Run full test suite (157 tests, fuzz 1,000 runs, invariant 64x32)
forge test

# Run CI profile (fuzz 10,000 runs, invariant 256x64, 16,384 handler calls)
FOUNDRY_PROFILE=ci forge test
```

#### Size & Gas Benchmarks (from `src/contracts/STATUS.md`)
* `KavachiModule` runtime size: **24,301 bytes** (275 bytes below the 24,576 byte EIP-170 limit).
* `KavachiPolicyLib` runtime size: **5,854 bytes**.
* Gas for `KavachiModule.execute(PAY_OBLIGATION)` in real Safe fixture: **292,385 gas**.
* Emergency unwind gas for both vaults: completes well within the 1,000,000 gas ceiling on Anvil.

### 5.2 Pure Rust Engine (`src/fusion-core`)
`fusion-core` implements integer candidate enumeration, preview simulation, Merkle trees, and cryptographic hashing.

```bash
cd src/fusion-core

# Run test suite and vector parity checks
cargo test --locked

# Run the deterministic/host agent runner CLI
cargo run --bin agent-runner -- --help
```

### 5.3 Inference Engine & Planner (`src/fusion-planner`)
`fusion-planner` integrates the AHEM-patched MNN runtime with LFM2.5-230M and enforces decode-time token gating.

```bash
cd src/fusion-planner

# Run unit tests and gate lifecycle tests
cargo test --release

# Run model-configured acceptance tests against local weights
FUSION_MODEL_DIR=/path/to/lfm_230m cargo test --release

# Run host evaluation runner
FUSION_MODEL_DIR=/path/to/lfm_230m python3 ../../evidence/planner/run_host.py
```

### 5.4 Android Mobile Application (`src/Fintech_Wallet_Agent`)
The Android mobile application features Jetpack Compose UI, a live Base Sepolia blockchain monitor, multi-process isolation (`:main`, `:inference`, `:signer`), and Android Keystore key management.

```bash
# 1. Initialize Android submodule
git submodule update --init src/Fintech_Wallet_Agent

# 2. Source Android build environment variables (Java 17, Android SDK 36, NDK 27)
source scripts/edge_android_env.sh

# 3. Build the native libraries the APK loads (gitignored, built from source):
#    the on-device planner (AHEM-G MNN fork + LFM2.5 wrapper) and the Rust core.
#    Without this step the app builds but the on-device model self-test fails.
bash scripts/android_planner/build.sh
bash scripts/android_planner/build_core.sh

# 4. Run the unit tests and build the debug APK
cd src/Fintech_Wallet_Agent/fusion-core-android
./gradlew testDebugUnitTest assembleDebug
# Output APK: app/build/outputs/apk/debug/app-debug.apk (~31 MB, model files not included:
# the app downloads and SHA-256-verifies them in onboarding step 4)
# Detailed build guide: docs/EDGE_ANDROID_BUILD.md
```

---

## 6. Empirical Verification & Evidence Summary

Full evidence tables and transaction links are compiled in [`docs/submission/06_EVIDENCE.md`](docs/submission/06_EVIDENCE.md).

### 1. Smart Contract Verification (`src/contracts/STATUS.md`)
* **157/157 passing tests** with zero failures and zero skipped tests.
* **Formal Invariant Campaign (I1–I7):** Verified via `TreasuryHandler` across 64 runs, 2,048 handler calls, and CI runs of 256 runs, 16,384 handler calls with zero reverts:
  * **I1:** Reserve preservation: $C \ge R + B$ holds across all deposit and investment operations.
  * **I2:** Treasury assets leave Safe only to Vault A, Vault B, or an active verified non-sanctioned payee matching a committed unpaid Merkle leaf.
  * **I3:** Cumulative limits strictly respected (gross daily limit $\le 4,000$ USDC, session limit $\le 8,000$ USDC).
  * **I4:** Invoices paid at most once across both agent and owner execution paths.
  * **I5:** Safe-to-vault ERC-20 allowances reset to exactly zero after every transaction.
  * **I6:** Paused module stops all agent operations; only 2-of-2 Safe transactions can unpause.
  * **I7:** Nonce consumption is strictly monotonic; cancelled/expired queues consume the nonce.

### 2. Autonomous Agent Golden Path (`evidence/agent/golden.json`)
* **`E7-AGENT` (Instant Bill Payment):** Autonomous `PAY_INSTANT` of 8.00 USDC in block `48322748` (`0x8bab4844...`), `decisionHash: 0x731de927...`.
* **`E8-QUEUE` (Delayed Bill Queued):** Autonomous queue of 75.00 USDC in block `48324683` (`0x5b7cd057...`), `decisionHash: 0xe04da051...`.
* **`E8-EXEC` (Delayed Bill Executed):** Autonomous delayed execution after 180s timelock in block `48324784` (`0xf1e0afcb...`), `decisionHash: 0xe04da051...`.
* **`HOLD-AGENT` (Fee Spike Defense):** Autonomous `HOLD` decision in block `48324783` when gas fees outweighed yield, preserving funds without broadcasting gas-wasting tx.

### 3. In-App Mobile Kill Switch (`evidence/app/guardian_pause.json`)
* **`E14-APP` (Guardian Pause):** Unilateral guardian pause (`0xb36b6d22...`) initiated from an Android phone (Tecno CL6) `:signer` process with Android Keystore-wrapped key in block `48325338`. Confirms `paused == true`.
* **`E14-APP-RESUME` (Owner Resume):** 2-of-2 Safe transaction (`0x82916108...`) calling `KavachiModule.resume`, restoring `paused == false`.

### 4. Adversarial Attack Arena & Corpus (`evidence/results/latest.json` & `ARENA.md`)
* **Arena (24 Scenarios):** 21 stopped, 2 escaped (A12: x402 budget signer not built; A17: proxy implementation swap escapes code hash pin), 1 not implemented (A13: crash journal).
* **Corpus (60 Cases):** Off-chain Attack Success Rate (ASR) is **0/30 (0.0%)** under AHEM-G; False Block Rate (FBR) is **25/30 (83.3%)** under AHEM-G (actionable FBR: **25/28 (89.3%)**).

### 5. AI Planner & Grammar Gate Host Benchmarks (`evidence/planner/host_results.json`)
Evaluated across a 200-sample adversarial suite including direct prompt injections, base64 encodings, and Morse code:
* **AHEM-G-01 (Grammar Gating Enforced):** **200/200 legal outputs**, **0 illegal outputs**, 46 token-level vetoes logged, 1,000 forced token skeletons emitted.
* **AHEM-G-02 (Unconstrained Plain Baseline):** 48 final invalid outputs (aborts) due to malformed JSON or unconstrained text.
* **AHEM-G-06 (Latency):** Median P50 latency of **116.62 ms** (AHEM-G) vs 139.96 ms (Plain baseline).

---

## 7. Security, Residual Risks & Engineering Boundaries

In accordance with strict technical standards, Fusion Wallet explicitly discloses its architectural boundaries:
* **Testnet Prototype:** Deployed on Base Sepolia using testnet assets and mock contracts. Not evaluated for mainnet custody.
* **Unaudited Contracts:** The smart contract code has not undergone external commercial audit.
* **Simulated Demo Parameters:** Payee cooling delay is configured to 3,600 seconds (1 hour) for demonstration purposes; production default is 86,400 seconds (24 hours). Sanctions screening utilizes an illustrative on-chain test oracle (`0xd2716e08BCe6eb5B1f01329FF5E1a2dd0468B678`).
* **Known Residuals:**
  * **A12:** Dedicated x402 budget signer is not implemented in current core.
  * **A17:** Vault code-hash pinning validates proxy `extcodehash`, but does not detect underlying proxy implementation swaps without external oracle monitoring.
  * **A13:** Durable write-ahead crash reconciliation journal is not implemented.
  * **High FBR (83.3%):** Under AHEM-G, the compact 230M model conservatively defaults to HOLD on complex benign opportunities. Funds remain safe; owners can pay directly via `ownerPayObligation` (card K14 is actively optimizing).

---

## 8. License
Dual-licensed under Apache-2.0 or MIT.
