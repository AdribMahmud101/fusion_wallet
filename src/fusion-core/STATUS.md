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
- **Status**: COMPLETE
- **Modules**:
  - `src/manifest.rs`: `DeploymentManifest` parse & validation (§10.3), enforcing `chainId ∈ {31337, 84532}`, RPC url validation, and address non-zero assertions.
  - `src/buckets.rs`: Integer reserve accounting (§7.9) computing `free`, `reserved`, `sweepable`, `shortfall`, `deployed`, and `total`.
  - `src/candidates.rs`: Deterministic candidate enumerator (§10.4) and integer measured economics (K44: <=85% utilization, <=14-day gas-adjusted break-even, conservative APY; letter labels A-G with H=HOLD per D14).
  - `src/planner_io.rs`: Prompt facts builder (§10.6, no addresses, hashes, or raw units in model prompts, owner note capped at 200 chars) and strict JSON output validator (`{"choice":"<letter>"}`).
  - `src/passport.rs`: Canonical RFC 8785 JCS JSON serializer and `DecisionHash` generator, reproducing reference passport vectors byte-for-byte.
- **Tests**:
  - `tests/candidates_prop.rs`:
    - `test_enumeration_determinism`
    - `test_bill_ordering_earliest_due_at`
  - `tests/labels.rs`:
    - `test_hold_is_always_label_h`
    - `test_prompt_facts_contain_no_addresses_or_hashes`
  - Unit tests in `src/manifest.rs`, `src/buckets.rs`, `src/planner_io.rs`, `src/passport.rs`.

## Priority 3: Preview, Calldata Encoding, UniFFI Surface
- **Status**: COMPLETE
- **Modules**:
  - `src/preview.rs`: Pure mirror of §7.7 normative rules in exact order (Rules 1–7, `PreviewResult`, `RuleOutcome`).
  - `src/encode.rs`: Calldata encoders using `alloy::sol!` for `execute`, `queueAction`, `cancelQueuedAction`, `pause`, `emergencyUnwind`, `startSession`, `resume`, `commitObligations`, and Safe `execTransaction`.
  - `src/lib.rs`: Complete UniFFI export surface (Appendix B §25) with `panic_guard` on every function returning `{"ok":false,"error":{"code":"PANIC"}}`.
- **Tests**:
  - `tests/preview_parity.rs`:
    - `test_preview_instant_deposit_passes_all_checks`
    - `test_preview_catches_paused_module`
    - `test_preview_catches_bad_nonce`
    - `test_preview_catches_missing_decision_hash`
    - `test_preview_catches_reserve_raid`
  - Unit tests in `src/preview.rs` and `src/encode.rs`.

## Priority 4: Snapshot, Signing, Agent Runner CLI
- **Status**: COMPLETE
- **Modules**:
  - `src/snapshot.rs`: One-block state snapshot reads via alloy and Multicall3 (§10.5), pinned to block number/hash/timestamp, decoding Safe cash, vault shares/assets, and module policy/state/queue.
  - `src/sign.rs`: EIP-1559 transaction construction and signing with zeroized secp256k1 keys (`Zeroizing<[u8; 32]>`) (§10.2, §11.3). Private keys never cross FFI or leak into logs.
  - `src/bin/agent-runner.rs`: Autonomous CLI agent runner per §10.2 and §19.5 (snapshot → candidates → planner → passport → preview → sign → broadcast → reconcile; produces evidence rows).
- **Tests**:
  - `src/sign.rs` tests:
    - `test_sign_eip1559_deterministic`
    - `test_sign_prepared_json`
  - `src/snapshot.rs` tests:
    - `test_convert_policy_mapping`
    - `test_convert_policy_invalid_tier`
    - `test_convert_module_state_mapping`
  - `src/bin/agent-runner.rs`:
    - CLI `--help` invocation
    - End-to-end `--dry-run` simulation over snapshot fixtures producing signed EIP-1559 transaction, JCS decision passport, and evidence row

