# Fusion Wallet — Presentation Deck Outline & Speaker Notes

**Competition Track:** FinTechathon 2026 International Track — Topic F: Agent Wallet  
**Target Duration:** ≤ 10 minutes (10 slides, ~1 minute per slide)  
**Target Audience:** WeBank industry experts and Tsinghua PBCSF academic scholars  

---

## Slide 1: The Problem — The Vulnerability of Autonomous Financial Agents

### Visual Layout
* Header: *Autonomous AI Agents Need Cryptographic Guardrails, Not Unbounded Authority*
* Three Problem Cards with empirical industry citations:
  1. **Invoice Fraud & BEC:** FBI IC3 2025 Report: ~24,768 complaints, ~$3.05B in reported financial losses (`system architecture/fusion_wallet_architecture.md` §2.2).
  2. **Address Poisoning & Vanity Confusion:** High-profile losses of ~$50M (Dec 2025) and ~$12.25M (Jan 2026).
  3. **AI Jailbreaks & Prompt Injection:** May 2026 Grok/Bankr incident: a Morse-code prompt injection manipulated an AI wallet agent into draining ~$150,000–$200,000.
* Bottom Callout: *Operating cash sits idle earning 0%, but naive automated sweeps risk liquidating funds required for imminent payroll.*

### Speaker Notes (0:00 – 1:00)
> "Distinguished judges from WeBank and Tsinghua PBCSF, good morning. 
> Autonomous AI agents are rapidly entering decentralized finance, but today’s AI wallets suffer from a fundamental architectural flaw: they give probabilistic language models direct authority to construct and sign transactions.
> As the FBI IC3 2025 report demonstrates, business email compromise and invoice manipulation account for over $3 billion in annual losses. In Web3, address poisoning drains tens of millions. And just five months ago in May 2026, the Grok/Bankr incident proved that an agent with open transfer capabilities can be talked into draining $150,000 through simple Morse-code prompt injection.
> If an AI agent can write an arbitrary recipient address or an arbitrary amount, it will eventually be exploited. Fusion Wallet solves this problem by design: the AI can choose, but it can never write."

---

## Slide 2: Persona & Jobs to be Done

### Visual Layout
* Header: *Target Persona: Cross-Border SME Treasury Management*
* Persona Card: **Mei (Owner)** & **Wei (Finance Manager)** — 20-person software agency earning USDC from overseas clients, paying contractors, SaaS vendors, and payroll in USDC. No dedicated treasury team.
* Core Jobs Table:
  * **J1: Pay Committed Bills:** Pay verified vendors on time, in the exact committed amount (`dueAt - payEarlyWindow`).
  * **J2: Guard Operational Reserves:** Ensure $C - \text{assets} \ge R + B$. Cash owed ($R$) and buffer ($B$) are untouchable.
  * **J3: Sweep True Surplus:** Deploy unencumbered capital into Aave V3 waUSDC when net yield exceeds transaction costs.
  * **J4: Complete Explainability:** Output structured audit passports with on-chain cryptographic `decisionHash` anchoring.
  * **J5: Instant Risk Mitigation:** One-tap guardian pause and emergency asset unwinding.

### Speaker Notes (1:00 – 2:00)
> "Consider our persona: Mei and Wei run a 20-person cross-border software studio. They receive payments in USDC and pay contractors worldwide. Last year, a spoofed 'updated bank details' email nearly cost them a month of payroll.
> Mei and Wei need automation, but they cannot afford existential risk.
> Fusion Wallet automates five core jobs: paying committed bills exactly on time; mathematically reserving cash owed for invoices and payroll before investing a single dollar; deploying true surplus into Aave V3; generating an auditable on-chain proof for every decision; and providing an instantaneous kill switch if market conditions change."

---

## Slide 3: Fusion Wallet in One Sentence & User Interface

### Visual Layout
* Hero Quote:
  > **"Fusion Wallet is an AI treasury agent for small businesses. It pays the bills you approved, to vendors you verified, in the exact amount, and invests only the money you don't owe. The AI runs on your phone and can't go beyond the limits enforced on-chain."**
