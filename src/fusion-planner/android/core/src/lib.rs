//! Android-only adapters around the public fusion-core APIs. Host logic is unchanged.
//! Context addresses are explicit: the legacy UniFFI enumeration/preview convenience
//! functions substitute zero addresses, which cannot bind a real treasury action.
use alloy::primitives::{Address, B256, U256};
use serde_json::json;
use std::str::FromStr;

uniffi::setup_scaffolding!("fusion_android_core");

fn guarded(
    f: impl FnOnce() -> Result<serde_json::Value, String> + std::panic::UnwindSafe,
) -> String {
    fusion_core::panic_guard(|| match f() {
        Ok(value) => value.to_string(),
        Err(error) => json!({"ok":false,"error":{"message":error}}).to_string(),
    })
}

#[uniffi::export]
pub fn candidates(
    snapshot_json: String,
    bills_json: String,
    payees_json: String,
    proofs_json: String,
    rates_json: String,
    safe: String,
    module: String,
) -> String {
    guarded(|| {
        let snapshot: fusion_core::types::Snapshot =
            serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let bills: Vec<fusion_core::types::Obligation> =
            serde_json::from_str(&bills_json).map_err(|e| e.to_string())?;
        let payees: Vec<fusion_core::types::Payee> =
            serde_json::from_str(&payees_json).map_err(|e| e.to_string())?;
        let proofs = serde_json::from_str(&proofs_json).map_err(|e| e.to_string())?;
        let rates: Vec<fusion_core::candidates::VaultRateInfo> =
            serde_json::from_str(&rates_json).map_err(|e| e.to_string())?;
        let safe = Address::from_str(&safe).map_err(|e| e.to_string())?;
        let module = Address::from_str(&module).map_err(|e| e.to_string())?;
        let set = fusion_core::candidates::enumerate_candidates(
            &snapshot,
            &bills,
            &payees,
            &proofs,
            &rates,
            U256::from(84532),
            safe,
            module,
            snapshot.module_state.mandate_hash,
            snapshot.module_state.session_id,
            true,
        );
        Ok(json!({"ok":true,"set":set}))
    })
}

