# Kavachi V4.1 — Specification & Architecture Decisions (Card K0)

This document records all architectural and cryptographic clarifications resolved during the Phase-1 freeze of Kavachi V4.1 smart contract interfaces and hashing specifications.

---

## 1. Verbatim Solidity Interface & ABI Generation (§24 Appendix A)

* **Source of Truth**: `system architecture/fusion_wallet_architecture.md` §24 (Appendix A).
* **Decision**: 
  - `spec/reference/src/IKavachiModule.sol` includes all declarations from Appendix A verbatim:
    1. `interface ISanctionsList`
    2. `interface ISafeMinimal`
    3. `interface IKavachiModule` (enums, structs, custom errors, events, view functions, mutative functions).
  - All three interfaces are inspected via `forge inspect <Contract> abi --json` to generate corresponding ABI artifacts in `spec/abi/`:
    - `IKavachiModule.json`
    - `ISafeMinimal.json`
    - `ISanctionsList.json`
  - Toolchain parameters pinned in `spec/reference/foundry.toml`:
    - `solc_version = "0.8.24"`
    - `evm_version = "cancun"`
    - `optimizer = true` (200 runs)
    - `via_ir = true` (per §7.1 and §14)

---

## 2. Cryptographic Domain Separation Tags (§9)

* **Specification**: §9 defines domain separation tags using `TAG(x) = keccak256(UTF8(x))`.
* **Decision**: All five primary tags are defined as immutable compile-time constants in `HashV41.sol`:
  - `TAG_POLICY = keccak256(bytes("KAVACHI_POLICY_V4_1"))` -> `0xf260190c2a965028ff458ec4ffd9891199dfb657e0250e76bf3f68be71243998`
  - `TAG_SESSION = keccak256(bytes("KAVACHI_SESSION_V4_1"))` -> `0xdd4af3d8f3e50bfb9b3c45617fa4ba78937866e38fef3d7c6b304b2ee1800972`
  - `TAG_MANDATE = keccak256(bytes("KAVACHI_MANDATE_V4_1"))` -> `0xd720e98dfe2b80ec41201d4f7d9e0ddfbf35b36460f289c78a31c2fa5128aa6f`
  - `TAG_ACTION = keccak256(bytes("KAVACHI_ACTION_V4_1"))` -> `0xb8178ae129065af1ead4d0d018f19dbc8667a4b7a334449fe4bee8f5a42d8c5e`
  - `TAG_CANDIDATE = keccak256(bytes("KAVACHI_CANDIDATE_V4_1"))` -> `0xb8c00e565ec7fb3feb1955188facef397ec149cc393fc749df462a6f0c4ae9bb`
* **Empty Obligations Root**: Defined in §8.2:
  - `EMPTY_OBLIGATIONS_ROOT = keccak256(bytes("KAVACHI_EMPTY_OBLIGATIONS_V4"))` -> `0x23741c6f1f8567b5bb9370afa7a28d473bc35c8ccc38118af5346e6d25f8fee0`

---

## 3. Vault Limits Array Sorting in `PolicyHash` (§7.4 & §9)

* **Specification**: 
  - §9: `vaultLimitsHash = keccak256(abi.encode((address,uint256)[] sortedVaultLimits))`
  - §7.4: `vaults[] (vault, maxExposureRaw): ⊆ {vaultA, vaultB}, sorted by address, cap > 0`
* **Decision**:
  - `HashV41.computeVaultLimitsHash` and `HashV41.computePolicyHash` explicitly sort the `VaultLimit[]` array ascending by vault address (`uint160(vault)`) in memory before encoding.
  - This ensures that `vaultLimitsHash` is completely deterministic regardless of the order in which vaults were declared in off-chain structures.
  - Both ordered and reversed arrays produce identical hashes in unit tests.

---

## 4. Session State & Null Session ID (§7.5 & §9)

* **Specification**: 
  - §9: `SessionId = keccak256(abi.encode(TAG("KAVACHI_SESSION_V4_1"), uint256 chainId, address safe, address module, uint256 sessionStartEpoch, address sessionKey, uint64 sessionExpiresAt, uint256 sessionLimitRaw))`
  - §9 note: "No session: sessionId = 0, key = 0, expiry = 0, limit = 0."
* **Decision**:
  - When no agent session is active (e.g. `sessionKey == address(0)`), the stored and referenced `sessionId` is explicitly `bytes32(0)`.
  - `HashV41.computeSessionId` returns `bytes32(0)` when `sessionKey == address(0) && sessionExpiresAt == 0 && sessionLimitRaw == 0`.
  - An underlying `computeSessionIdRaw` helper is provided for explicit raw TAG hashing.
  - In `MandateHash`, when no session is active, the session parameter fields are set to null: `sessionId = bytes32(0)`, `sessionKey = address(0)`, `sessionExpiresAt = 0`, `sessionLimitRaw = 0`.

---

