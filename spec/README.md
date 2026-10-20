# Kavachi V4.1 Specification & Contract Interface Freeze

This directory contains the frozen on-chain specification, ABI artifacts, reference hashing library, and cryptographic test vectors for **Kavachi V4.1** (Fusion Wallet, FinTechathon 2026 Topic F).

All downstream implementations—**K1 (Solidity contracts)**, **K3 (fusion-core in Rust)**, and **K4/K5 (Android/Kotlin)**—must code against these exact files and reproduce these digests byte-for-byte.

---

## What Is Frozen

1. **Solidity Specification Interfaces (`spec/abi/`)**
   - [`IKavachiModule.json`](spec/abi/IKavachiModule.json): Complete ABI specification for the Kavachi V4.1 smart contract module. Generated verbatim from §24 Appendix A.
   - [`ISafeMinimal.json`](spec/abi/ISafeMinimal.json): Minimal Safe v1.4.1 interface utilized by the module.
   - [`ISanctionsList.json`](spec/abi/ISanctionsList.json): Sanctions screening oracle interface.

2. **Reference Hashing Library (`spec/reference/src/HashV41.sol`)**
   - Reference Solidity implementation of every formula in §9:
     - `TAG` domain separation strings: `KAVACHI_POLICY_V4_1`, `KAVACHI_SESSION_V4_1`, `KAVACHI_MANDATE_V4_1`, `KAVACHI_ACTION_V4_1`, `KAVACHI_CANDIDATE_V4_1`.
     - `PolicyHash` and address-sorted `vaultLimitsHash`.
     - `SessionId` (with session) and null session handling (`bytes32(0)`).
     - `MandateHash` binding policy, obligations root, session, and pause state.
     - `ActionDigest` encoding all 14 Action struct fields inline.
     - `CandidateId` binding block hash and pre-decision action digest (`decisionHash = 0`).
     - `ObligationLeaf` double-hashing via `bytes.concat` to prevent second-preimage attacks.
     - Sorted-pair Merkle tree root computation and proof verification (OpenZeppelin `StandardMerkleTree` compatible).
     - `DecisionHash` from RFC 8785 JCS canonical passport JSON.

3. **Frozen Test Vectors (`spec/vectors/v4_1.json`)**
   - Machine-checkable test vectors containing both inputs and expected outputs for:
     - Constants (all TAGs and empty obligations root)
     - `PolicyHash`
     - `SessionId` (active session and inactive null session)
     - `MandateHash`
     - `ActionDigest` for all four action kinds (`DEPOSIT`, `WITHDRAW`, `REBALANCE`, `PAY_OBLIGATION`)
     - `CandidateId`
     - Commitments (empty commitment, single-leaf proof, 3-leaf sorted Merkle proof)
     - Decision passport canonical JSON string and `DecisionHash`
   - Strict serialization: `uint256` as base-10 strings, hex lowercase `0x`-prefixed, addresses EIP-55 checksummed.

4. **Architectural Decisions & Rationale (`spec/DECISIONS.md`)**
   - Complete documentation of resolved design questions, citations to §7–§10 and §24, and structural explanations.

---

## Verification & Definition of Done

To verify that the reference implementation, ABIs, and vector definitions remain in complete synchronization:

```bash
cd fusion_wallet && bash spec/check_freeze.sh
```

### What `check_freeze.sh` Does:
1. Rebuilds ABI definitions from `spec/reference/src/IKavachiModule.sol` in an isolated temporary directory and diffs them byte-for-byte against `spec/abi/`.
2. Executes `spec/reference/script/GenVectors.s.sol` to emit vectors into a temporary directory and diffs them byte-for-byte against `spec/vectors/v4_1.json`.
3. Runs the full Foundry test suite (`spec/reference/test/HashV41.t.sol`), testing every vector, commutativity, and Merkle proofs.
4. Exits with code `0` on success, or non-zero on any detected drift.

---

## The Versioning Rule

> **CRITICAL RULE**: 
> The vector file and interface definitions are frozen for Kavachi V4.1.
> **Changing any vector, hash formula, or ABI element requires an explicit specification version bump (e.g. V4.1 -> V4.2).**
> Downstream cards (K1 Solidity, K3 Rust, K4 Kotlin) must NEVER invent fields or alter hashing formulas independently.
