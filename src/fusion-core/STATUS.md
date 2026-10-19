# fusion-core v4 Migration & Implementation Status

## Priority 1: Amounts, Types, Hashing, and Merkle Trees
- **Status**: COMPLETE
- **Modules**:
  - `src/amount.rs`: `RawAmount(U256)` with checked arithmetic, decimal parsing/formatting (6 dp, USDC standard), base-10 raw serialization/deserialization per §9. Zero floating-point money.
  - `src/types.rs`: `PermissionTier`, `ActionKind`, `VaultLimit`, `Policy`, `Action`, `Payee`, `Queue`, `ModuleState`, `Obligation`, `Snapshot` DTOs.
  - `src/hashing.rs`: Exact implementation of §9 hashing contract (`TAG_*`, `PolicyHash` with sorted vault limits, `SessionId`, `MandateHash`, `ActionDigest`, `CandidateId` with zeroed decision hash, `ObligationLeaf` double hashing, `DecisionHash`).
  - `src/merkle.rs`: OpenZeppelin-compatible `StandardMerkleTree` complete binary tree, sorted-pair hashing, proof generation, and verification.
- **Tests**:
  - `tests/vectors.rs`:
    - `test_vector_constants`
    - `test_vector_policy_hash`
    - `test_vector_session_id`
    - `test_vector_mandate_hash`
    - `test_vector_actions`
    - `test_vector_candidate_id`
    - `test_vector_commitments_empty`
    - `test_vector_commitments_single_leaf`
    - `test_vector_commitments_three_leaves`
    - `test_vector_decision_hash`
  - `tests/merkle.rs`:
    - `test_empty_merkle_tree`
    - `test_single_leaf`
    - `test_two_leaves`
    - `test_many_leaves_and_tamper_detection`
  - Unit tests in `src/amount.rs` and `src/merkle.rs`.

## Priority 2: Manifest, Buckets, Candidates, Planner I/O, Passport
- **Status**: IN PROGRESS

## Priority 3: Preview, Calldata Encoding, UniFFI Surface
- **Status**: PENDING

## Priority 4: Snapshot, Signing, Agent Runner CLI
- **Status**: PENDING
