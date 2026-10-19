//! Decision passport construction and canonical JSON (JCS, RFC 8785) formatting (§10.8).
//!
//! Produces the canonical passport JSON that generates the on-chain DecisionHash.
//! Reproduces the reference passport vector byte-for-byte.

use crate::hashing::compute_decision_hash;
use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// AHEM-G and AHEM-L report inside planner metadata (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AhemGReport {
    pub forced_tokens: u64,
    pub mode: String,
    pub retries: u64,
    pub vetoes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AhemLReport {
    pub mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AhemMetadata {
    pub g: AhemGReport,
    pub l: AhemLReport,
}

/// Metadata describing the planner execution environment (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerMetadata {
    pub ahem: AhemMetadata,
    pub host: String,
    pub kind: String,
    pub lib_mnn_sha256: String,
    pub libllm_sha256: String,
    pub model_id: String,
    pub model_sha256: String,
}

/// Snapshot reference in the decision passport (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportSnapshot {
    pub block: u64,
    pub hash: String,
}

/// Candidate summary for passport recording (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportCandidate {
    pub assets: String,
    pub id: String,
    pub kind: String,
    pub label: String,
    pub reason: String,
}

/// Decision passport DTO (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionPassport {
    pub candidate_set: Vec<PassportCandidate>,
    pub candidate_set_digest: String,
    pub chain_id: u64,
    pub choice_id: String,
    pub choice_label: String,
    pub created_at_chain: u64,
    pub mandate_hash: String,
    pub module: String,
    pub planner: PlannerMetadata,
    pub reason_code: String,
    pub route: String,
    pub safe: String,
    pub snapshot: PassportSnapshot,
    pub user_goal_sha256: String,
    pub version: String,
}

/// Canonicalizes any JSON value according to RFC 8785 (JSON Canonicalization Scheme, JCS).
///
/// Features:
/// - UTF-8 encoding.
/// - Deterministically sorted object keys by UTF-16 code units.
/// - Minimal spacing (no whitespace after ':' or ',').
pub fn to_canonical_json_value(val: &Value) -> String {
    match val {
        Value::Null => "null".to_string(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| format!("\"{s}\"")),
        Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(to_canonical_json_value).collect();
            format!("[{}]", inner.join(","))
        }
        Value::Object(map) => {
            // Sort keys lexicographically by UTF-16 code units (matches UTF-8 byte comparison for ASCII)
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|(k1, _), (k2, _)| {
                let u1: Vec<u16> = k1.encode_utf16().collect();
                let u2: Vec<u16> = k2.encode_utf16().collect();
                u1.cmp(&u2)
            });
            let inner: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let k_str = serde_json::to_string(k).unwrap_or_else(|_| format!("\"{k}\""));
                    let v_str = to_canonical_json_value(v);
                    format!("{k_str}:{v_str}")
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

/// Serializes a struct to canonical RFC 8785 JCS JSON string.
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json_val = serde_json::to_value(value)?;
    Ok(to_canonical_json_value(&json_val))
}

/// Canonicalizes an existing JSON string into JCS format.
pub fn canonicalize_json_str(raw_json: &str) -> Result<String, serde_json::Error> {
    let json_val: Value = serde_json::from_str(raw_json)?;
    Ok(to_canonical_json_value(&json_val))
}

/// Result of passport construction: canonical passport JSON and on-chain DecisionHash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportOutput {
    pub passport_json: String,
    pub decision_hash: B256,
}

impl DecisionPassport {
    /// Formats the passport into canonical JCS JSON and computes its DecisionHash.
    pub fn build_output(&self) -> Result<PassportOutput, serde_json::Error> {
        let passport_json = to_canonical_json(self)?;
        let decision_hash = compute_decision_hash(&passport_json);
        Ok(PassportOutput {
            passport_json,
            decision_hash,
        })
    }
}

/// Helper to build a deterministic passport with default host planner settings (§10.8).
#[allow(clippy::too_many_arguments)]
pub fn build_deterministic_passport(
    candidates: &[PassportCandidate],
    candidate_set_digest: B256,
    chain_id: u64,
    choice_id: B256,
    choice_label: &str,
    created_at_chain: u64,
    mandate_hash: B256,
    module: Address,
    reason_code: &str,
    route: &str,
    safe: Address,
    snapshot_block: u64,
    snapshot_hash: B256,
    user_goal_hash: B256,
) -> Result<PassportOutput, serde_json::Error> {
    let passport = DecisionPassport {
        candidate_set: candidates.to_vec(),
        candidate_set_digest: format!("{candidate_set_digest:#x}"),
        chain_id,
        choice_id: format!("{choice_id:#x}"),
        choice_label: choice_label.to_string(),
        created_at_chain,
        mandate_hash: format!("{mandate_hash:#x}"),
        module: module.to_checksum(None),
        planner: PlannerMetadata {
            ahem: AhemMetadata {
                g: AhemGReport {
                    forced_tokens: 0,
                    mode: "OFF".to_string(),
                    retries: 0,
                    vetoes: 0,
                },
                l: AhemLReport {
                    mode: "OFF".to_string(),
                },
            },
            host: "linux-host".to_string(),
            kind: "deterministic".to_string(),
            lib_mnn_sha256: format!("{:#x}", B256::ZERO),
            libllm_sha256: format!("{:#x}", B256::ZERO),
            model_id: "none".to_string(),
            model_sha256: format!("{:#x}", B256::ZERO),
        },
        reason_code: reason_code.to_string(),
        route: route.to_string(),
        safe: safe.to_checksum(None),
        snapshot: PassportSnapshot {
            block: snapshot_block,
            hash: format!("{snapshot_hash:#x}"),
        },
        user_goal_sha256: format!("{user_goal_hash:#x}"),
        version: "4.1".to_string(),
    };

    passport.build_output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_matching_reference_vector() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir).join("../../spec/vectors/v4_1.json");
        let content = std::fs::read_to_string(&path).expect("read vectors");
        let v: Value = serde_json::from_str(&content).expect("parse json");

        let expected_json = v["decision_hash"]["input"]["canonicalPassportJson"]
            .as_str()
            .expect("canonical json string");
        let expected_hash = v["decision_hash"]["expected"]["decisionHash"]
            .as_str()
            .expect("expected hash");

        let parsed: Value = serde_json::from_str(expected_json).expect("parse canonical json");
        let reconstructed = to_canonical_json_value(&parsed);

        assert_eq!(reconstructed, expected_json);

        let hash = compute_decision_hash(&reconstructed);
        assert_eq!(format!("{hash:#x}"), expected_hash);
    }
}
