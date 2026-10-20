//! `fusion-core` v4 — Deterministic, memory-safe EVM execution engine with economic
//! guardrails for Kavachi V4.1.
//!
//! Exposes the frozen UniFFI surface specified in Appendix B (§25).
//! All exports enforce panic guarding: any panic across the FFI boundary returns
//! `{"ok":false,"error":{"code":"PANIC"}}`.

pub mod amount;
pub mod buckets;
pub mod candidates;
pub mod config;
pub mod encode;
pub mod executor;
pub mod guardrails;
pub mod hashing;
pub mod journal;
pub mod manifest;
pub mod merkle;
pub mod models;
pub mod passport;
pub mod planner_io;
pub mod preview;
pub mod router;
pub mod sign;
pub mod snapshot;
pub mod types;

use alloy::primitives::{Address, B256};

// Emits UniFFI scaffolding
uniffi::setup_scaffolding!("fusion_core");

/// Catches panics across the FFI boundary and serializes to a clean failure JSON (§25).
pub fn panic_guard<F>(f: F) -> String
where
    F: FnOnce() -> String + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => result,
        Err(_) => serde_json::json!({
            "ok": false,
            "error": { "code": "PANIC" }
        })
        .to_string(),
    }
}

/// Returns the engine API and hashing version (§25).
#[uniffi::export]
pub fn core_version() -> String {
    panic_guard(|| {
        serde_json::json!({
            "api": "4.0",
            "hashing": "V4_1",
            "gitRev": "card/K3"
        })
        .to_string()
    })
}

/// Parses and validates a deployment manifest JSON (§10.3, Appendix B).
#[uniffi::export]
pub fn load_manifest(json_str: String) -> String {
    panic_guard(
        || match manifest::DeploymentManifest::parse_and_validate(&json_str) {
            Ok(m) => serde_json::json!({
                "ok": true,
                "manifest": m
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "ok": false,
                "error": { "code": "INVALID_MANIFEST", "message": e.to_string() }
            })
            .to_string(),
        },
    )
}

/// Computes treasury buckets from a snapshot (§7.9, Appendix B).
#[uniffi::export]
pub fn compute_buckets(snapshot_json: String) -> String {
    panic_guard(|| {
        let snapshot: types::Snapshot = match serde_json::from_str(&snapshot_json) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::json!({
                    "ok": false,
                    "error": { "code": "INVALID_SNAPSHOT", "message": e.to_string() }
                })
                .to_string()
            }
        };
        let b = buckets::Buckets::from_snapshot(&snapshot);
        serde_json::json!({
            "ok": true,
            "free": b.free.to_string(),
            "reserved": b.reserved.to_string(),
            "sweepable": b.sweepable.to_string(),
            "shortfall": b.shortfall.to_string(),
            "deployed": b.deployed.to_string(),
            "total": b.total.to_string()
        })
        .to_string()
    })
}

/// Builds an obligation commitment Merkle tree (§8.2, Appendix B).
#[uniffi::export]
pub fn build_commitment(obligations_json: String) -> String {
    panic_guard(|| {
        let obligations: Vec<types::Obligation> = match serde_json::from_str(&obligations_json) {
            Ok(o) => o,
            Err(e) => {
                return serde_json::json!({
                    "ok": false,
                    "error": { "code": "INVALID_OBLIGATIONS", "message": e.to_string() }
                })
                .to_string()
            }
        };

        let mut reserved_raw = alloy::primitives::U256::ZERO;
        let mut leaves = Vec::new();
        let mut id_to_leaf = std::collections::HashMap::new();

        for ob in &obligations {
            reserved_raw = reserved_raw.saturating_add(ob.amount_raw);
            let leaf = hashing::compute_obligation_leaf(
                ob.obligation_id_hash,
                ob.payee_id_hash,
                ob.amount_raw,
                ob.due_at,
            );
            leaves.push(leaf);
            id_to_leaf.insert(ob.obligation_id_hash, leaf);
        }

        let tree = merkle::MerkleTree::from_leaves(leaves);
        let root = tree.root();
        let count = obligations.len();

        let mut proofs = std::collections::BTreeMap::new();
        for ob in &obligations {
            if let Some(leaf) = id_to_leaf.get(&ob.obligation_id_hash) {
                if let Ok(proof) = tree.proof_for_leaf(*leaf) {
                    proofs.insert(ob.obligation_id_hash.to_string(), proof);
                }
            }
        }

        serde_json::json!({
            "ok": true,
            "root": root,
            "reservedRaw": reserved_raw.to_string(),
            "count": count,
            "leaves": tree.sorted_leaves(),
            "proofs": proofs,
            "changeList": []
        })
        .to_string()
    })
}

