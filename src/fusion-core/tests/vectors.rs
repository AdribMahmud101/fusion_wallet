//! Vector integration tests for Kavachi V4.1.
//!
//! Loads `spec/vectors/v4_1.json` at test time and verifies all reference vectors.
//! NEVER hardcodes expected values into Rust source.

use alloy::primitives::{Address, B256, U256};
use fusion_core::hashing::*;
use fusion_core::merkle::MerkleTree;
use fusion_core::types::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::str::FromStr;

fn load_vectors() -> Value {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let vector_path = Path::new(manifest_dir).join("../../spec/vectors/v4_1.json");
    let content = fs::read_to_string(&vector_path)
        .unwrap_or_else(|e| panic!("Failed to read vectors file at {vector_path:?}: {e}"));
    serde_json::from_str(&content).expect("Valid vectors JSON")
}

fn parse_u256(val: &Value) -> U256 {
    match val {
        Value::String(s) => {
            if let Some(stripped) = s.strip_prefix("0x") {
                U256::from_str_radix(stripped, 16).expect("Valid hex uint256")
            } else {
                U256::from_str_radix(s, 10).expect("Valid base-10 uint256")
            }
        }
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                U256::from(u)
            } else {
                panic!("Unsupported number format: {n}")
            }
        }
        _ => panic!("Expected string or number for U256: {val:?}"),
    }
}

fn parse_b256(val: &Value) -> B256 {
    let s = val.as_str().expect("B256 must be a string");
    B256::from_str(s).expect("Valid B256 hex string")
}

fn parse_address(val: &Value) -> Address {
    let s = val.as_str().expect("Address must be a string");
    Address::from_str(s).expect("Valid Address hex string")
}

fn parse_u64(val: &Value) -> u64 {
    match val {
        Value::String(s) => s.parse().expect("Valid u64 string"),
        Value::Number(n) => n.as_u64().expect("Valid u64 number"),
        _ => panic!("Expected u64: {val:?}"),
    }
}

fn parse_u32(val: &Value) -> u32 {
    parse_u64(val) as u32
}

fn parse_u16(val: &Value) -> u16 {
    parse_u64(val) as u16
}

fn parse_bool(val: &Value) -> bool {
    val.as_bool().expect("Valid boolean")
}

#[test]
fn test_vector_constants() {
    let v = load_vectors();
    let constants = &v["constants"];

    assert_eq!(tag_policy(), parse_b256(&constants["TAG_POLICY"]));
    assert_eq!(tag_session(), parse_b256(&constants["TAG_SESSION"]));
    assert_eq!(tag_mandate(), parse_b256(&constants["TAG_MANDATE"]));
    assert_eq!(tag_action(), parse_b256(&constants["TAG_ACTION"]));
    assert_eq!(tag_candidate(), parse_b256(&constants["TAG_CANDIDATE"]));
    assert_eq!(
        empty_obligations_root(),
        parse_b256(&constants["EMPTY_OBLIGATIONS_ROOT"])
    );
}