* Screenshot / Mockup: *Treasury Dashboard*
  * Prominent **TESTNET** indicator (Base Sepolia).
  * Asset Breakdown: Liquid Cash ($C$), Reserved Obligations ($R$), Safety Cushion ($B$), Sweepable Surplus.
  * Active Session Status & 24h Outflow Velocity Gauge.
  * Bilingual EN / 中文 interface toggle.

### Speaker Notes (2:00 – 3:00)
> "Here is our product thesis in one sentence: Fusion Wallet pays the bills you approved, to vendors you verified, in the exact amount, and invests only what you don't owe.
> On the screen, you see our Treasury tab running against live Base Sepolia testnet contracts.
> Notice the accounting clarity: cash is strictly partitioned into committed liabilities, operating buffer, and sweepable surplus. The AI agent cannot treat committed invoice money as deployable capital. 
> Furthermore, for our evaluation panel today, the entire mobile interface supports real-time bilingual switching between English and Chinese."

---

## Slide 4: Authority Split & Permission Tiers

### Visual Layout
* Header: *Separation of Duties & Tiered Velocity Limits*
* Roles Matrix:
  * **Safe 2-of-2 (Mei & Wei):** Policy updates, bill commitments, session creation, payee approvals.
  * **Agent Session Key:** Policy-bounded autonomous execution; strictly capped lifetime (≤ 24h) and budget.
  * **Guardian Key / Single Owner:** Asymmetric power: can pause, cancel queues, and unwind, but can *never* move funds out.
  * **Auditor Role:** Zero signing capability; read-only verification mode.
* Velocity Tier Table (`fusion_wallet_architecture.md` §6.2):
  * **Micro (≤ 10 USDC):** Instant autonomous execution.
  * **Delayed (10 to 100 USDC):** Mandatory 180s on-chain queue with guardian cancellation window.
  * **Owner Tier (100 to 2,500 USDC):** Requires 2-of-2 Safe multi-signature approval.
  * **Hard Cap (> 2,500 USDC):** Contract-level absolute revert.

### Speaker Notes (3:00 – 4:00)
> "We implement institutional banking controls directly in smart contracts.
> First, segregation of duties: starting a session, committing bills, or loosening limits requires a 2-of-2 multi-signature Safe transaction from both Mei and Wei.
> Second, asymmetric guardian control: any single human—or an automated monitoring service—can pause the wallet or cancel queued actions with a single signature. Stopping requires one person; moving money requires two, or the policy-bound agent.
> Third, transaction velocity tiers: micro payments up to 10 dollars execute instantly. Mid-sized transactions enter an on-chain 180-second delay queue, providing a visible cancellation window. Large payments up to 2,500 dollars require explicit dual-signature authorization. Anything above 2,500 dollars is unconditionally rejected by contract bytecode."

---

## Slide 5: System Architecture & Android Process Isolation

### Visual Layout
* Architecture Diagram (Mermaid rendering of §5.2):
  * Device: `:main` (UI/Worker) $\leftrightarrow$ `:inference` (Isolated, No Internet) $\leftrightarrow$ `:signer` (Keystore).
  * Chain: Base Sepolia $\rightarrow$ Gnosis Safe v1.4.1 $\leftrightarrow$ `KavachiModule` $\leftrightarrow$ Aave waUSDC & Demo Vault.
* Trust Boundary Highlights:
  * `:inference` process holds **zero private keys** and has **no network permissions**.
  * Model prompt contains only display labels (`A`–`H`); no hex addresses or raw integers.
  * Signer process performs independent local `eth_call` delta simulations before signing.

### Speaker Notes (4:00 – 5:00)
> "Turning to system architecture: Fusion Wallet separates execution into three distinct processes on the mobile device.
> The inference process runs our local small language model. Crucially, this process has zero network permissions in AndroidManifest, and holds no cryptographic keys. It literally lacks the OS capability to exfiltrate data or sign transactions.
> The prompt passed to the model never contains Ethereum addresses or raw balances. It receives only pre-generated option labels: Option A, Option B, or Option H to hold.
> Even if the entire application UI and inference process were compromised, transactions must pass through our isolated signer process, which validates the action against active policy and simulates state deltas before producing a signature."