/// A separate owner-signed operational mandate, not a yield recommendation.
/// Kotlin in BOTH runtime and signer independently verifies the successful 2/2
/// Safe authorization receipt. The ordinary economics strategy is unchanged.
#[uniffi::export]
pub fn allocation_candidates(
    snapshot_json: String,
    mandate_json: String,
    safe: String,
    module: String,
) -> String {
    guarded(|| {
        use fusion_core::buckets::Buckets;
        use fusion_core::candidates::{Candidate, CandidateSummary, FeeAssumptions, Route, Tier};
        use fusion_core::types::{ActionKind, Snapshot};
        let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let mandate: serde_json::Value =
            serde_json::from_str(&mandate_json).map_err(|e| e.to_string())?;
        let amount: U256 = mandate["assetsRaw"]
            .as_str()
            .ok_or("INVALID_OWNER_ALLOCATION")?
            .parse()
            .map_err(|_| "INVALID_OWNER_ALLOCATION")?;
        let nonce: U256 = mandate["depositNonce"]
            .as_str()
            .ok_or("INVALID_OWNER_ALLOCATION")?
            .parse()
            .map_err(|_| "INVALID_OWNER_ALLOCATION")?;
        let vault: Address = mandate["vault"]
            .as_str()
            .ok_or("INVALID_OWNER_ALLOCATION")?
            .parse()
            .map_err(|_| "INVALID_OWNER_ALLOCATION")?;
        let safe: Address = safe.parse().map_err(|_| "INVALID_CONTEXT")?;
        let module: Address = module.parse().map_err(|_| "INVALID_CONTEXT")?;
        if !matches!(mandate["phase"].as_str(), Some("DEPOSIT" | "WITHDRAW")) {
            return Err("INVALID_OWNER_ALLOCATION".into());
        }
        let deposit = mandate["phase"] == "DEPOSIT";
        let st = &snapshot.module_state;
        if snapshot.policy.permission_tier != fusion_core::types::PermissionTier::Autonomous
            || st.reserved_raw != U256::ZERO
            || amount == U256::ZERO
            || amount > U256::from(10_000_000)
            || amount > snapshot.policy.micro_max_raw
            || mandate["expiresAt"].as_u64().unwrap_or(u64::MAX) > st.session_expires_at
            || snapshot.block_timestamp
                >= mandate["expiresAt"]
                    .as_u64()
                    .ok_or("INVALID_OWNER_ALLOCATION")?
            || mandate["sessionId"] != st.session_id.to_string()
            || mandate["policyHash"] != st.policy_hash.to_string()
            || st.action_nonce != nonce + if deposit { U256::ZERO } else { U256::from(1) }
            || snapshot.policy.vaults.first().map(|v| v.vault) != Some(vault)
        {
            return Err("INVALID_OWNER_ALLOCATION".into());
        }
        let mut set = fusion_core::candidates::enumerate_candidates(
            &snapshot,
            &[],
            &[],
            &Default::default(),
            &[],
            U256::from(84532),
            safe,
            module,
            st.mandate_hash,
            st.session_id,
            true,
        );
        let before = Buckets::from_snapshot(&snapshot);
        let assets = if deposit {
            amount
        } else {
            amount.min(snapshot.vault_a_assets)
        };
        let cap = snapshot
            .policy
            .vaults
            .first()
            .ok_or("INVALID_OWNER_ALLOCATION")?
            .max_exposure_raw;
        if assets == U256::ZERO
            || (deposit
                && (assets > before.sweepable
                    || assets > cap.saturating_sub(snapshot.vault_a_assets)))
        {
            return Err("INVALID_OWNER_ALLOCATION".into());
        }
        let mut action = set.hold.action.clone();
        action.kind = if deposit {
            ActionKind::Deposit
        } else {
            ActionKind::Withdraw
        };
        action.assets = assets;
        action.deadline = action.deadline.min(
            mandate["expiresAt"]
                .as_u64()
                .ok_or("INVALID_OWNER_ALLOCATION")?,
        );
        if deposit {
            action.destination_vault = vault;
        } else {
            action.source_vault = vault;
        }
        let id = fusion_core::hashing::compute_candidate_id_from_action(
            U256::from(84532),
            safe,
            module,
            st.mandate_hash,
            st.session_id,
            snapshot.block_hash,
            &action,
        );
        let after = Buckets::compute(
            if deposit {
                snapshot.safe_cash - assets
            } else {
                snapshot.safe_cash + assets
            },
            st.reserved_raw,
            snapshot.policy.operating_buffer_raw,
            if deposit {
                before.deployed + assets
            } else {
                before.deployed.saturating_sub(assets)
            },
        );
        let owner_hash = mandate["ownerTxHash"]
            .as_str()
            .ok_or("INVALID_OWNER_ALLOCATION")?;
        let _: B256 = owner_hash.parse().map_err(|_| "INVALID_OWNER_ALLOCATION")?;
        set.candidates = vec![Candidate {
            label: "A".into(),
            id,
            kind: action.kind,
            action,
            proof: vec![],
            tier: Tier::Instant,
            route: Route::Instant,
            reason: format!("OWNER_ALLOCATION_REHEARSAL:{owner_hash}"),
            before_buckets: before,
            after_buckets: after,
            fee_assumptions: FeeAssumptions {
                fee_raw: U256::from(50_000),
            },
        }];
        let summaries: Vec<CandidateSummary> = set
            .candidates
            .iter()
            .chain(std::iter::once(&set.hold))
            .map(|c| CandidateSummary {
                assets: c.action.assets,
                id: c.id,
                kind: c.kind.name().into(),
                label: c.label.clone(),
                reason: c.reason.clone(),
            })
            .collect();
        set.candidate_set_digest =
            fusion_core::candidates::compute_candidate_set_digest(&summaries);
        Ok(json!({"ok":true,"set":set}))
    })
}

