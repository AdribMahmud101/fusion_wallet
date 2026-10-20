//! Pure transaction preview engine mirroring smart contract checks (§7.7, §10.7).
//!
//! Evaluates every normative rule from `IKavachiModule.execute` before signing or broadcasting.
//! Operates with zero network calls, zero private keys, and zero nonce allocation.

use crate::buckets::Buckets;
use crate::candidates::{determine_route, determine_tier, Candidate, Route, Tier};
use crate::hashing::compute_action_digest;
use crate::merkle::MerkleTree;
use crate::types::{Action, ActionKind, PermissionTier, Snapshot};
use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

/// Outcome status of a specific validation rule (§10.7).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleStatus {
    Pass,
    Fail,
    NotApplicable,
}

/// Evaluation record for a single contract rule (§7.7, §10.7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleOutcome {
    pub rule_id: String,
    pub name: String,
    pub status: RuleStatus,
    pub error_code: Option<String>,
    pub message: String,
}

/// Comprehensive outcome of transaction preview (§10.7, Appendix B).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub passed: bool,
    pub action_digest: B256,
    pub exact_action: Action,
    pub tier: Tier,
    pub route: Route,
    pub before_buckets: Buckets,
    pub after_buckets: Buckets,
    pub checks: Vec<RuleOutcome>,
}

/// Runs all §7.7 execution checks against a candidate and on-chain snapshot.
pub fn preview_candidate(
    candidate: &Candidate,
    snapshot: &Snapshot,
    decision_hash: B256,
    chain_id: U256,
    safe: Address,
    module: Address,
) -> PreviewResult {
    preview(
        candidate,
        snapshot,
        decision_hash,
        chain_id,
        safe,
        module,
        false,
    )
}

/// Preview admission of a proposal through its required route, not permission to execute.
/// A delayed action with no active queue is checked for queue creation. All other
/// actions retain the execution preview checks, including an existing queue's delay.
pub fn preview_proposal(
    candidate: &Candidate,
    snapshot: &Snapshot,
    decision_hash: B256,
    chain_id: U256,
    safe: Address,
    module: Address,
) -> PreviewResult {
    let route = determine_route(
        determine_tier(candidate.action.assets, &snapshot.policy),
        snapshot.policy.permission_tier,
    );
    let queueing = matches!(route, Route::Queue | Route::ApproveAndQueue) && !snapshot.queue.exists;
    preview(
        candidate,
        snapshot,
        decision_hash,
        chain_id,
        safe,
        module,
        queueing,
    )
}

