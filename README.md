# Fusion Wallet

> **Autonomous AI Treasury Agent for Small Businesses**  
> *FinTechathon 2026 International Track — Topic F: Agent Wallet*  
> Repository: [{{REPO_URL}}]({{REPO_URL}})

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
│   └── planner/                       # host_results.json (200 AHEM-G adversarial evaluations)
├── docs/                              # Auxiliary documentation (UI, assets, accountant)
│   └── submission/                    # Official FinTechathon submission deliverables
│       ├── 01_TECHNICAL.md            # System architecture, core algorithm, security design
│       ├── 02_DECK.md                 # 10-slide outline with speaker notes (≤ 10 min)
│       ├── 03_VIDEO_SCRIPT.md         # 5-minute timed demonstration script and shot list
│       ├── 05_SECURITY_SELF_ASSESSMENT.md # Permission matrix, controls mapping, known limits
│       └── 06_EVIDENCE.md             # Execution table generated from on-chain receipts
└── scripts/                           # Python verification and evidence collection utilities
```

---

## 3. Build, Test, and Deploy Instructions

### 3.1 Smart Contracts (`src/contracts`)
The on-chain guard layer is developed with Foundry (Solidity `0.8.24`, Cancun EVM, `via_ir = true`, 200 optimizer runs).

```bash
# Navigate to contracts directory
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

### 3.2 Pure Rust Engine (`src/fusion-core`)
`fusion-core` implements integer candidate enumeration, preview simulation, Merkle trees, and cryptographic hashing.

```bash
cd src/fusion-core

# Run test suite and vector parity checks
cargo test --locked

# Run the deterministic/host agent runner CLI
cargo run --bin agent-runner -- --help
```

### 3.3 Inference Engine & Planner (`src/fusion-planner`)
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

### 3.4 Verification Tooling (`scripts/verify_evidence.py`)
Fusion Wallet provides a standalone, read-only Python script to independently verify on-chain deployment receipts, Safe module configuration, and decision passport hash bindings against a public Base Sepolia RPC.

```bash
# Verify all on-chain evidence
python3 scripts/verify_evidence.py
```

---

## 4. Test Suite & Verification Summary

### Smart Contract Verification (`src/contracts/STATUS.md`)
* **157/157 passing tests** with zero failures and zero skipped tests.
* **Formal Invariant Campaign (I1–I7):** Verified via `TreasuryHandler` across 64 runs, 2,048 handler calls, and CI runs of 256 runs, 16,384 handler calls with zero reverts:
  * **I1:** Reserve preservation: $C \ge R + B$ holds across all deposit and investment operations.
  * **I2:** Treasury assets can only leave the Safe to Vault A, Vault B, or an active, verified, non-sanctioned payee matching a committed unpaid Merkle leaf.
  * **I3:** Cumulative limits strictly respected (gross daily limit $\le 4,000$ USDC, session limit $\le 8,000$ USDC).
  * **I4:** Invoices paid at most once across both agent and owner execution paths.
  * **I5:** Safe-to-vault ERC-20 allowances reset to exactly zero after every transaction.
  * **I6:** Paused module stops all agent operations; only 2-of-2 Safe transactions can unpause.
  * **I7:** Nonce consumption is strictly monotonic; cancelled/expired queues consume the nonce.

### AI Planner & Grammar Gate Verification (`evidence/planner/host_results.json`)
Evaluated across a 200-sample adversarial suite including direct prompt injections, base64 encodings, and Morse code:
* **AHEM-G-01 (Grammar Gating Enforced):** **200/200 legal outputs**, **0 illegal outputs**, 46 token-level vetoes logged, 1,000 forced token skeletons emitted.
* **AHEM-G-02 (Unconstrained Plain Baseline):** 48 final invalid outputs (aborts) due to malformed JSON or unconstrained text.
* **AHEM-G-06 (Latency):** Median P50 latency of **116.62 ms** (AHEM-G) vs 139.96 ms (Plain baseline).

---

## 5. Security & Engineering Boundaries

In accordance with strict technical standards:
* **Testnet Prototype:** Deployed on Base Sepolia using testnet assets and mock contracts. Not evaluated for mainnet custody.
* **Unaudited Contracts:** The smart contract code has not undergone external commercial audit.
* **Simulated Demo Parameters:** Payee cooling delay is configured to 3,600 seconds (1 hour) for demonstration purposes; production default is 86,400 seconds (24 hours). Sanctions screening utilizes an illustrative on-chain test oracle.
* **Yield Calculations:** Vault yield rates are variable and scenario-based; no yield guarantees are made.
* **Model Decisions:** AHEM-G guarantees that any output produced by the model is syntactically constrained to valid options; it does not claim that the model possesses human financial wisdom.

---

## 6. License
Dual-licensed under Apache-2.0 or MIT.