/// Bind policy share limits to real ERC-4626 conversions before presenting
/// candidates to the planner. Every quote must match the snapshot and exact
/// asset amount. This changes IDs/digest before inference, never after choice.
#[uniffi::export]
pub fn quote_candidates(
    set_json: String,
    snapshot_json: String,
    quotes_json: String,
    safe: String,
    module: String,
) -> String {
    guarded(|| {
        use fusion_core::candidates::{CandidateSet, CandidateSummary};
        use fusion_core::types::{ActionKind, Snapshot};
        let mut set: CandidateSet = serde_json::from_str(&set_json).map_err(|e| e.to_string())?;
        let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let quotes: Vec<serde_json::Value> =
            serde_json::from_str(&quotes_json).map_err(|e| e.to_string())?;
        let safe: Address = safe.parse().map_err(|_| "INVALID_CONTEXT")?;
        let module: Address = module.parse().map_err(|_| "INVALID_CONTEXT")?;
        for candidate in &mut set.candidates {
            let action = &mut candidate.action;
            let vault = match action.kind {
                ActionKind::Deposit => action.destination_vault,
                ActionKind::Withdraw => action.source_vault,
                _ => continue,
            };
            let quote = quotes
                .iter()
                .find(|q| {
                    q["vault"].as_str().and_then(|v| v.parse::<Address>().ok()) == Some(vault)
                        && q["assets"].as_str().and_then(|v| v.parse::<U256>().ok())
                            == Some(action.assets)
                        && q["kind"].as_u64() == Some(action.kind as u64)
                        && q["blockHash"].as_str() == Some(snapshot.block_hash.to_string().as_str())
                })
                .ok_or("MISSING_PINNED_VAULT_QUOTE")?;
            let shares: U256 = quote["convertToShares"]
                .as_str()
                .ok_or("INVALID_QUOTE")?
                .parse()
                .map_err(|_| "INVALID_QUOTE")?;
            let preview: U256 = quote["preview"]
                .as_str()
                .ok_or("INVALID_QUOTE")?
                .parse()
                .map_err(|_| "INVALID_QUOTE")?;
            let bps = U256::from(snapshot.policy.max_share_slippage_bps);
            let denominator = U256::from(10_000);
            if action.kind == ActionKind::Deposit {
                let floor = shares
                    .checked_mul(denominator - bps)
                    .ok_or("QUOTE_OVERFLOW")?
                    / denominator;
                if preview == U256::ZERO || preview < floor {
                    return Err("SHARE_BOUND_VIOLATION".into());
                }
                action.min_shares_out = floor;
            } else {
                let ceiling = shares
                    .checked_mul(denominator + bps)
                    .ok_or("QUOTE_OVERFLOW")?
                    .checked_add(denominator - U256::from(1))
                    .ok_or("QUOTE_OVERFLOW")?
                    / denominator;
                if preview == U256::ZERO || preview > ceiling {
                    return Err("SHARE_BOUND_VIOLATION".into());
                }
                // Use the exact observed preview, inside the policy ceiling.
                // A maximum equal to the ceiling can exceed a later ceiling
                // by one share unit as waUSDC's exchange rate accrues upward.
                action.max_shares_burn = preview;
            }
            candidate.id = fusion_core::hashing::compute_candidate_id_from_action(
                U256::from(84532),
                safe,
                module,
                snapshot.module_state.mandate_hash,
                snapshot.module_state.session_id,
                snapshot.block_hash,
                action,
            );
        }
        let summaries: Vec<CandidateSummary> = set
            .candidates
            .iter()
            .chain(std::iter::once(&set.hold))
            .map(|c| CandidateSummary {
                assets: c.action.assets,
                id: c.id,
                kind: c.kind.name().into(),
                label: c.label.clone(),
                reason: c.reason.clone(),
            })
            .collect();
        set.candidate_set_digest =
            fusion_core::candidates::compute_candidate_set_digest(&summaries);
        Ok(json!({"ok":true,"set":set}))
    })
}

#[uniffi::export]
pub fn preview(
    candidate_json: String,
    snapshot_json: String,
    decision_hash: String,
    safe: String,
    module: String,
) -> String {
    guarded(|| {
        let candidate = serde_json::from_str(&candidate_json).map_err(|e| e.to_string())?;
        let snapshot = serde_json::from_str(&snapshot_json).map_err(|e| e.to_string())?;
        let hash = B256::from_str(&decision_hash).map_err(|e| e.to_string())?;
        let safe = Address::from_str(&safe).map_err(|e| e.to_string())?;
        let module = Address::from_str(&module).map_err(|e| e.to_string())?;
        Ok(
            json!({"ok":true,"result":fusion_core::preview::preview_candidate(&candidate, &snapshot, hash, U256::from(84532), safe, module)}),
        )
    })
}