/// Enumerates eligible treasury candidates deterministically (§10.4, Appendix B).
#[uniffi::export]
pub fn enumerate_candidates(
    snapshot_json: String,
    policy_json: String,
    obligations_json: String,
    rates_json: String,
) -> String {
    panic_guard(|| {
        let mut snapshot: types::Snapshot = match serde_json::from_str(&snapshot_json) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        if let Ok(p) = serde_json::from_str(&policy_json) {
            snapshot.policy = p;
        }

        let bills: Vec<types::Obligation> =
            serde_json::from_str(&obligations_json).unwrap_or_default();
        let rates: Vec<candidates::VaultRateInfo> =
            serde_json::from_str(&rates_json).unwrap_or_default();

        let set = candidates::enumerate_candidates(
            &snapshot,
            &bills,
            &[],
            &std::collections::BTreeMap::new(),
            &rates,
            alloy::primitives::U256::from(84532),
            snapshot.module_state.session_key,
            Address::ZERO,
            snapshot.module_state.mandate_hash,
            snapshot.module_state.session_id,
            true,
        );

        serde_json::json!({
            "ok": true,
            "candidateSetDigest": set.candidate_set_digest,
            "candidates": set.candidates,
            "hold": set.hold
        })
        .to_string()
    })
}

/// Generates sanitized prompt facts for edge model inference (§10.6, Appendix B).
#[uniffi::export]
pub fn planner_prompt(candidate_set_json: String, user_goal: Option<String>) -> String {
    panic_guard(|| {
        let set: candidates::CandidateSet = match serde_json::from_str(&candidate_set_json) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        let res = planner_io::build_planner_prompt(&set, user_goal.as_deref());
        serde_json::json!({
            "ok": true,
            "prompt": res.prompt,
            "labels": res.labels,
            "candidateSetDigest": res.candidate_set_digest
        })
        .to_string()
    })
}

/// Validates raw planner model choice (§10.6, Appendix B).
#[uniffi::export]
pub fn validate_planner_output(raw_text: String, candidate_set_json: String) -> String {
    panic_guard(|| {
        let set: candidates::CandidateSet = match serde_json::from_str(&candidate_set_json) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        let res = planner_io::validate_planner_output(&raw_text, &set);
        serde_json::json!({
            "ok": true,
            "choiceLabel": res.choice_label,
            "choiceId": res.choice_id,
            "reasonCode": res.reason_code,
            "retryable": res.retryable
        })
        .to_string()
    })
}

