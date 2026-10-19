//! Android-only adapters around the public fusion-core APIs. Host logic is unchanged.
//! Context addresses are explicit: the legacy UniFFI enumeration/preview convenience
//! functions substitute zero addresses, which cannot bind a real treasury action.
use alloy::primitives::{Address, B256, U256};
use serde_json::json;
use std::str::FromStr;

uniffi::setup_scaffolding!("fusion_android_core");

fn guarded(f: impl FnOnce() -> Result<serde_json::Value, String> + std::panic::UnwindSafe) -> String {
    fusion_core::panic_guard(|| match f() {
        Ok(value) => value.to_string(),
        Err(error) => json!({"ok":false,"error":{"message":error}}).to_string(),
    })
}

#[uniffi::export]
pub fn candidates(snapshot_json: String, bills_json: String, payees_json: String, proofs_json: String,
                  rates_json: String, safe: String, module: String) -> String {
    guarded(|| {
        let snapshot: fusion_core::types::Snapshot = serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let bills: Vec<fusion_core::types::Obligation> = serde_json::from_str(&bills_json).map_err(|e| e.to_string())?;
        let payees: Vec<fusion_core::types::Payee> = serde_json::from_str(&payees_json).map_err(|e| e.to_string())?;
        let proofs = serde_json::from_str(&proofs_json).map_err(|e| e.to_string())?;
        let rates: Vec<fusion_core::candidates::VaultRateInfo> = serde_json::from_str(&rates_json).map_err(|e| e.to_string())?;
        let safe = Address::from_str(&safe).map_err(|e| e.to_string())?;
        let module = Address::from_str(&module).map_err(|e| e.to_string())?;
        let set = fusion_core::candidates::enumerate_candidates(&snapshot, &bills, &payees, &proofs, &rates,
            U256::from(84532), safe, module, snapshot.module_state.mandate_hash, snapshot.module_state.session_id, true);
        Ok(json!({"ok":true,"set":set}))
    })
}

#[uniffi::export]
pub fn preview(candidate_json: String, snapshot_json: String, decision_hash: String, safe: String, module: String) -> String {
    guarded(|| {
        let candidate = serde_json::from_str(&candidate_json).map_err(|e| e.to_string())?;
        let snapshot = serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let hash = B256::from_str(&decision_hash).map_err(|e| e.to_string())?;
        let safe = Address::from_str(&safe).map_err(|e| e.to_string())?;
        let module = Address::from_str(&module).map_err(|e| e.to_string())?;
        Ok(json!({"ok":true,"result":fusion_core::preview::preview_candidate(&candidate, &snapshot, hash, U256::from(84532), safe, module)}))
    })
}

#[uniffi::export]
pub fn planner_prompt(set: String, goal: Option<String>) -> String { fusion_core::planner_prompt(set, goal) }
#[uniffi::export]
pub fn validate_output(raw: String, set: String) -> String { fusion_core::validate_planner_output(raw, set) }
#[uniffi::export]
pub fn build_passport(decision: String) -> String { fusion_core::build_passport(decision) }
#[uniffi::export]
pub fn build_commitment(obligations: String) -> String { fusion_core::build_commitment(obligations) }
#[uniffi::export]
pub fn encode_call(kind: String, args: String) -> String { fusion_core::encode_call(kind, args) }
#[uniffi::export]
pub fn sign_prepared(tx: String, key: String) -> String { fusion_core::sign_prepared(tx, key) }