#[uniffi::export]
pub fn planner_prompt(set: String, goal: Option<String>) -> String {
    fusion_core::planner_prompt(set, goal)
}
#[uniffi::export]
pub fn validate_output(raw: String, set: String) -> String {
    fusion_core::validate_planner_output(raw, set)
}
#[uniffi::export]
pub fn build_passport(decision: String) -> String {
    fusion_core::build_passport(decision)
}
#[uniffi::export]
pub fn build_commitment(obligations: String) -> String {
    fusion_core::build_commitment(obligations)
}
#[uniffi::export]
pub fn encode_call(kind: String, args: String) -> String {
    fusion_core::encode_call(kind, args)
}
#[uniffi::export]
pub fn sign_prepared(tx: String, key: String, policy: String) -> String {
    let key = zeroize::Zeroizing::new(key);
    guarded(
        || match fusion_core::sign::sign_prepared_json(&tx, key.as_str(), &policy) {
            Ok(out) => Ok(json!({"ok":true,"rawTx":out.raw_tx,"txHash":out.tx_hash})),
            Err(fusion_core::sign::SignError::Refused(reason)) => Err(reason.to_string()),
            Err(_) => Err("SIGNING_FAILED".into()),
        },
    )
}

/// A constrained signer input: bind immutable app report to its original set and
/// passport, then re-preview the exact action against an independently fresh read.
/// Returns only an instantaneous Kavachi call; no generic transaction endpoint.
#[uniffi::export]
pub fn verify_for_signing(
    report_json: String,
    fresh_snapshot_json: String,
    safe: String,
    module: String,
) -> String {
    guarded(|| verify_report(&report_json, &fresh_snapshot_json, &safe, &module))
}