fn preview(
    candidate: &Candidate,
    snapshot: &Snapshot,
    decision_hash: B256,
    chain_id: U256,
    safe: Address,
    module: Address,
    queueing: bool,
) -> PreviewResult {
    let mut exact_action = candidate.action.clone();
    exact_action.decision_hash = decision_hash;

    let action_digest = compute_action_digest(
        chain_id,
        safe,
        module,
        snapshot.module_state.mandate_hash,
        snapshot.module_state.session_id,
        &exact_action,
    );

    let mut checks = Vec::new();
    let now = snapshot.block_timestamp;
    let state = &snapshot.module_state;
    let policy = &snapshot.policy;

    // Check 1: Lifecycle and permissions (§7.7.1)
    let not_paused = !state.paused;
    let tier_not_read_only = policy.permission_tier != PermissionTier::ReadOnly;
    let session_live = state.session_key != Address::ZERO && now <= state.session_expires_at;

    checks.push(RuleOutcome {
        rule_id: "RULE_1_LIFECYCLE".to_string(),
        name: "Module live and authorized".to_string(),
        status: if not_paused && tier_not_read_only && session_live {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: if !not_paused {
            Some("ModulePaused".to_string())
        } else if !tier_not_read_only {
            Some("ReadOnly".to_string())
        } else if !session_live {
            Some("SessionExpired".to_string())
        } else {
            None
        },
        message: format!(
            "paused={not_paused}, tier={:?}, session_live={session_live}",
            policy.permission_tier
        ),
    });

    // Check 2: Queue gate (§7.7.2)
    let queue_ok = if snapshot.queue.exists {
        snapshot.queue.action_digest == action_digest
    } else {
        true
    };

    checks.push(RuleOutcome {
        rule_id: "RULE_2_QUEUE_GATE".to_string(),
        name: "Queue availability".to_string(),
        status: if queue_ok {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: if !queue_ok {
            Some("QueueBusy".to_string())
        } else {
            None
        },
        message: if snapshot.queue.exists {
            "Active queue matches digest".to_string()
        } else {
            "No active queue blocking".to_string()
        },
    });

    // Check 3: Nonce, Epoch, Deadline & DecisionHash (§7.7.3)
    let nonce_ok = exact_action.expected_nonce == state.action_nonce;
    let epoch_ok = exact_action.policy_epoch == state.policy_epoch;
    let deadline_ok = exact_action.deadline >= now
        && exact_action.deadline <= state.session_expires_at
        && (!queueing
            || (exact_action.deadline >= now.saturating_add(policy.delay_seconds as u64 + 30)
                && exact_action.deadline <= now.saturating_add(policy.delay_seconds as u64 + 900)));
    let decision_hash_ok = exact_action.decision_hash != B256::ZERO;

    let check_3_passed = nonce_ok && epoch_ok && deadline_ok && decision_hash_ok;
    checks.push(RuleOutcome {
        rule_id: "RULE_3_METADATA".to_string(),
        name: "Nonce, Epoch, Deadline, and DecisionHash validity".to_string(),
        status: if check_3_passed {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: if !nonce_ok {
            Some("BadNonce".to_string())
        } else if !epoch_ok {
            Some("StalePolicy".to_string())
        } else if !deadline_ok {
            Some("ActionExpired".to_string())
        } else if !decision_hash_ok {
            Some("MissingDecisionHash".to_string())
        } else {
            None
        },
        message: format!(
            "nonce_ok={nonce_ok}, epoch_ok={epoch_ok}, deadline_ok={deadline_ok}, decision_hash_ok={decision_hash_ok}"
        ),
    });

    // Check 4: Shape and limits (§7.7.4)
    let shape_ok = exact_action.kind != ActionKind::Rebalance; // Rebalance reverts UnsupportedAction at S phase
    let assets_positive = exact_action.assets > U256::ZERO
        || exact_action.assets == U256::ZERO && candidate.reason == "HOLD";
    let within_hard_max = exact_action.assets <= policy.hard_max_raw;

    let check_4_passed = shape_ok && assets_positive && within_hard_max;
    checks.push(RuleOutcome {
        rule_id: "RULE_4_SHAPE".to_string(),
        name: "Action shape and hard max limit".to_string(),
        status: if check_4_passed {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: if !shape_ok {
            Some("UnsupportedAction".to_string())
        } else if !within_hard_max {
            Some("HardLimitExceeded".to_string())
        } else if !assets_positive {
            Some("InvalidAmount".to_string())
        } else {
            None
        },
        message: format!(
            "assets={}, hard_max={}",
            exact_action.assets, policy.hard_max_raw
        ),
    });

    // Check 5: Velocity limits (§7.7.5)
    let daily_used = if state.daily_day_index == U256::from(snapshot.block_timestamp / 86_400) {
        state.daily_used_raw
    } else {
        U256::ZERO
    };
    let daily_ok = daily_used.saturating_add(exact_action.assets) <= policy.daily_limit_raw;
    let session_ok =
        state.session_used_raw.saturating_add(exact_action.assets) <= state.session_limit_raw;

    let check_5_passed = daily_ok && session_ok;
    checks.push(RuleOutcome {
        rule_id: "RULE_5_VELOCITY".to_string(),
        name: "Daily and session usage caps".to_string(),
        status: if check_5_passed {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: if !daily_ok {
            Some("DailyLimitExceeded".to_string())
        } else if !session_ok {
            Some("SessionLimitExceeded".to_string())
        } else {
            None
        },
        message: format!(
            "daily_used={}/{}, session_used={}/{}",
            daily_used, policy.daily_limit_raw, state.session_used_raw, state.session_limit_raw
        ),
    });

    // Check 6: Tier Route Route satisfaction (§7.7.6)
    let tier = determine_tier(exact_action.assets, policy);
    let route = determine_route(tier, policy.permission_tier);

    let (route_ok, route_err) = match route {
        Route::Instant => (true, None),
        Route::Queue if queueing => (true, None),
        Route::Queue => {
            if !snapshot.queue.exists {
                (false, Some("QueueRequired".to_string()))
            } else if snapshot.queue.action_digest != action_digest {
                (false, Some("QueueBusy".to_string()))
            } else if snapshot.queue.session_id != state.session_id {
                (false, Some("QueueMismatch".to_string()))
            } else if now > snapshot.queue.expires_at {
                (false, Some("QueueExpired".to_string()))
            } else if now < snapshot.queue.ready_at {
                (false, Some("QueueNotReady".to_string()))
            } else {
                (true, None)
            }
        }
        Route::Approve | Route::ApproveAndQueue => (true, None),
        Route::Reject => (false, Some("ApprovalRequired".to_string())),
    };

    checks.push(RuleOutcome {
        rule_id: "RULE_6_TIER_ROUTE".to_string(),
        name: "Tier and route conditions".to_string(),
        status: if route_ok {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: route_err,
        message: format!("tier={tier:?}, route={route:?}"),
    });

    // Check 7: Kind-specific rules (§7.7.7)
    let is_underfunded = snapshot.safe_cash < state.reserved_raw;
    let mut check_7_passed = true;
    let mut check_7_error = None;
    let mut check_7_msg = String::new();

    match exact_action.kind {
        ActionKind::Deposit => {
            if is_underfunded {
                check_7_passed = false;
                check_7_error = Some("Underfunded".to_string());
                check_7_msg = "Cannot deposit when treasury cash is below reserve".to_string();
            } else {
                let rb = state
                    .reserved_raw
                    .saturating_add(policy.operating_buffer_raw);
                if snapshot.safe_cash.saturating_sub(exact_action.assets) < rb {
                    check_7_passed = false;
                    check_7_error = Some("ReserveViolation".to_string());
                    check_7_msg = "Cash after deposit would drop below R + B".to_string();
                } else {
                    check_7_msg = "Deposit satisfies reserve cushion".to_string();
                }
            }
        }
        ActionKind::Withdraw => {
            check_7_msg = "Withdraw provides cash liquidity".to_string();
        }
        ActionKind::PayObligation => {
            if snapshot.safe_cash < exact_action.assets {
                check_7_passed = false;
                check_7_error = Some("Underfunded".to_string());
                check_7_msg = "Insufficient cash to pay obligation".to_string();
            } else if !candidate.proof.is_empty() {
                let leaf = crate::hashing::compute_obligation_leaf(
                    exact_action.obligation_id_hash,
                    exact_action.payee_id_hash,
                    exact_action.assets,
                    exact_action.due_at,
                );
                if !MerkleTree::verify(&candidate.proof, state.obligations_root, leaf) {
                    check_7_passed = false;
                    check_7_error = Some("BadObligationProof".to_string());
                    check_7_msg = "Merkle proof verification failed".to_string();
                } else {
                    check_7_msg = "Bill Merkle proof verified against commitment root".to_string();
                }
            }
        }
        ActionKind::Rebalance => {
            check_7_passed = false;
            check_7_error = Some("UnsupportedAction".to_string());
            check_7_msg = "Rebalance unsupported in S phase".to_string();
        }
    }

    checks.push(RuleOutcome {
        rule_id: "RULE_7_KIND_SPECIFIC".to_string(),
        name: "Action kind invariant checks".to_string(),
        status: if check_7_passed {
            RuleStatus::Pass
        } else {
            RuleStatus::Fail
        },
        error_code: check_7_error,
        message: check_7_msg,
    });

    if candidate.reason != "HOLD"
        && matches!(
            candidate.kind,
            ActionKind::Deposit | ActionKind::Rebalance | ActionKind::Withdraw
        )
    {
        let quant = crate::quant::QuantReport::read(snapshot);
        let status = if candidate.kind == ActionKind::Withdraw {
            snapshot
                .quant
                .as_ref()
                .is_some_and(|q| q.vault == candidate.action.source_vault)
                && quant
                    .withdrawable_now_raw
                    .as_deref()
                    .and_then(|s| s.parse::<U256>().ok())
                    .is_some_and(|a| candidate.action.assets <= a)
        } else {
            let current = if candidate.kind == ActionKind::Rebalance {
                u64::MAX
            } else {
                0
            };
            // Rebalances remain unsupported; a current-source yield measurement is
            // required before that route can ever be enabled.
            quant
                .evaluate(
                    candidate.action.destination_vault,
                    candidate.action.assets,
                    current,
                    snapshot.quant.as_ref(),
                )
                .status
                == "ACTIONABLE"
        };
        checks.push(RuleOutcome {
            rule_id: "RULE_QUANT".into(),
            name: "Measured pool liquidity and economics".into(),
            status: if status {
                RuleStatus::Pass
            } else {
                RuleStatus::Fail
            },
            error_code: (!status).then(|| "QuantGuard".into()),
            message: if status {
                "Measured quant guard passed"
            } else {
                "Yield move unmeasured, unprofitable after gas, or pool liquidity unavailable"
            }
            .into(),
        });
    }
    let all_passed = checks.iter().all(|c| c.status != RuleStatus::Fail);

    PreviewResult {
        passed: all_passed,
        action_digest,
        exact_action,
        tier,
        route,
        before_buckets: candidate.before_buckets,
        after_buckets: candidate.after_buckets,
        checks,
    }
}
