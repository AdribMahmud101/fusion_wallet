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

### Deployed Contracts Manifest (`deployments/84532.json`)

- **Deployment Status:** `SETUP_COMPLETE`
- **Deployment Profile:** `FULL`
- **Gnosis Safe Threshold:** 2-of-2
- **Safe Owners:** `0x61dd06cE2f07d3E4F97d097741b0D7FD50F97F79, 0x208Bfee9F222d611b4Db71d5ee53b3dAB1A2188A`
- **Payee Cooling Delay:** 3600 s (demo setting; production default 24 h)

| Contract Name | Deployed Address | Deployment Block | Deployment Tx Hash | Explorer Link |
|---|---|---|---|---|
| **safe** | [`0x02986991...`](https://sepolia.basescan.org/address/0x029869910714D9e09b854a2688311AFd42fBb44E) | 48317703 | [`0x957ef32e...`](https://sepolia.basescan.org/tx/0x957ef32ef5e4244b6c76ec20c6aa857b2ca57d2402ed1176eecf4bafcccb9e7b) | [BaseScan](https://sepolia.basescan.org/address/0x029869910714D9e09b854a2688311AFd42fBb44E) |
| **asset** | [`0xba50Cd2A...`](https://sepolia.basescan.org/address/0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f) |
| **vaultA** | [`0xf430cb6E...`](https://sepolia.basescan.org/address/0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a) |
| **vaultB** | [`0xf052C0ED...`](https://sepolia.basescan.org/address/0xf052C0ED8404adEaAEA773A5F8Ae4628134aff6D) | 48317704 | [`0xe706e189...`](https://sepolia.basescan.org/tx/0xe706e18973fb4aeee32416e1430f149ebbcc2154e6fddd3fb82338588bbedc8d) | [BaseScan](https://sepolia.basescan.org/address/0xf052C0ED8404adEaAEA773A5F8Ae4628134aff6D) |
| **sanctionsList** | [`0xd2716e08...`](https://sepolia.basescan.org/address/0xd2716e08BCe6eb5B1f01329FF5E1a2dd0468B678) | 48317705 | [`0x2b28ba96...`](https://sepolia.basescan.org/tx/0x2b28ba967cca84f05d803b2528c097e8af5ff8ea88dd50472008eff3c455de98) | [BaseScan](https://sepolia.basescan.org/address/0xd2716e08BCe6eb5B1f01329FF5E1a2dd0468B678) |
| **module** | [`0xA560ec10...`](https://sepolia.basescan.org/address/0xA560ec10259657E9bADd297E68286d2838Cd5a8f) | 48317706 | [`0x58f90dc5...`](https://sepolia.basescan.org/tx/0x58f90dc5df9e8734ee195bfd672d6adda0a4e8bcb656813e53f534c94501e4aa) | [BaseScan](https://sepolia.basescan.org/address/0xA560ec10259657E9bADd297E68286d2838Cd5a8f) |
| **safeSingleton** | [`0x29fcB43b...`](https://sepolia.basescan.org/address/0x29fcB43b46531BcA003ddC8FCB67FFE91900C762) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0x29fcB43b46531BcA003ddC8FCB67FFE91900C762) |
| **safeProxyFactory** | [`0x4e1DCf7A...`](https://sepolia.basescan.org/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67) |
| **safeFallbackHandler** | [`0xfd0732Dc...`](https://sepolia.basescan.org/address/0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99) |
| **multiSendCallOnly** | [`0x9641d764...`](https://sepolia.basescan.org/address/0x9641d764fc13c8B624c04430C7356C1C7C8102e2) | N/A | N/A | [BaseScan](https://sepolia.basescan.org/address/0x9641d764fc13c8B624c04430C7356C1C7C8102e2) |
| **policyLibrary** | [`0x7ee72192...`](https://sepolia.basescan.org/address/0x7ee721922dffe2bee9ec8e4ef51646110e6cbc1f) | 48317685 | [`0x53b60136...`](https://sepolia.basescan.org/tx/0x53b601369fe5105096d821448d7ae20415c9dca9baf811dbb5088ca427008c8b) | [BaseScan](https://sepolia.basescan.org/address/0x7ee721922dffe2bee9ec8e4ef51646110e6cbc1f) |
| **payeeRegistry** | [`0xD798F4DF...`](https://sepolia.basescan.org/address/0xD798F4DF0dEf13971870AEfEE7fce4691F739B5b) | 48317706 | [`0x58f90dc5...`](https://sepolia.basescan.org/tx/0x58f90dc5df9e8734ee195bfd672d6adda0a4e8bcb656813e53f534c94501e4aa) | [BaseScan](https://sepolia.basescan.org/address/0xD798F4DF0dEf13971870AEfEE7fce4691F739B5b) |

### Golden-Path Execution Transactions (`evidence/base-sepolia.json`)

| ID | Scenario / What It Proves | Kind | Transaction Hash | Block | Status | Source | Decision Hash |
|---|---|---|---|---|---|---|---|
| **E1** | Official Safe v1.4.1, two owners, threshold 2 | `SAFE_CREATION` | [`0x957ef32e...`](https://sepolia.basescan.org/tx/0x957ef32ef5e4244b6c76ec20c6aa857b2ca57d2402ed1176eecf4bafcccb9e7b) | 48317703 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E2** | D17 threshold-two owner setup | `MODULE_ENABLE` | [`0xd8d145e6...`](https://sepolia.basescan.org/tx/0xd8d145e6795471b8fcc68c98c3b1e3763e5e95cbb383140fbcc04c256065e299) | 48317802 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E3** | D17 threshold-two owner setup | `POLICY_CONFIGURED` | [`0xb8927af2...`](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) | 48317803 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E5** | D17 threshold-two owner setup | `OBLIGATIONS_COMMITTED` | [`0xb8927af2...`](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) | 48317803 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E6** | D17 threshold-two owner setup | `SESSION_STARTED` | [`0xb8927af2...`](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) | 48317803 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E6** | D17 threshold-two owner setup | `RESUMED` | [`0xb8927af2...`](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) | 48317803 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E4** | Payee cooling started (3600-second demo setting) | `PAYEE_ADDED` | [`0x8c2a208f...`](https://sepolia.basescan.org/tx/0x8c2a208fed80010e4dfdeaf227f344d80d4411ee1669389abfaabd9d1a4e6ad0) | 48317804 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E4** | Payee cooling started (3600-second demo setting) | `PAYEE_ADDED` | [`0xabb5bd79...`](https://sepolia.basescan.org/tx/0xabb5bd793393a221a72d773a8b65708d65ae7cc0be1a3064fe98ed088adf4f94) | 48317805 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E9** | K2 operational script; deterministic, Linux host | `ACTION_APPROVED` | [`0x4fe8a794...`](https://sepolia.basescan.org/tx/0x4fe8a794c15a91bf70e841549b2bf1907290100ee46f6a1abfa98cdc1221b603) | 48317855 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E10** | K2 operational script; deterministic, Linux host | `DEPOSIT` | [`0x35ff759c...`](https://sepolia.basescan.org/tx/0x35ff759c17bc48442a1d25b02a75fb66aac2307502cd0402e36b40916d65d726) | 48317876 | SUCCESS (1) | SCRIPT (deterministic) | `0x5eb5af80...` |
| **E16** | Cooling payee rejected | `INTENTIONAL_REVERT` | [`0x8f9dde94...`](https://sepolia.basescan.org/tx/0x8f9dde942e571fad7b19c7e1f7a75564a80d18a893731539438a817100d30a70) | 48317882 | REVERT (0) | SCRIPT (deterministic) | `0x400877f5...` |
| **E16** | Delayed action cannot bypass queue | `INTENTIONAL_REVERT` | [`0xe1d5dba1...`](https://sepolia.basescan.org/tx/0xe1d5dba152fb9a183192794e37bd17c98fe5530e1fcb937970ef2739904baa17) | 48317887 | REVERT (0) | SCRIPT (deterministic) | `0xb80fcb67...` |
| **FUNDING** | Explicit threshold-two owner allocation; not agent execution | `OWNER_ALLOCATION` | [`0xfd7cd40d...`](https://sepolia.basescan.org/tx/0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba) | 48318006 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **FUNDING** | Explicit threshold-two owner allocation; not agent execution | `OWNER_ALLOCATION` | [`0xfd7cd40d...`](https://sepolia.basescan.org/tx/0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba) | 48318006 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E16** | Reserve raid rejected before queue creation | `INTENTIONAL_REVERT` | [`0x55d44a2b...`](https://sepolia.basescan.org/tx/0x55d44a2b3f7dedab3d898528a14cdd66aefaa49cf1eaa658b84dd3b777505d27) | 48318011 | REVERT (0) | SCRIPT (deterministic) | `0x3ab47bb5...` |
| **E7** | K2 operational script; deterministic, Linux host | `PAY_INSTANT` | [`0x3d5682af...`](https://sepolia.basescan.org/tx/0x3d5682afb067993bf04ae88a344eff6d642138062f56e8539d79ed975cffe6c7) | 48319621 | SUCCESS (1) | SCRIPT (deterministic) | `0xf3d8ba5c...` |
| **E8** | K2 operational script; deterministic, Linux host | `ACTION_QUEUED` | [`0x01316eae...`](https://sepolia.basescan.org/tx/0x01316eaeebf051827e46b0fa61645376ea531342cb6d526e4252acc6b0db202a) | 48319637 | SUCCESS (1) | SCRIPT (deterministic) | `0x808bab5a...` |
| **E8** | K2 operational script; deterministic, Linux host | `PAY_DELAYED` | [`0xc63f9d98...`](https://sepolia.basescan.org/tx/0xc63f9d98e7a2d28ebe88eea2ffcec9288a39c4be0aa218da1acb584670b1fe7f) | 48319745 | SUCCESS (1) | SCRIPT (deterministic) | `0x808bab5a...` |
| **E8** | K2 operational script; deterministic, Linux host | `ACTION_QUEUED` | [`0xdac59209...`](https://sepolia.basescan.org/tx/0xdac5920996832c73375b584905d34cf430f35ad548ee1368a7695c927736548c) | 48319764 | SUCCESS (1) | SCRIPT (deterministic) | `0xa1f706ae...` |
| **E12** | K2 operational script; deterministic, Linux host | `QUEUE_CANCELLED` | [`0x7e069c86...`](https://sepolia.basescan.org/tx/0x7e069c86c94e06fda563190567876b467f0b2facf9189ebb1a0939253182c4b5) | 48319776 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E16** | Duplicate committed bill payment rejected | `INTENTIONAL_REVERT` | [`0x5af192e0...`](https://sepolia.basescan.org/tx/0x5af192e04e2ac730bd3eebc0a4f7f76d9ee76525896d40b0c9084f84d98e96b4) | 48319779 | REVERT (0) | SCRIPT (deterministic) | `0x7770f965...` |
| **E9** | K2 operational script; deterministic, Linux host | `ACTION_APPROVED` | [`0xddbc47e9...`](https://sepolia.basescan.org/tx/0xddbc47e9037a49a0db323136d79360a9bd89f7e8220fb32f1ed987e4ef2601e0) | 48319789 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E9** | K2 operational script; deterministic, Linux host | `PAY_OWNER_TIER` | [`0x97e5cd9c...`](https://sepolia.basescan.org/tx/0x97e5cd9c1c5bc351b32d2819e6735ef161fda91d3ab6285ccd4a06aec3ba232d) | 48319807 | SUCCESS (1) | SCRIPT (deterministic) | `0x2500ca47...` |
| **E13** | K2 operational script; deterministic, Linux host | `OWNER_PAY` | [`0x46470601...`](https://sepolia.basescan.org/tx/0x46470601cddedd6214792c494472affad03e5457e246bb7b68e35f38cf422826) | 48319821 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E4** | K2 operational script; deterministic, Linux host | `PAYEE_NEW_BANK` | [`0x8c7216a9...`](https://sepolia.basescan.org/tx/0x8c7216a9f5b69f9e7cdde1bc7085051510fda7a05f47c90574acaa82751597b3) | 48319832 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E16** | Changed bank details require fresh cooling | `INTENTIONAL_REVERT` | [`0xc04b77c2...`](https://sepolia.basescan.org/tx/0xc04b77c2ffe8d0d945c1cc13aa70d3e34827a85b8af3629b2a2575e30f775d02) | 48319835 | REVERT (0) | SCRIPT (deterministic) | `0x1b732b11...` |
| **E11** | K2 operational script; deterministic, Linux host | `WITHDRAW` | [`0x06530794...`](https://sepolia.basescan.org/tx/0x06530794bc195da12a84ee3ae4538ed1cca3b91f727f2ba4eaeb87f76da67950) | 48319856 | SUCCESS (1) | SCRIPT (deterministic) | `0x5ebedc47...` |
| **E14** | K2 operational script; deterministic, Linux host | `GUARDIAN_PAUSE` | [`0xe33f6e10...`](https://sepolia.basescan.org/tx/0xe33f6e100a88d1008bcb226302ec2e90c147929b1ae71167c6ba134059d7c61c) | 48319867 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E6** | K2 operational script; deterministic, Linux host | `OWNER_RESUME` | [`0x4e097ab5...`](https://sepolia.basescan.org/tx/0x4e097ab5cf673de69cd1e99283424b4935a9cc7ac536e4746b2889057bf43cf7) | 48319881 | SUCCESS (1) | SCRIPT (deterministic) | - |
| **E15** | K2 operational script; deterministic, Linux host | `EMERGENCY_UNWIND` | [`0xbcbc2d8a...`](https://sepolia.basescan.org/tx/0xbcbc2d8a0e6e460d2a19319f59a9a05ca4685ac5b284c105e75ef15e58518759) | 48319899 | SUCCESS (1) | SCRIPT (deterministic) | - |

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