fn verify_report(
    report_json: &str,
    fresh_json: &str,
    safe: &str,
    module: &str,
) -> Result<serde_json::Value, String> {
    use fusion_core::hashing::compute_candidate_id_from_action;
    use fusion_core::{
        candidates::{Candidate, CandidateSet, CandidateSummary, Route},
        passport::{DecisionPassport, PassportCandidate},
        types::Snapshot,
    };
    let report: serde_json::Value =
        serde_json::from_str(report_json).map_err(|_| "INVALID_REPORT")?;
    let passport_json = report["passportJson"].as_str().ok_or("INVALID_PASSPORT")?;
    let passport: DecisionPassport =
        serde_json::from_str(passport_json).map_err(|_| "INVALID_PASSPORT")?;
    let built = passport.build_output().map_err(|_| "INVALID_PASSPORT")?;
    if built.passport_json != passport_json
        || report["decisionHash"].as_str() != Some(&built.decision_hash.to_string())
    {
        return Err("PASSPORT_HASH_MISMATCH".into());
    }
    let safe: Address = safe.parse().map_err(|_| "INVALID_CONTEXT")?;
    let module: Address = module.parse().map_err(|_| "INVALID_CONTEXT")?;
    if report["source"] != "APP"
        || report["dryRun"] != false
        || passport.chain_id != 84532
        || passport.safe.parse::<Address>() != Ok(safe)
        || passport.module.parse::<Address>() != Ok(module)
        || passport.planner.kind != "mnn+ahem-g@android"
        || passport.planner.host != "device"
        || passport.choice_label == "H"
    {
        return Err("INVALID_CONTEXT_OR_HOLD".into());
    }
    let original: Snapshot =
        serde_json::from_value(report["snapshot"].clone()).map_err(|_| "INVALID_SNAPSHOT")?;
    let fresh: Snapshot = serde_json::from_str(fresh_json).map_err(|_| "INVALID_SNAPSHOT")?;
    let set: CandidateSet =
        serde_json::from_value(report["candidateSet"].clone()).map_err(|_| "INVALID_SET")?;
    let candidate: Candidate = serde_json::from_value(report["chosenCandidate"].clone())
        .map_err(|_| "INVALID_CANDIDATE")?;
    let summaries: Vec<CandidateSummary> = set
        .candidates
        .iter()
        .chain(std::iter::once(&set.hold))
        .map(|c| CandidateSummary {
            assets: c.action.assets,
            id: c.id,
            kind: c.kind.name().into(),
            label: c.label.clone(),
            reason: c.reason.clone(),
        })
        .collect();
    let passport_candidates: Vec<PassportCandidate> = summaries
        .iter()
        .map(|c| PassportCandidate {
            assets: c.assets.to_string(),
            id: c.id.to_string(),
            kind: c.kind.clone(),
            label: c.label.clone(),
            reason: c.reason.clone(),
        })
        .collect();
    if fusion_core::candidates::compute_candidate_set_digest(&summaries) != set.candidate_set_digest
        || passport.candidate_set_digest != set.candidate_set_digest.to_string()
        || passport.candidate_set != passport_candidates
        || passport.choice_id != candidate.id.to_string()
        || passport.choice_label != candidate.label
        || !set.candidates.contains(&candidate)
        || candidate.action.decision_hash != B256::ZERO
        || candidate.id
            != compute_candidate_id_from_action(
                U256::from(84532),
                safe,
                module,
                original.module_state.mandate_hash,
                original.module_state.session_id,
                original.block_hash,
                &candidate.action,
            )
    {
        return Err("CANDIDATE_BINDING_MISMATCH".into());
    }
    let model = &report["model"];
    let labels: Vec<String> = set
        .candidates
        .iter()
        .map(|c| c.label.clone())
        .chain(std::iter::once("H".into()))
        .collect();
    let raw = model["raw"].as_str().ok_or("INVALID_MODEL_OUTPUT")?;
    if strict_grammar::validate(raw, &labels).as_deref() != Some(&candidate.label)
        || model["label"] != candidate.label
        || model["rawValid"] != true
        || model["ahem"]["g"]["aborted"] != false
        || model["ahem"]["g"]["mode"] != "ahem-g"
        || model["candidateSetDigest"] != set.candidate_set_digest.to_string()
        || passport.planner.ahem.g.mode != "ENFORCED"
        || passport.planner.ahem.l.mode != "OFF"
    {
        return Err("INVALID_MODEL_OUTPUT".into());
    }
    if passport.snapshot.block != original.block_number
        || passport.snapshot.hash != original.block_hash.to_string()
        || passport.mandate_hash != original.module_state.mandate_hash.to_string()
        || fresh.block_number < original.block_number
        || fresh.block_number - original.block_number >= 5
        || fresh.block_timestamp < original.block_timestamp
        || fresh.module_state.mandate_hash != original.module_state.mandate_hash
        || fresh.module_state.action_nonce != original.module_state.action_nonce
        || fresh.module_state.policy_epoch != original.module_state.policy_epoch
        || candidate.route != Route::Instant
        || passport.route != "INSTANT"
    {
        return Err("STALE_STATE_OR_UNSUPPORTED_ROUTE".into());
    }
    let preview = fusion_core::preview::preview_candidate(
        &candidate,
        &fresh,
        built.decision_hash,
        U256::from(84532),
        safe,
        module,
    );
    if !preview.passed {
        return Err("FRESH_POLICY_PREVIEW_FAILED".into());
    }
    let call = fusion_core::encode::encode_execute(module, &preview.exact_action, &candidate.proof);
    Ok(
        json!({"ok":true,"to":call.to,"data":call.data,"actionDigest":preview.action_digest,
        "decisionHash":built.decision_hash,"action":preview.exact_action,"choiceLabel":candidate.label,"candidateId":candidate.id}),
    )
}

#[allow(dead_code)]
#[path = "../../../src/candidate_grammar.rs"]
mod strict_grammar;

#[cfg(test)]
mod android_boundary_tests {
    use super::*;
    use alloy::primitives::keccak256;
    use fusion_core::{
        candidates::{CandidateSet, CandidateSummary},
        passport::{DecisionPassport, PassportCandidate},
        types::{Obligation, Payee, Snapshot},
    };
    use std::collections::BTreeMap;

