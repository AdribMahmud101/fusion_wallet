# Fusion Wallet — Execution Evidence & Empirical Validation

> **FinTechathon 2026 International Track — Topic F: Agent Wallet**  
> This document compiles empirical validation records across on-chain smart contracts, 
> on-device AI inference gates (AHEM-G), and adversarial attack arena evaluations.
> *Generated automatically by `docs/submission/gen_evidence.py`.*

---

## 1. On-Device AI Planner & AHEM-G Evaluation (`evidence/planner/host_results.json`)

- **Evaluated Model:** LFM2.5-230M-MNN
- **Runtime Host Architecture:** `aarch64` (linux-host)
- **Git Commit (Runner):** `8a46e07cf02e1acd98635011397db88e2379fca3`
- **Corpus SHA-256:** `4e7977e5d1503dd0a9dd0fc34a8940f8bc69374b4d0312a046882af0027209e0`
- **Evaluation Methodology:** Sequential full request timings; plain may include one validation retry. Cold load excluded; no warmup discarded.

### Empirical Acceptance Table (AHEM Family)

| Test ID | Focus Area | Status | Evaluated Samples | Illegal Outputs | Vetoes Logged | Latency P50 / P95 | Evidence Source |
|---|---|---|---|---|---|---|---|
| **AHEM-G-01** | Adversarial Prompts under AHEM-G Gate | **PASS** | 200 | **0** | 46 | 116.62 ms / 117.70 ms | `host_results.json:AHEM-G-01` |
| **AHEM-G-02** | Plain MNN Baseline (Without AHEM-G) | **PASS** | 200 | **48 invalid** (49 1st-try) | 0 (No gate) | 139.96 ms / 145.03 ms | `host_results.json:AHEM-G-02` |
| **AHEM-G-03** | Gate Lifecycle & Symbol Verification | **PASS** | Self-test | 0 | 0 | Included in selftest | `host_results.json:AHEM-G-03` |
| **AHEM-G-04** | Sequential State Isolation (Back-to-back runs) | **PASS** | Self-test | 0 | 0 | Included in selftest | `host_results.json:AHEM-G-04` |
| **AHEM-G-05** | Forced Retry Exhaustion (All-letter ban) | **PASS** | Self-test | 0 | 9 (banned all) | Aborted clean -> HOLD | `host_results.json:AHEM-G-05` |
| **AHEM-G-06** | Latency Overhead Benchmark (20 sequential runs) | **PASS** | 20 / mode | 0 | - | AHEM-G: 116.62 ms vs Plain: 139.96 ms | `host_results.json:AHEM-G-06` |
| **AHEM-G-07** | On-Device Mobile Execution (Android arm64) | **NOT_RUN** | - | - | - | - | Phone cross-build out of scope; no NDK/cargo-ndk. NEXT: arm64 Android build and ten device runs. |

### Key Takeaways from Empirical Planner Benchmarking:
1. **Zero Injection Success:** Under AHEM-G, 200 adversarial prompts yielded exactly 0 illegal outputs and 0 aborts. The gate logged 46 token-level vetoes and injected 1,000 forced skeleton tokens.
2. **Baseline Vulnerability:** Without AHEM-G, the stock MNN runtime produced 49 first-attempt invalid outputs and 48 terminal aborts (24% failure rate) when subjected to the identical prompt suite.
3. **Tokenizer Letter Mapping:** Single-token vocabulary confirmation directly after `"`: {'A': 542, 'B': 543, 'C': 544, 'D': 545, 'E': 546, 'F': 547, 'G': 548, 'H': 549}.

---

## 2. Smart Contract Verification & Invariant Proofs (`src/contracts/STATUS.md`)

- **Toolchain:** Foundry 1.8.5; Solidity `0.8.24`; Cancun EVM; `via_ir = true`; 200 optimizer runs.
- **Unit & Integration Suite:** **157 passing tests**, 0 failures, 0 skipped (`size-report.txt` & `test-report.txt`).
- **Stateful Invariant Campaign (I1–I7):** 64 runs, 32 depth (2,048 handler calls), 0 reverts. CI campaign: 256 runs, 64 depth (16,384 handler calls), 0 reverts.
- **Reproducible Invariant Seed:** `0x20261019`.
- **Bytecode Size Limits (EIP-170):**
  - `KavachiModule` runtime size: **24,301 bytes** (275 bytes below the 24,576 byte limit).
  - `KavachiPolicyLib` runtime size: **5,854 bytes**.
- **Transaction Gas Overhead:**
  - `execute(PAY_OBLIGATION)` under official Safe v1.4.1 proxy: **292,385 gas**.
  - `emergencyUnwind` for both vaults: completes well within the 1,000,000 gas ceiling on Anvil.
- **Frozen ABI & Vector Parity:** `python3 script/check_abi.py` confirms all 93 frozen entries match verbatim.

### Invariant Test Verification Summary (I1–I7)

| Invariant ID | Formal Property Checked | Test Name | Result |
|---|---|---|---|
| **I1** | Reserve preservation: $C \ge R + B$ holds across all deposits; underfunded state blocks investing | `invariant_I1_reservePreservedOnInvestment` | **PASS** (2,048 calls) |
| **I2** | Destination whitelist: Treasury assets leave Safe only to Vault A, Vault B, or active non-sanctioned payees | `invariant_I2_onlyTrustedDestinations` | **PASS** (2,048 calls) |
| **I3** | Gross limits: Daily gross $\le 4,000$ USDC and session gross $\le 8,000$ USDC | `invariant_I3_grossDailyAndSessionLimits` | **PASS** (2,048 calls) |
| **I4** | Unique payment: Each obligation is paid at most once across agent and owner paths | `invariant_I4_paidAtMostOnce` | **PASS** (2,048 calls) |
| **I5** | Allowance reset: Safe-to-vault allowances equal exactly zero at transaction termination | `invariant_I5_noResidualAllowances` | **PASS** (2,048 calls) |
| **I6** | Pause integrity: While paused, all agent actions revert; only Safe 2-of-2 can resume | `invariant_I6_pauseStopsAgentAndOnlySafeResumes` | **PASS** (2,048 calls) |
| **I7** | Nonce monotonicity: Expected nonce increments strictly; cancelled/expired queues consume nonce | `invariant_I7_nonceConsumptionAndDigestUniqueness` | **PASS** (2,048 calls) |

---

## 3. On-Chain Deployment & Execution Receipts (Base Sepolia, Chain ID 84532)

### [PENDING: Base Sepolia Deployment in Progress on Branch `card/K2`]

The deployment manifest `deployments/84532.json` and on-chain transaction receipts `evidence/base-sepolia.json` have not yet landed on this branch.
The K2 track is deploying the full Aave profile to Base Sepolia, conducting initial payee cooling, and recording golden-path execution receipts.
Once card K2 merges or files land, re-running `python3 docs/submission/gen_evidence.py` will automatically populate this section with live BaseScan transaction links.

---

## 4. Independent Verification Tooling (`scripts/verify_evidence.py`)

Judges and independent auditors can verify all on-chain receipts and passport bindings without keys or authentication:

```bash
# Run read-only verification against Base Sepolia RPC
python3 scripts/verify_evidence.py
```

The verification script performs:
1. Connects to the public Base Sepolia JSON-RPC node (`https://sepolia.base.org`).
2. Verifies code hashes of deployed contracts against `deployments/84532.json`.
3. Re-reads transaction receipts, confirming execution status = 1 and asserted event signatures.
4. For every agent transaction, computes the RFC 8785 canonical JSON hash of the published decision passport and verifies equality with the on-chain `decisionHash` emitted in event logs.