---

## Slide 6: Core Algorithm & Token-Level Grammar Gate (AHEM-G)

### Visual Layout
* Header: *Deterministic Candidate Generation + Decode-Time Grammar Masking*
* Left Column: **Rust Candidate Enumerator (`candidates.rs`)**
  * Evaluates snapshot $S$ and integer economics: $\text{net} = \text{gross} - \text{fee} > \text{minNet}$ (1 USDC).
  * Emits sorted letter candidates ($A$–$G$) and mandatory HOLD ($H$).
* Right Column: **AHEM-G MNN Sampler Enforcement**
  * Sits *inside* the native MNN decoding loop (`register_ahem_gate_callback`).
  * Enforces forced skeleton `{"choice":"` followed by allowed letter token.
* Empirical Results Callout (`evidence/planner/host_results.json`):
  * **200/200 adversarial prompts:** 0 illegal outputs, 46 token vetoes.
  * **Plain MNN baseline:** 48/200 invalid outputs (aborts).
  * **Latency overhead:** P50 116.62 ms (AHEM-G) vs 139.96 ms (Plain baseline).

### Speaker Notes (5:00 – 6:00)
> "How does the agent make decisions?
> Step one: our Rust engine evaluates the on-chain state and deterministically enumerates candidate options. It computes integer economics with a 30-day horizon, asserting that expected net yield strictly exceeds transaction fees.
> Step two: the model chooses. But unlike conventional agents that prompt an LLM to generate function arguments, our AHEM-G grammar gate modifies the model's token sampler at decode time.
> Tokens outside the legal letter choices are physically banned before sampling completes. As proven in our test suite of 200 adversarial prompts, AHEM-G achieved zero illegal outputs with 46 logged token vetoes, whereas the unconstrained baseline suffered 48 complete failures.
> And because AHEM-G forces a tight output skeleton, median inference latency on host hardware is just 116 milliseconds."

---

## Slide 7: Institutional Anti-Fraud Controls

### Visual Layout
* Header: *Four Layers of Payee & Obligation Protection*
* Four Control Pillars:
  1. **Two-Person Bill Commitment:** Safe 2-of-2 commits obligations as a Merkle tree root (`commitObligations`).
  2. **Payee Cooling Period:** New payees require an on-chain cooling delay (`payeeDelaySeconds`: 3,600s demo, 24h production) before becoming `ACTIVE`.
  3. **Double-Hashed Merkle Invoices:** Leaves double-hashed to prevent second-preimage attacks: $\text{keccak256}(\text{bytes.concat}(\text{keccak256}(\dots)))$. Paid status tracked permanently in contract storage.
  4. **Fail-Closed Sanctions Screening:** `isSanctioned` oracle evaluated at payee creation, queueing, and execution.

### Speaker Notes (6:00 – 7:00)
> "To prevent invoice fraud and business email compromise, Fusion Wallet implements banking-grade controls directly on-chain.
> First, bill commitments require dual-signature authorization from both Mei and Wei. Invoices are committed as a cryptographic Merkle root.
> Second, our payee registry enforces a mandatory cooling-off period. If an attacker tricks one owner into registering a new vendor address, the agent cannot send funds to that address until the cooling clock expires—allowing the second owner or guardian time to cancel.
> Third, obligation leaves are double-hashed, and every paid bill is marked permanently in contract storage, mathematically preventing double-payment attacks.
> Finally, sanctions screening fails closed: if the compliance oracle reverts or is unreachable, the transaction aborts."

---

## Slide 8: Verification & Attack Arena Evidence

### Visual Layout
* Header: *Comprehensive Verification Across 157 Contract Tests & 200 Host Scenarios*
* Left Card: **Smart Contract Test Suite (`src/contracts/STATUS.md`)**
  * **157/157 passing Foundry tests** (zero failures, zero skips).
  * Invariant Handler Campaign (I1–I7): 64 runs, 2,048 calls, 0 reverts.
  * Formal Invariants: Reserve preservation ($C \ge R + B$), zero residual allowances, strict nonce monotonicity.
  * Runtime Code Size: `KavachiModule` is **24,301 bytes** (275 bytes below EIP-170 limit of 24,576).
