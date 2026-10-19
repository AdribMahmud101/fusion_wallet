//! Kavachi V4.1 cryptographic hashing specification (§9).
//!
//! All hashes are Keccak-256 of Solidity `abi.encode` unless specified.
//! Tags are domain-separated using `TAG(x) = keccak256(UTF8(x))`.
//! No floating-point arithmetic or hardcoded test hashes in production paths.

use crate::types::{Action, ModuleState, Policy, VaultLimit};
use alloy::primitives::{keccak256, Address, B256, U256};
use alloy::sol;
use alloy::sol_types::SolValue;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct SolVaultLimit {
        address vault;
        uint256 maxExposureRaw;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolPolicyTuple {
        bytes32 tag;
        uint8 permissionTier;
        uint256 operatingBufferRaw;
        uint256 microMaxRaw;
        uint256 delayedMaxRaw;
        uint256 hardMaxRaw;
        uint256 dailyLimitRaw;
        uint32 delaySeconds;
        uint16 maxShareSlippageBps;
        address guardian;
        bytes32 vaultLimitsHash;
        uint32 payeeDelaySeconds;
        uint32 payEarlyWindowSeconds;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolSessionTuple {
        bytes32 tag;
        uint256 chainId;
        address safe;
        address module;
        uint256 sessionStartEpoch;
        address sessionKey;
        uint64 sessionExpiresAt;
        uint256 sessionLimitRaw;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolMandateTuple {
        bytes32 tag;
        uint256 chainId;
        address safe;
        address module;
        uint256 policyEpoch;
        bytes32 policyHash;
        bytes32 obligationsRoot;
        uint256 reservedRaw;
        bytes32 sessionId;
        address sessionKey;
        uint64 sessionExpiresAt;
        uint256 sessionLimitRaw;
        bool paused;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolActionStruct {
        uint8 kind;
        address sourceVault;
        address destinationVault;
        uint256 assets;
        uint256 minSharesOut;
        uint256 maxSharesBurn;
        bytes32 obligationIdHash;
        bytes32 payeeIdHash;
        address recipient;
        uint64 dueAt;
        uint256 expectedNonce;
        uint256 policyEpoch;
        uint64 deadline;
        bytes32 decisionHash;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolActionDigestTuple {
        bytes32 tag;
        uint256 chainId;
        address safe;
        address module;
        bytes32 mandateHash;
        bytes32 sessionId;
        SolActionStruct action;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolCandidateIdTuple {
        bytes32 tag;
        bytes32 snapshotBlockHash;
        bytes32 actionDigestWithoutDecision;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SolObligationInnerTuple {
        bytes32 obligationIdHash;
        bytes32 payeeIdHash;
        uint256 amountRaw;
        uint64 dueAt;
    }
}

/// Computes TAG(name) = keccak256(UTF8(name)).
pub fn tag(name: &str) -> B256 {
    keccak256(name.as_bytes())
}

/// Domain separation tag for PolicyHash.
pub fn tag_policy() -> B256 {
    tag("KAVACHI_POLICY_V4_1")
}

/// Domain separation tag for SessionId.
pub fn tag_session() -> B256 {
    tag("KAVACHI_SESSION_V4_1")
}

/// Domain separation tag for MandateHash.
pub fn tag_mandate() -> B256 {
    tag("KAVACHI_MANDATE_V4_1")
}

/// Domain separation tag for ActionDigest.
pub fn tag_action() -> B256 {
    tag("KAVACHI_ACTION_V4_1")
}

/// Domain separation tag for CandidateId.
pub fn tag_candidate() -> B256 {
    tag("KAVACHI_CANDIDATE_V4_1")
}

/// Root hash of an empty obligations commitment (§8.2).
pub fn empty_obligations_root() -> B256 {
    tag("KAVACHI_EMPTY_OBLIGATIONS_V4")
}

/// Sorts vault limits ascending by vault address (uint160) per §7.4 & §9.
pub fn sort_vault_limits(mut vaults: Vec<VaultLimit>) -> Vec<VaultLimit> {
    vaults.sort_by_key(|v| v.vault);
    vaults
}

/// Computes vaultLimitsHash = keccak256(abi.encode((address,uint256)[] sortedVaultLimits)).
pub fn compute_vault_limits_hash(vaults: &[VaultLimit]) -> B256 {
    let sorted = sort_vault_limits(vaults.to_vec());
    let sol_vaults: Vec<SolVaultLimit> = sorted
        .into_iter()
        .map(|v| SolVaultLimit {
            vault: v.vault,
            maxExposureRaw: v.max_exposure_raw,
        })
        .collect();

    let encoded = sol_vaults.abi_encode();
    keccak256(encoded)
}

/// Computes PolicyHash (§9).
pub fn compute_policy_hash(policy: &Policy) -> B256 {
    let vault_limits_hash = compute_vault_limits_hash(&policy.vaults);
    compute_policy_hash_with_vault_limits(policy, vault_limits_hash)
}

/// Computes PolicyHash with precomputed vaultLimitsHash (§9).
pub fn compute_policy_hash_with_vault_limits(policy: &Policy, vault_limits_hash: B256) -> B256 {
    let sol_policy = SolPolicyTuple {
        tag: tag_policy(),
        permissionTier: policy.permission_tier.as_u8(),
        operatingBufferRaw: policy.operating_buffer_raw,
        microMaxRaw: policy.micro_max_raw,
        delayedMaxRaw: policy.delayed_max_raw,
        hardMaxRaw: policy.hard_max_raw,
        dailyLimitRaw: policy.daily_limit_raw,
        delaySeconds: policy.delay_seconds,
        maxShareSlippageBps: policy.max_share_slippage_bps,
        guardian: policy.guardian,
        vaultLimitsHash: vault_limits_hash,
        payeeDelaySeconds: policy.payee_delay_seconds,
        payEarlyWindowSeconds: policy.pay_early_window_seconds,
    };
    keccak256(sol_policy.abi_encode())
}

/// Computes SessionId (§9).
/// Returns B256::ZERO when sessionKey == Address::ZERO && sessionExpiresAt == 0 && sessionLimitRaw == 0.
pub fn compute_session_id(
    chain_id: U256,
    safe: Address,
    module: Address,
    session_start_epoch: U256,
    session_key: Address,
    session_expires_at: u64,
    session_limit_raw: U256,
) -> B256 {
    if session_key == Address::ZERO && session_expires_at == 0 && session_limit_raw == U256::ZERO {
        return B256::ZERO;
    }

    compute_session_id_raw(
        chain_id,
        safe,
        module,
        session_start_epoch,
        session_key,
        session_expires_at,
        session_limit_raw,
    )
}

/// Computes raw SessionId using the TAG_SESSION formula unconditionally.
pub fn compute_session_id_raw(
    chain_id: U256,
    safe: Address,
    module: Address,
    session_start_epoch: U256,
    session_key: Address,
    session_expires_at: u64,
    session_limit_raw: U256,
) -> B256 {
    let sol_session = SolSessionTuple {
        tag: tag_session(),
        chainId: chain_id,
        safe,
        module,
        sessionStartEpoch: session_start_epoch,
        sessionKey: session_key,
        sessionExpiresAt: session_expires_at,
        sessionLimitRaw: session_limit_raw,
    };
    keccak256(sol_session.abi_encode())
}

/// Computes MandateHash (§9).
#[allow(clippy::too_many_arguments)]
pub fn compute_mandate_hash(
    chain_id: U256,
    safe: Address,
    module: Address,
    policy_epoch: U256,
    policy_hash: B256,
    obligations_root: B256,
    reserved_raw: U256,
    session_id: B256,
    session_key: Address,
    session_expires_at: u64,
    session_limit_raw: U256,
    paused: bool,
) -> B256 {
    let sol_mandate = SolMandateTuple {
        tag: tag_mandate(),
        chainId: chain_id,
        safe,
        module,
        policyEpoch: policy_epoch,
        policyHash: policy_hash,
        obligationsRoot: obligations_root,
        reservedRaw: reserved_raw,
        sessionId: session_id,
        sessionKey: session_key,
        sessionExpiresAt: session_expires_at,
        sessionLimitRaw: session_limit_raw,
        paused,
    };
    keccak256(sol_mandate.abi_encode())
}

/// Computes MandateHash from ModuleState (§9).
pub fn compute_mandate_hash_from_state(
    chain_id: U256,
    safe: Address,
    module: Address,
    state: &ModuleState,
) -> B256 {
    compute_mandate_hash(
        chain_id,
        safe,
        module,
        state.policy_epoch,
        state.policy_hash,
        state.obligations_root,
        state.reserved_raw,
        state.session_id,
        state.session_key,
        state.session_expires_at,
        state.session_limit_raw,
        state.paused,
    )
}

fn to_sol_action(action: &Action) -> SolActionStruct {
    SolActionStruct {
        kind: action.kind.as_u8(),
        sourceVault: action.source_vault,
        destinationVault: action.destination_vault,
        assets: action.assets,
        minSharesOut: action.min_shares_out,
        maxSharesBurn: action.max_shares_burn,
        obligationIdHash: action.obligation_id_hash,
        payeeIdHash: action.payee_id_hash,
        recipient: action.recipient,
        dueAt: action.due_at,
        expectedNonce: action.expected_nonce,
        policyEpoch: action.policy_epoch,
        deadline: action.deadline,
        decisionHash: action.decision_hash,
    }
}

/// Computes ActionDigest (§9).
pub fn compute_action_digest(
    chain_id: U256,
    safe: Address,
    module: Address,
    mandate_hash: B256,
    session_id: B256,
    action: &Action,
) -> B256 {
    let sol_action = to_sol_action(action);
    let sol_digest = SolActionDigestTuple {
        tag: tag_action(),
        chainId: chain_id,
        safe,
        module,
        mandateHash: mandate_hash,
        sessionId: session_id,
        action: sol_action,
    };
    keccak256(sol_digest.abi_encode())
}

/// Computes CandidateId (§9).
pub fn compute_candidate_id(
    snapshot_block_hash: B256,
    action_digest_without_decision: B256,
) -> B256 {
    let sol_candidate = SolCandidateIdTuple {
        tag: tag_candidate(),
        snapshotBlockHash: snapshot_block_hash,
        actionDigestWithoutDecision: action_digest_without_decision,
    };
    keccak256(sol_candidate.abi_encode())
}

/// Computes CandidateId directly from action by zeroing decisionHash (§7.11 & §9).
#[allow(clippy::too_many_arguments)]
pub fn compute_candidate_id_from_action(
    chain_id: U256,
    safe: Address,
    module: Address,
    mandate_hash: B256,
    session_id: B256,
    snapshot_block_hash: B256,
    action: &Action,
) -> B256 {
    let mut action_without_decision = action.clone();
    action_without_decision.decision_hash = B256::ZERO;

    let digest_without_decision = compute_action_digest(
        chain_id,
        safe,
        module,
        mandate_hash,
        session_id,
        &action_without_decision,
    );
    compute_candidate_id(snapshot_block_hash, digest_without_decision)
}

/// Computes ObligationLeaf (§8.2 & §9): double hash via inner abi.encode.
pub fn compute_obligation_leaf(
    obligation_id_hash: B256,
    payee_id_hash: B256,
    amount_raw: U256,
    due_at: u64,
) -> B256 {
    let inner = SolObligationInnerTuple {
        obligationIdHash: obligation_id_hash,
        payeeIdHash: payee_id_hash,
        amountRaw: amount_raw,
        dueAt: due_at,
    };
    let inner_hash = keccak256(inner.abi_encode());
    keccak256(inner_hash.as_slice())
}

/// Computes DecisionHash (§9): keccak256 of UTF-8 RFC 8785 JCS canonical JSON.
pub fn compute_decision_hash(canonical_passport_json: &str) -> B256 {
    keccak256(canonical_passport_json.as_bytes())
}