/// Builds canonical RFC 8785 passport JSON and DecisionHash (§10.8, Appendix B).
#[uniffi::export]
pub fn build_passport(decision_json: String) -> String {
    panic_guard(|| {
        let passport: passport::DecisionPassport = match serde_json::from_str(&decision_json) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        match passport.build_output() {
            Ok(out) => serde_json::json!({
                "ok": true,
                "passportJson": out.passport_json,
                "decisionHash": out.decision_hash
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                .to_string(),
        }
    })
}

/// Previews transaction execution against §7.7 normative rules (§10.7, Appendix B).
#[uniffi::export]
pub fn preview(candidate_json: String, snapshot_json: String, decision_hash: String) -> String {
    panic_guard(|| {
        let candidate: candidates::Candidate = match serde_json::from_str(&candidate_json) {
            Ok(c) => c,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        let snapshot: types::Snapshot = match serde_json::from_str(&snapshot_json) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };
        let hash: B256 = decision_hash.parse().unwrap_or(B256::ZERO);
        let res = preview::preview_candidate(
            &candidate,
            &snapshot,
            hash,
            alloy::primitives::U256::from(84532),
            Address::ZERO,
            Address::ZERO,
        );
        serde_json::json!({
            "ok": true,
            "result": res
        })
        .to_string()
    })
}

/// Runs self-test against embedded test vectors (Appendix B).
#[uniffi::export]
pub fn hash_vectors_selftest() -> String {
    panic_guard(|| {
        let v: serde_json::Value = match serde_json::from_str(include_str!(
            "../../../spec/vectors/v4_1.json"
        )) {
            Ok(val) => val,
            Err(e) => {
                return serde_json::json!({ "ok": false, "passed": false, "error": e.to_string() })
                    .to_string()
            }
        };

        let c = &v["constants"];
        let c1 = hashing::tag_policy().to_string().to_lowercase()
            == c["TAG_POLICY"].as_str().unwrap().to_lowercase();
        let c2 = hashing::tag_session().to_string().to_lowercase()
            == c["TAG_SESSION"].as_str().unwrap().to_lowercase();
        let c3 = hashing::tag_mandate().to_string().to_lowercase()
            == c["TAG_MANDATE"].as_str().unwrap().to_lowercase();
        let c4 = hashing::tag_action().to_string().to_lowercase()
            == c["TAG_ACTION"].as_str().unwrap().to_lowercase();
        let c5 = hashing::tag_candidate().to_string().to_lowercase()
            == c["TAG_CANDIDATE"].as_str().unwrap().to_lowercase();
        let c6 = hashing::empty_obligations_root().to_string().to_lowercase()
            == c["EMPTY_OBLIGATIONS_ROOT"].as_str().unwrap().to_lowercase();

        let passed = c1 && c2 && c3 && c4 && c5 && c6;
        serde_json::json!({
            "ok": true,
            "passed": passed
        })
        .to_string()
    })
}

/// Encodes transaction calldata for module and Safe operations (§10.2, Appendix B).
#[uniffi::export]
pub fn encode_call(kind: String, args_json: String) -> String {
    panic_guard(|| match encode::encode_call(&kind, &args_json) {
        Ok(call) => serde_json::json!({
            "ok": true,
            "to": call.to,
            "data": call.data
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "error": { "message": e }
        })
        .to_string(),
    })
}

/// Reads snapshot at single block (§10.5, Appendix B).
#[uniffi::export]
pub fn read_snapshot(manifest_json: String) -> String {
    panic_guard(|| {
        let manifest = match manifest::DeploymentManifest::parse_and_validate(&manifest_json) {
            Ok(m) => m,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };

        let res = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt.block_on(snapshot::read_snapshot(&manifest)),
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                    .to_string()
            }
        };

        match res {
            Ok(snap) => serde_json::json!({
                "ok": true,
                "snapshot": snap
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": { "message": e.to_string() } })
                .to_string(),
        }
    })
}

/// Signs prepared EIP-1559 transaction in :signer: process (§10.2, Appendix B).
#[uniffi::export]
pub fn sign_prepared(tx_json: String, key_hex: String, policy_json: String) -> String {
    panic_guard(
        || match sign::sign_prepared_json(&tx_json, &key_hex, &policy_json) {
            Ok(out) => serde_json::json!({
                "ok": true,
                "rawTx": out.raw_tx,
                "txHash": out.tx_hash
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "ok": false,
                "error": { "message": e.to_string() }
            })
            .to_string(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_reports_expected_contract() {
        let raw = core_version();
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        assert_eq!(parsed["api"], "4.0");
        assert_eq!(parsed["hashing"], "V4_1");
    }

    #[test]
    fn hash_vectors_selftest_passes() {
        let raw = hash_vectors_selftest();
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["passed"], true);
    }

    #[test]
    fn panic_guard_catches_unwind_and_reports_clean_json() {
        let raw = panic_guard(|| {
            // Assert panic converts cleanly
            panic_helper();
            String::new()
        });
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "PANIC");
    }

    fn panic_helper() {
        std::panic::panic_any("simulated failure");
    }
}