* Right Card: **Adversarial Attack Arena (`evidence/planner/host_results.json`)**
  * Direct prompt injections, Morse code, base64 payload attacks.
  * All 200 adversarial test cases resulted in 0 unauthorized actions.

### Speaker Notes (7:00 – 8:00)
> "We believe security claims are meaningless without verifiable artifacts.
> Our Foundry test suite contains 157 tests, passing with zero failures. Our stateful invariant test campaign executed over 2,000 randomized handler calls without a single assertion violation, verifying seven core invariants including absolute reserve preservation and zero residual token allowances.
> Notice our bytecode size: `KavachiModule` compiles to 24,301 bytes under Cancun via-IR, fitting cleanly within the strict EIP-170 limit.
> In our adversarial attack arena, when subjected to Morse code and encoded prompt injections identical to the Grok/Bankr attack, the system stopped 100% of attempts across our five defensive layers."

---

## Slide 9: On-Chain Decision Passports & 'Verify Us Yourself'

### Visual Layout
* Header: *Cryptographic Auditability: The Decision Passport*
* Diagram: *From Off-Chain Inference to On-Chain Immutability*
  * Canonical Passport JSON (RFC 8785 JCS) $\rightarrow \text{decisionHash} = \text{keccak256}(\text{JCS}) \rightarrow$ `ActionExecuted(..., decisionHash)`.
* "Verify Us Yourself" Terminal Box:
  ```bash
  python3 scripts/verify_evidence.py
  # Re-reads Base Sepolia RPC, validates receipts, re-computes decisionHash
  # Result: ALL CHECKS PASSED
  ```
* BaseScan Links to live deployed contracts on Base Sepolia.

### Speaker Notes (8:00 – 9:00)
> "Transparency requires verification.
> For every autonomous decision, Fusion Wallet generates an RFC 8785 canonical JSON decision passport recording the snapshot block, candidate options, model hash, and AHEM veto metrics.
> The Keccak-256 hash of this passport is passed as `decisionHash` in the transaction itself, permanently emitted in the `ActionExecuted` event.
> Anyone in the world can run our read-only verifier script with one command: `python3 scripts/verify_evidence.py`. The script queries public Base Sepolia RPC nodes, inspects the transaction receipts, re-hashes the published passports, and proves that on-chain execution perfectly matches off-chain intent."

---

## Slide 10: Limitations, Roadmap, & FinTechathon Summary

### Visual Layout
* Header: *Honest Disclosures, Production Roadmap, & Conclusion*
* Transparent Limitations Box:
  * Deployed on Base Sepolia testnet; uses mock USDC and simulated Aave/demo vault rates.
  * Payee cooling set to 3,600s for competition demonstration (production default: 24h).
  * Contracts are unaudited testnet prototypes; no formal regulatory compliance claimed.
  * Host execution demonstrated on Linux aarch64; Android NDK cross-compilation pending.
* Post-Competition Roadmap:
  * Mainnet Safe deployment with hardware wallet signing integration.
  * Production Chainalysis oracle integration.
  * Full Android OS-level multi-APK sandbox separation.
* Summary: *Autonomous treasury management with mathematical guarantees: the AI chooses, but the smart contract governs.*

### Speaker Notes (9:00 – 10:00)
> "To conclude, we present our honest boundaries. Fusion Wallet is an advanced competition prototype deployed on Base Sepolia. Our contracts are unaudited, our payee cooling is set to one hour for live demonstration purposes rather than the 24-hour production default, and our sanctions screening uses a test oracle.
> But the core architectural innovation is proven: by pairing deterministic Rust candidate generation and decode-time AHEM-G grammar gating with an on-chain Safe guard layer, we eliminate the vulnerability of open-ended prompt injection.
> The AI agent can optimize cash management, but it can never breach the mathematical boundaries enforced by the blockchain.
> Thank you, and we welcome your questions."
