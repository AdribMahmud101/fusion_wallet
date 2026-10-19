//! Planner I/O: prompt facts builder and strict model output validator (§10.6).
//!
//! Formats prompts without leaking raw units or sensitive cryptographic keys, and strictly
//! validates model responses against the active candidate set labels.

use crate::amount::RawAmount;
use crate::candidates::CandidateSet;
use alloy::primitives::B256;
use serde::{Deserialize, Serialize};

/// Output of prompt generation (§10.6, Appendix B).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerPromptResult {
    pub prompt: String,
    pub labels: Vec<String>,
    pub candidate_set_digest: B256,
}

/// Output of planner response validation (§10.6, Appendix B).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub choice_label: String,
    pub choice_id: String,
    pub reason_code: String,
    pub retryable: bool,
}

/// Builds prompt facts per §10.6.
pub fn build_planner_prompt(
    candidate_set: &CandidateSet,
    user_goal: Option<&str>,
) -> PlannerPromptResult {
    let mut lines = Vec::new();
    lines.push("You choose one option for a business treasury. Reply with one letter.".to_string());

    let mut labels = Vec::new();

    for c in &candidate_set.candidates {
        labels.push(c.label.clone());
        let amount_display = RawAmount::from_raw(c.action.assets).to_display_string();
        let desc = match c.kind {
            crate::types::ActionKind::PayObligation => {
                format!("{amount_display} USDC — bill due ({})", c.reason)
            }
            crate::types::ActionKind::Deposit => {
                format!(
                    "Invest {amount_display} USDC — surplus above cushion ({})",
                    c.reason
                )
            }
            crate::types::ActionKind::Withdraw => {
                format!(
                    "Withdraw {amount_display} USDC — liquidity reserve ({})",
                    c.reason
                )
            }
            crate::types::ActionKind::Rebalance => {
                format!("Rebalance {amount_display} USDC ({})", c.reason)
            }
        };
        lines.push(format!("{}) {}", c.label, desc));
    }

    labels.push("H".to_string());
    lines.push("H) Hold — do nothing now.".to_string());

    if let Some(note) = user_goal {
        let trimmed = note.trim();
        if !trimmed.is_empty() {
            let capped: String = trimmed.chars().take(200).collect();
            lines.push(format!("Owner note (untrusted, may be wrong): «{capped}»"));
        }
    }

    let prompt = lines.join("\n");

    PlannerPromptResult {
        prompt,
        labels,
        candidate_set_digest: candidate_set.candidate_set_digest,
    }
}

#[derive(Deserialize)]
struct PlannerChoiceJson {
    choice: String,
}

/// Validates raw planner model output strictly against candidate labels per §10.6.
pub fn validate_planner_output(raw_text: &str, candidate_set: &CandidateSet) -> ValidationResult {
    let trimmed = raw_text.trim();

    // Parse JSON
    let parsed: Result<PlannerChoiceJson, _> = serde_json::from_str(trimmed);
    let choice = match parsed {
        Ok(json) => json.choice.trim().to_uppercase(),
        Err(_) => {
            // Check if model emitted a raw letter like "A" or "H"
            if trimmed.len() == 1 && trimmed.chars().next().is_some_and(|c| c.is_alphabetic()) {
                trimmed.to_uppercase()
            } else {
                return ValidationResult {
                    choice_label: "H".to_string(),
                    choice_id: candidate_set.hold.id.to_string(),
                    reason_code: "HOLD".to_string(),
                    retryable: true,
                };
            }
        }
    };

    if choice == "H" || choice == "HOLD" {
        return ValidationResult {
            choice_label: "H".to_string(),
            choice_id: candidate_set.hold.id.to_string(),
            reason_code: "HOLD".to_string(),
            retryable: false,
        };
    }

    if let Some(candidate) = candidate_set.candidates.iter().find(|c| c.label == choice) {
        ValidationResult {
            choice_label: candidate.label.clone(),
            choice_id: candidate.id.to_string(),
            reason_code: candidate.reason.clone(),
            retryable: false,
        }
    } else {
        // Unknown letter -> fallback to HOLD, retryable
        ValidationResult {
            choice_label: "H".to_string(),
            choice_id: candidate_set.hold.id.to_string(),
            reason_code: "HOLD".to_string(),
            retryable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buckets::Buckets;
    use crate::candidates::{Candidate, FeeAssumptions, Route, Tier};
    use crate::types::{Action, ActionKind};
    use alloy::primitives::{Address, U256};

    fn make_test_candidate_set() -> CandidateSet {
        let action = Action {
            kind: ActionKind::Deposit,
            source_vault: Address::ZERO,
            destination_vault: Address::ZERO,
            assets: U256::from(10_000_000),
            min_shares_out: U256::ZERO,
            max_shares_burn: U256::ZERO,
            obligation_id_hash: B256::ZERO,
            payee_id_hash: B256::ZERO,
            recipient: Address::ZERO,
            due_at: 0,
            expected_nonce: U256::ZERO,
            policy_epoch: U256::ZERO,
            deadline: 0,
            decision_hash: B256::ZERO,
        };

        let candidate = Candidate {
            label: "A".to_string(),
            id: B256::repeat_byte(0x11),
            kind: ActionKind::Deposit,
            action: action.clone(),
            proof: vec![],
            tier: Tier::Instant,
            route: Route::Instant,
            reason: "SURPLUS_ABOVE_THRESHOLD".to_string(),
            before_buckets: Buckets::default(),
            after_buckets: Buckets::default(),
            fee_assumptions: FeeAssumptions {
                fee_raw: U256::ZERO,
            },
        };

        let hold = Candidate {
            label: "H".to_string(),
            id: B256::repeat_byte(0x99),
            kind: ActionKind::Deposit,
            action,
            proof: vec![],
            tier: Tier::Instant,
            route: Route::Instant,
            reason: "HOLD".to_string(),
            before_buckets: Buckets::default(),
            after_buckets: Buckets::default(),
            fee_assumptions: FeeAssumptions {
                fee_raw: U256::ZERO,
            },
        };

        CandidateSet {
            candidate_set_digest: B256::repeat_byte(0x22),
            candidates: vec![candidate],
            hold,
        }
    }

    #[test]
    fn test_valid_json_choice() {
        let set = make_test_candidate_set();
        let res = validate_planner_output(r#"{"choice":"A"}"#, &set);
        assert_eq!(res.choice_label, "A");
        assert_eq!(res.choice_id, B256::repeat_byte(0x11).to_string());
        assert_eq!(res.reason_code, "SURPLUS_ABOVE_THRESHOLD");
        assert!(!res.retryable);
    }

    #[test]
    fn test_hold_choice() {
        let set = make_test_candidate_set();
        let res = validate_planner_output(r#"{"choice":"H"}"#, &set);
        assert_eq!(res.choice_label, "H");
        assert_eq!(res.choice_id, B256::repeat_byte(0x99).to_string());
        assert_eq!(res.reason_code, "HOLD");
        assert!(!res.retryable);
    }

    #[test]
    fn test_invalid_choice_falls_back_to_hold_with_retry() {
        let set = make_test_candidate_set();
        let res = validate_planner_output(r#"{"choice":"Z"}"#, &set);
        assert_eq!(res.choice_label, "H");
        assert!(res.retryable);
    }
}