## 5. Struct Inlining in `ActionDigest` (§9 & Appendix A)

* **Specification**:
  - §9: `ActionDigest = keccak256(abi.encode(TAG("KAVACHI_ACTION_V4_1"), uint256 chainId, address safe, address module, bytes32 mandateHash, bytes32 sessionId, Action action))`
* **Decision**:
  - `Action` contains only static value types (`uint8`, `address`, `uint256`, `bytes32`, `uint64`).
  - Under Solidity ABI v2 specification, static structs passed directly to `abi.encode` are encoded inline as a continuous tuple without pointer or offset indirection.
  - The parameter sequence encoded into the Keccak-256 buffer is:
    1. `TAG_ACTION` (`bytes32`)
    2. `chainId` (`uint256`)
    3. `safe` (`address`, 32-byte word right-aligned)
    4. `module` (`address`, 32-byte word right-aligned)
    5. `mandateHash` (`bytes32`)
    6. `sessionId` (`bytes32`)
    7. `action.kind` (`uint8`, 32-byte word right-aligned)
    8. `action.sourceVault` (`address`)
    9. `action.destinationVault` (`address`)
    10. `action.assets` (`uint256`)
    11. `action.minSharesOut` (`uint256`)
    12. `action.maxSharesBurn` (`uint256`)
    13. `action.obligationIdHash` (`bytes32`)
    14. `action.payeeIdHash` (`bytes32`)
    15. `action.recipient` (`address`)
    16. `action.dueAt` (`uint64`, 32-byte word right-aligned)
    17. `action.expectedNonce` (`uint256`)
    18. `action.policyEpoch` (`uint256`)
    19. `action.deadline` (`uint64`, 32-byte word right-aligned)
    20. `action.decisionHash` (`bytes32`)
  - Both Rust (`alloy-sol-types`) and Kotlin/JVM can code directly to this exact ABI sequence.

---

## 6. Candidate ID & Breaking Decision Circularity (§7.11, §9 & §10.8)

* **Specification**:
  - §7.11: `decisionHash = keccak256(canonical passport JSON)` is computed before signing and included in `Action`, making it part of `ActionDigest`.
  - §9: `CandidateId = keccak256(abi.encode(TAG("KAVACHI_CANDIDATE_V4_1"), bytes32 snapshotBlockHash, bytes32 actionDigestWithoutDecision))` where `actionDigestWithoutDecision = ActionDigest computed with decisionHash = 0`.
  - §10.8: Passport includes `actionDigest?` with an optional marker.
* **Decision**:
  - Because `ActionDigest` depends on `decisionHash`, the passport cannot contain `actionDigest` prior to its hash being generated without creating a circular cryptographic dependency.
  - The canonical decision passport evaluated for `decisionHash` omits `actionDigest` (or treats it as an unhashed post-decision field).
  - Candidates are enumerated and identified before agent planning; therefore `CandidateId` zeroes out `decisionHash` in the action struct.
  - Cloned memory structs are used in `HashV41.computeCandidateId` to ensure the original `Action.decisionHash` remains unmodified.

---

## 7. Obligation Double Hashing & Merkle Trees (§8.2 & §9)

* **Specification**:
  - §8.2 & §9: `ObligationLeaf = keccak256(bytes.concat(keccak256(abi.encode(bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 amountRaw, uint64 dueAt))))`.
  - §8.2: Compatible with OpenZeppelin `StandardMerkleTree` and `MerkleProof` (sorted pairs). Leaves are sorted by leaf hash.
* **Decision**:
  - Leaf hashing strictly uses the double hash:
    1. Inner: `keccak256(abi.encode(obligationIdHash, payeeIdHash, amountRaw, dueAt))`
    2. Outer: `keccak256(bytes.concat(innerHash))`
  - Sibling pair hashing: `hashPair(a, b)`:
    - If `a < b`: `keccak256(bytes.concat(a, b))`
    - Else: `keccak256(bytes.concat(b, a))`
    - Commutative by definition (`hashPair(a, b) == hashPair(b, a)`).
  - Tree construction: Matches OpenZeppelin `StandardMerkleTree` complete binary tree layout stored as a flat array of size `2 * N - 1`:
    - For `N = 1`: `root = leaf`, `proof = []`.
    - For `N = 3`: Leaves are sorted by leaf hash, paired, and verified against root.

---

## 8. Data Serialization & Transport Standards (§9)

* **Specification**: §9 defines transport formatting rules:
  - `uint256` as base-10 strings without leading zeros.
  - `bytes32` / hex as lowercase `0x`-prefixed 64-character hex strings.
  - `address` formatted with EIP-55 mixed-case checksumming.
  - Passport canonical JSON formatted per RFC 8785 JSON Canonicalization Scheme (JCS).
* **Decision**:
  - `spec/vectors/v4_1.json` strictly adheres to these rules across all inputs and expected outputs.
  - Verified with Foundry's `vm.toString(...)` and automated Python validation.