    // A synthetic policy-bound unit fixture derived from the captured public M2
    // snapshot. This is never labelled model evidence or broadcast on a network.
    fn fixture() -> (serde_json::Value, Snapshot, String, String) {
        let mut report: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../evidence/app/agent/m2_report.json"
        ))
        .unwrap();
        let mut snapshot: Snapshot = serde_json::from_value(report["snapshot"].clone()).unwrap();
        let bill = Obligation {
            obligation_id: "unit-bill".into(),
            obligation_id_hash: keccak256("unit-bill"),
            payee_id_hash: keccak256("unit-payee"),
            amount_raw: U256::from(2_000_000),
            due_at: snapshot.block_timestamp + 100,
        };
        snapshot.module_state.reserved_raw = bill.amount_raw;
        snapshot.module_state.obligations_root = fusion_core::hashing::compute_obligation_leaf(
            bill.obligation_id_hash,
            bill.payee_id_hash,
            bill.amount_raw,
            bill.due_at,
        );
        let payee = Payee {
            account: Address::repeat_byte(0xaa),
            label_hash: bill.payee_id_hash,
            active_at: 0,
            cancelled: false,
        };
        let mut passport: DecisionPassport =
            serde_json::from_str(report["passportJson"].as_str().unwrap()).unwrap();
        let safe: Address = passport.safe.parse().unwrap();
        let module: Address = passport.module.parse().unwrap();
        let set: CandidateSet = fusion_core::candidates::enumerate_candidates(
            &snapshot,
            &[bill],
            &[payee],
            &BTreeMap::new(),
            &[],
            U256::from(84532),
            safe,
            module,
            snapshot.module_state.mandate_hash,
            snapshot.module_state.session_id,
            true,
        );
        assert_eq!(set.candidates.len(), 1);
        let selected = &set.candidates[0];
        passport.candidate_set = set
            .candidates
            .iter()
            .chain(std::iter::once(&set.hold))
            .map(|c| {
                let s = CandidateSummary {
                    assets: c.action.assets,
                    id: c.id,
                    kind: c.kind.name().into(),
                    label: c.label.clone(),
                    reason: c.reason.clone(),
                };
                PassportCandidate {
                    assets: s.assets.to_string(),
                    id: s.id.to_string(),
                    kind: s.kind,
                    label: s.label,
                    reason: s.reason,
                }
            })
            .collect();
        passport.candidate_set_digest = set.candidate_set_digest.to_string();
        passport.choice_id = selected.id.to_string();
        passport.choice_label = "A".into();
        passport.reason_code = selected.reason.clone();
        passport.route = "INSTANT".into();
        let built = passport.build_output().unwrap();
        report["dryRun"] = json!(false);
        report["passportJson"] = json!(built.passport_json);
        report["decisionHash"] = json!(built.decision_hash.to_string());
        report["candidateSet"] = json!(set);
        report["chosenCandidate"] = json!(selected);
        report["snapshot"] = json!(snapshot);
        report["model"]["raw"] = json!(r#"{"choice":"A"}"#);
        report["model"]["label"] = json!("A");
        report["model"]["candidateSetDigest"] = json!(set.candidate_set_digest.to_string());
        (report, snapshot, safe.to_string(), module.to_string())
    }
    #[test]
    fn owner_allocation_is_nonce_expiry_amount_and_reserve_bound() {
        let (_, mut snapshot, safe, module) = fixture();
        snapshot.module_state.reserved_raw = U256::ZERO;
        snapshot.module_state.session_expires_at = snapshot.block_timestamp + 3600;
        let nonce = snapshot.module_state.action_nonce;
        let mut mandate = json!({"assetsRaw":"10000000", "depositNonce":nonce.to_string(),
            "vault":snapshot.policy.vaults[0].vault, "phase":"DEPOSIT",
            "expiresAt":snapshot.block_timestamp+60, "sessionId":snapshot.module_state.session_id,
            "policyHash":snapshot.module_state.policy_hash, "ownerTxHash":B256::repeat_byte(0xab)});
        let call = |snapshot: &Snapshot, mandate: &serde_json::Value| -> serde_json::Value {
            serde_json::from_str(&allocation_candidates(
                serde_json::to_string(snapshot).unwrap(),
                mandate.to_string(),
                safe.clone(),
                module.clone(),
            ))
            .unwrap()
        };
        let result = call(&snapshot, &mandate);
        assert_eq!(result["ok"], true);
        assert_eq!(
            result["set"]["candidates"][0]["action"]["assets"],
            "10000000"
        );
        assert_eq!(
            result["set"]["candidates"][0]["action"]["deadline"],
            snapshot.block_timestamp + 60
        );
        mandate["assetsRaw"] = json!("10000001");
        assert_eq!(call(&snapshot, &mandate)["ok"], false);
        mandate["assetsRaw"] = json!("10000000");
        snapshot.module_state.reserved_raw = U256::from(1);
        assert_eq!(call(&snapshot, &mandate)["ok"], false);
        snapshot.module_state.reserved_raw = U256::ZERO;
        snapshot.module_state.action_nonce += U256::from(1);
        assert_eq!(call(&snapshot, &mandate)["ok"], false);
        mandate["phase"] = json!("WITHDRAW");
        snapshot.vault_a_assets = U256::from(9_999_999);
        assert_eq!(
            call(&snapshot, &mandate)["set"]["candidates"][0]["action"]["assets"],
            "9999999"
        );
        snapshot.module_state.action_nonce += U256::from(1);
        assert_eq!(call(&snapshot, &mandate)["ok"], false);
    }

    #[test]
    fn erc4626_quotes_bind_real_share_units_and_reject_stale_or_bad_bounds() {
        let (report, snapshot, safe, module) = fixture();
        let mut set: CandidateSet = serde_json::from_value(report["candidateSet"].clone()).unwrap();
        let candidate = &mut set.candidates[0];
        candidate.kind = fusion_core::types::ActionKind::Deposit;
        candidate.action.kind = candidate.kind;
        candidate.action.destination_vault = snapshot.policy.vaults[0].vault;
        candidate.action.assets = U256::from(10_000_000);
        let original_digest = set.candidate_set_digest;
        let mut quotes = json!([{"vault": snapshot.policy.vaults[0].vault,
            "kind": 0, "assets": "10000000", "blockHash": snapshot.block_hash,
            "convertToShares": "8000001", "preview": "8000001"}]);
        let call = |set: &CandidateSet, quotes: &serde_json::Value| -> serde_json::Value {
            serde_json::from_str(&quote_candidates(
                serde_json::to_string(set).unwrap(),
                serde_json::to_string(&snapshot).unwrap(),
                quotes.to_string(),
                safe.clone(),
                module.clone(),
            ))
            .unwrap()
        };
        let result = call(&set, &quotes);
        assert_eq!(result["ok"], true);
        assert_eq!(
            result["set"]["candidates"][0]["action"]["minSharesOut"],
            "7960000"
        );
        assert_ne!(
            result["set"]["candidateSetDigest"],
            original_digest.to_string()
        );
        quotes[0]["preview"] = json!("7959999");
        assert_eq!(call(&set, &quotes)["ok"], false);
        quotes[0]["preview"] = json!("8000001");
        quotes[0]["blockHash"] = json!(B256::ZERO);
        assert_eq!(call(&set, &quotes)["ok"], false);
        quotes[0]["blockHash"] = json!(snapshot.block_hash);
        quotes[0]["kind"] = json!(1);
        set.candidates[0].kind = fusion_core::types::ActionKind::Withdraw;
        set.candidates[0].action.kind = set.candidates[0].kind;
        set.candidates[0].action.source_vault = snapshot.policy.vaults[0].vault;
        let result = call(&set, &quotes);
        assert_eq!(result["ok"], true);
        assert_eq!(
            result["set"]["candidates"][0]["action"]["maxSharesBurn"],
            "8000001"
        );
        quotes[0]["preview"] = json!("8040003");
        assert_eq!(call(&set, &quotes)["ok"], false);
    }

    #[test]
    fn native_signing_boundary_refuses_a12_and_accepts_real_app_calldata() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let golden: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("evidence/app/agent/golden_app.json")).unwrap(),
        )
        .unwrap();
        let tx: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(golden[0]["transaction"].as_str().unwrap()))
                .unwrap(),
        )
        .unwrap();
        let q = |field: &str| {
            u64::from_str_radix(tx[field].as_str().unwrap().trim_start_matches("0x"), 16).unwrap()
        };
        let mut prepared = json!({"chainId":84532,"nonce":q("nonce"),"to":tx["to"],
            "value":"0x0","data":tx["input"],"gasLimit":q("gas"),
            "maxFeePerGas":q("maxFeePerGas"),"maxPriorityFeePerGas":q("maxPriorityFeePerGas")});
        let policy = json!({"module":golden[0]["to"],"chainId":84532}).to_string();
        let signed: serde_json::Value = serde_json::from_str(&sign_prepared(
            prepared.to_string(),
            "42".repeat(32),
            policy.clone(),
        ))
        .unwrap();
        assert_eq!(signed["ok"], true);
        assert!(signed["rawTx"].as_str().unwrap().starts_with("0x02"));
        prepared["to"] = json!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        prepared["chainId"] = json!(31337);
        prepared["data"] = json!(format!(
            "0xa9059cbb{:0>64}{:064x}",
            "ee".repeat(20),
            99_000_000u64
        ));
        let refused: serde_json::Value = serde_json::from_str(&sign_prepared(
            prepared.to_string(),
            "42".repeat(32),
            policy,
        ))
        .unwrap();
        assert_eq!(refused["ok"], false);
        assert_eq!(refused["error"]["message"], "SignerRefused::NotModuleCall");
        assert!(refused.get("rawTx").is_none());
    }

    #[test]
    fn canonical_bound_decision_encodes_real_module_and_passport_hash() {
        let (report, snapshot, safe, module) = fixture();
        let output = verify_report(
            &report.to_string(),
            &serde_json::to_string(&snapshot).unwrap(),
            &safe,
            &module,
        )
        .unwrap();
        assert_eq!(
            output["to"].as_str().unwrap().parse::<Address>().unwrap(),
            module.parse::<Address>().unwrap()
        );
        assert!(output["data"].as_str().unwrap().contains(
            report["decisionHash"]
                .as_str()
                .unwrap()
                .trim_start_matches("0x")
        ));
    }
    #[test]
    fn modified_action_cannot_keep_old_candidate_or_passport_binding() {
        let (mut r, s, safe, module) = fixture();
        r["chosenCandidate"]["action"]["assets"] = json!("2000001");
        assert_eq!(
            verify_report(
                &r.to_string(),
                &serde_json::to_string(&s).unwrap(),
                &safe,
                &module
            )
            .unwrap_err(),
            "CANDIDATE_BINDING_MISMATCH"
        );
    }
    #[test]
    fn stale_five_block_result_is_rejected() {
        let (r, mut s, safe, module) = fixture();
        s.block_number += 5;
        assert_eq!(
            verify_report(
                &r.to_string(),
                &serde_json::to_string(&s).unwrap(),
                &safe,
                &module
            )
            .unwrap_err(),
            "STALE_STATE_OR_UNSUPPORTED_ROUTE"
        );
    }
    #[test]
    fn noncanonical_passport_bytes_are_rejected() {
        let (mut r, s, safe, module) = fixture();
        r["passportJson"] = json!(format!("{}\n", r["passportJson"].as_str().unwrap()));
        assert_eq!(
            verify_report(
                &r.to_string(),
                &serde_json::to_string(&s).unwrap(),
                &safe,
                &module
            )
            .unwrap_err(),
            "PASSPORT_HASH_MISMATCH"
        );
    }
    #[test]
    fn raw_output_with_extra_fields_is_rejected_independently() {
        let (mut r, s, safe, module) = fixture();
        r["model"]["raw"] = json!(r#"{"choice":"A","extra":"bypass"}"#);
        assert_eq!(
            verify_report(
                &r.to_string(),
                &serde_json::to_string(&s).unwrap(),
                &safe,
                &module
            )
            .unwrap_err(),
            "INVALID_MODEL_OUTPUT"
        );
    }
    #[test]
    fn hold_never_enters_signing_surface() {
        let r: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../evidence/app/agent/m2_report.json"
        ))
        .unwrap();
        let p: DecisionPassport =
            serde_json::from_str(r["passportJson"].as_str().unwrap()).unwrap();
        assert_eq!(
            verify_report(
                &r.to_string(),
                &r["snapshot"].to_string(),
                &p.safe,
                &p.module
            )
            .unwrap_err(),
            "INVALID_CONTEXT_OR_HOLD"
        );
    }
}