#[test]
fn test_vector_policy_hash() {
    let v = load_vectors();
    let sec = &v["policy_hash"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let vaults: Vec<VaultLimit> = input["vaults"]
        .as_array()
        .expect("vaults array")
        .iter()
        .map(|v| VaultLimit {
            vault: parse_address(&v["vault"]),
            max_exposure_raw: parse_u256(&v["maxExposureRaw"]),
        })
        .collect();

    let policy = Policy {
        permission_tier: PermissionTier::try_from(parse_u64(&input["permissionTier"]) as u8)
            .expect("Valid tier"),
        operating_buffer_raw: parse_u256(&input["operatingBufferRaw"]),
        micro_max_raw: parse_u256(&input["microMaxRaw"]),
        delayed_max_raw: parse_u256(&input["delayedMaxRaw"]),
        hard_max_raw: parse_u256(&input["hardMaxRaw"]),
        daily_limit_raw: parse_u256(&input["dailyLimitRaw"]),
        delay_seconds: parse_u32(&input["delaySeconds"]),
        max_share_slippage_bps: parse_u16(&input["maxShareSlippageBps"]),
        guardian: parse_address(&input["guardian"]),
        vaults,
        payee_delay_seconds: parse_u32(&input["payeeDelaySeconds"]),
        pay_early_window_seconds: parse_u32(&input["payEarlyWindowSeconds"]),
    };

    let computed_vault_limits_hash = compute_vault_limits_hash(&policy.vaults);
    let expected_vault_limits_hash = parse_b256(&expected["vaultLimitsHash"]);
    assert_eq!(computed_vault_limits_hash, expected_vault_limits_hash);

    let computed_policy_hash = compute_policy_hash(&policy);
    let expected_policy_hash = parse_b256(&expected["policyHash"]);
    assert_eq!(computed_policy_hash, expected_policy_hash);

    // Vault sorting invariance test (DECISIONS.md §3)
    let mut reversed_policy = policy.clone();
    reversed_policy.vaults.reverse();
    assert_eq!(
        compute_vault_limits_hash(&reversed_policy.vaults),
        expected_vault_limits_hash
    );
    assert_eq!(compute_policy_hash(&reversed_policy), expected_policy_hash);
}

#[test]
fn test_vector_session_id() {
    let v = load_vectors();
    let sec = &v["session_id"];

    // With session
    {
        let with_s = &sec["with_session"];
        let input = &with_s["input"];
        let expected = &with_s["expected"];

        let session_id = compute_session_id(
            parse_u256(&input["chainId"]),
            parse_address(&input["safe"]),
            parse_address(&input["module"]),
            parse_u256(&input["sessionStartEpoch"]),
            parse_address(&input["sessionKey"]),
            parse_u64(&input["sessionExpiresAt"]),
            parse_u256(&input["sessionLimitRaw"]),
        );
        assert_eq!(session_id, parse_b256(&expected["sessionId"]));
    }

    // Without session
    {
        let without_s = &sec["without_session"];
        let input = &without_s["input"];
        let expected = &without_s["expected"];

        let session_id = compute_session_id(
            parse_u256(&input["chainId"]),
            parse_address(&input["safe"]),
            parse_address(&input["module"]),
            parse_u256(&input["sessionStartEpoch"]),
            parse_address(&input["sessionKey"]),
            parse_u64(&input["sessionExpiresAt"]),
            parse_u256(&input["sessionLimitRaw"]),
        );
        assert_eq!(session_id, parse_b256(&expected["sessionId"]));
    }
}

#[test]
fn test_vector_mandate_hash() {
    let v = load_vectors();
    let sec = &v["mandate_hash"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let mandate_hash = compute_mandate_hash(
        parse_u256(&input["chainId"]),
        parse_address(&input["safe"]),
        parse_address(&input["module"]),
        parse_u256(&input["policyEpoch"]),
        parse_b256(&input["policyHash"]),
        parse_b256(&input["obligationsRoot"]),
        parse_u256(&input["reservedRaw"]),
        parse_b256(&input["sessionId"]),
        parse_address(&input["sessionKey"]),
        parse_u64(&input["sessionExpiresAt"]),
        parse_u256(&input["sessionLimitRaw"]),
        parse_bool(&input["paused"]),
    );
    assert_eq!(mandate_hash, parse_b256(&expected["mandateHash"]));
}

fn parse_action_helper(act: &Value) -> Action {
    Action {
        kind: ActionKind::try_from(parse_u64(&act["kind"]) as u8).expect("Valid action kind"),
        source_vault: parse_address(&act["sourceVault"]),
        destination_vault: parse_address(&act["destinationVault"]),
        assets: parse_u256(&act["assets"]),
        min_shares_out: parse_u256(&act["minSharesOut"]),
        max_shares_burn: parse_u256(&act["maxSharesBurn"]),
        obligation_id_hash: parse_b256(&act["obligationIdHash"]),
        payee_id_hash: parse_b256(&act["payeeIdHash"]),
        recipient: parse_address(&act["recipient"]),
        due_at: parse_u64(&act["dueAt"]),
        expected_nonce: parse_u256(&act["expectedNonce"]),
        policy_epoch: parse_u256(&act["policyEpoch"]),
        deadline: parse_u64(&act["deadline"]),
        decision_hash: parse_b256(&act["decisionHash"]),
    }
}

#[test]
fn test_vector_actions() {
    let v = load_vectors();
    let actions = &v["actions"];

    for key in ["deposit", "withdraw", "rebalance", "pay_obligation"] {
        let entry = &actions[key];
        let input = &entry["input"];
        let expected = &entry["expected"];

        let chain_id = parse_u256(&input["chainId"]);
        let safe = parse_address(&input["safe"]);
        let module = parse_address(&input["module"]);
        let mandate_hash = parse_b256(&input["mandateHash"]);
        let session_id = parse_b256(&input["sessionId"]);
        let action = parse_action_helper(&input["action"]);

        let digest =
            compute_action_digest(chain_id, safe, module, mandate_hash, session_id, &action);
        assert_eq!(
            digest,
            parse_b256(&expected["actionDigest"]),
            "Action digest mismatch for {key}"
        );
    }
}

#[test]
fn test_vector_candidate_id() {
    let v = load_vectors();
    let sec = &v["candidate_id"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let candidate_id = compute_candidate_id(
        parse_b256(&input["snapshotBlockHash"]),
        parse_b256(&input["actionDigestWithoutDecision"]),
    );
    assert_eq!(candidate_id, parse_b256(&expected["candidateId"]));
}

#[test]
fn test_vector_commitments_empty() {
    let v = load_vectors();
    let sec = &v["commitments"]["empty_commitment"];
    let expected = &sec["expected"];

    let tree = MerkleTree::from_leaves(vec![]);
    assert_eq!(tree.root(), parse_b256(&expected["root"]));
}

#[test]
fn test_vector_commitments_single_leaf() {
    let v = load_vectors();
    let sec = &v["commitments"]["single_leaf"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let leaf = compute_obligation_leaf(
        parse_b256(&input["obligationIdHash"]),
        parse_b256(&input["payeeIdHash"]),
        parse_u256(&input["amountRaw"]),
        parse_u64(&input["dueAt"]),
    );
    assert_eq!(leaf, parse_b256(&expected["leaf"]));

    let tree = MerkleTree::from_leaves(vec![leaf]);
    assert_eq!(tree.root(), parse_b256(&expected["root"]));

    let proof = tree.proof_for_leaf(leaf).expect("Proof for leaf");
    assert!(proof.is_empty());
    assert!(MerkleTree::verify(&proof, tree.root(), leaf));
}

#[test]
fn test_vector_commitments_three_leaves() {
    let v = load_vectors();
    let sec = &v["commitments"]["three_leaves"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let obligations = input["obligations"].as_array().expect("obligations array");
    let leaves: Vec<B256> = obligations
        .iter()
        .map(|ob| {
            compute_obligation_leaf(
                parse_b256(&ob["obligationIdHash"]),
                parse_b256(&ob["payeeIdHash"]),
                parse_u256(&ob["amountRaw"]),
                parse_u64(&ob["dueAt"]),
            )
        })
        .collect();

    let expected_sorted: Vec<B256> = expected["sortedLeaves"]
        .as_array()
        .expect("sortedLeaves array")
        .iter()
        .map(parse_b256)
        .collect();

    let tree = MerkleTree::from_leaves(leaves);
    assert_eq!(tree.sorted_leaves(), expected_sorted.as_slice());
    assert_eq!(tree.root(), parse_b256(&expected["root"]));

    let expected_proofs = expected["proofs"].as_array().expect("proofs array");
    for (i, leaf) in expected_sorted.iter().enumerate() {
        let proof = tree.proof_for_index(i).expect("Proof for index");
        let expected_proof: Vec<B256> = expected_proofs[i]
            .as_array()
            .expect("proof elements")
            .iter()
            .map(parse_b256)
            .collect();
        assert_eq!(proof, expected_proof, "Proof mismatch for leaf {i}");
        assert!(MerkleTree::verify(&proof, tree.root(), *leaf));
    }
}

#[test]
fn test_vector_decision_hash() {
    let v = load_vectors();
    let sec = &v["decision_hash"];
    let input = &sec["input"];
    let expected = &sec["expected"];

    let passport_json = input["canonicalPassportJson"]
        .as_str()
        .expect("canonicalPassportJson string");
    let decision_hash = compute_decision_hash(passport_json);
    assert_eq!(decision_hash, parse_b256(&expected["decisionHash"]));
}
