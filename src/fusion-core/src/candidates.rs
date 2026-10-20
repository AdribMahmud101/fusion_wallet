//! Core algorithm: candidate enumeration and economics (§10.4).
//!
//! Deterministically produces <= 7 candidates plus HOLD (label H).
//! All economics calculations strictly use integer math (no floating point).

use crate::buckets::Buckets;
use crate::hashing::compute_candidate_id_from_action;
use crate::types::{
    deserialize_u256, serialize_u256, Action, ActionKind, Obligation, Payee, PermissionTier,
    Policy, Snapshot,
};
use alloy::primitives::{keccak256, Address, B256, U256};
use serde::{Deserialize, Serialize};

/// Tiers for action amounts (§10.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Tier {
    Instant,
    Delayed,
    Owner,
    Reject,
}

/// Execution route required by policy and tier (§10.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Route {
    Instant,
    Queue,
    Approve,
    ApproveAndQueue,
    Reject,
}

/// Determines the amount tier (§10.4).
pub fn determine_tier(assets: U256, policy: &Policy) -> Tier {
    if assets <= policy.micro_max_raw {
        Tier::Instant
    } else if assets <= policy.delayed_max_raw {
        Tier::Delayed
    } else if assets <= policy.hard_max_raw {
        Tier::Owner
    } else {
        Tier::Reject
    }
}

/// Determines the execution route (§10.4).
pub fn determine_route(tier: Tier, permission_tier: PermissionTier) -> Route {
    match permission_tier {
        PermissionTier::ReadOnly => Route::Reject,
        PermissionTier::OwnerApproved => match tier {
            Tier::Instant | Tier::Owner => Route::Approve,
            Tier::Delayed => Route::ApproveAndQueue,
            Tier::Reject => Route::Reject,
        },
        PermissionTier::Autonomous => match tier {
            Tier::Instant => Route::Instant,
            Tier::Delayed => Route::Queue,
            Tier::Owner => Route::Approve,
            Tier::Reject => Route::Reject,
        },
    }
}

/// Fee and yield scenario assumptions (§10.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeAssumptions {
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub fee_raw: U256,
}

/// Candidate action proposed by the core engine (§10.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub label: String,
    pub id: B256,
    pub kind: ActionKind,
    pub action: Action,
    pub proof: Vec<B256>,
    pub tier: Tier,
    pub route: Route,
    pub reason: String,
    pub before_buckets: Buckets,
    pub after_buckets: Buckets,
    pub fee_assumptions: FeeAssumptions,
}

/// Summary of a candidate for inclusion in passports and candidate digests (§10.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateSummary {
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub assets: U256,
    pub id: B256,
    pub kind: String,
    pub label: String,
    pub reason: String,
}

/// Full candidate set evaluated by the planner (§10.4, §10.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateSet {
    pub candidate_set_digest: B256,
    pub candidates: Vec<Candidate>,
    pub hold: Candidate,
}

/// K25 policy proposal, before the model's veto. Enumeration already orders
/// committed, funded bills to active payees by due time and obligation hash.
/// Keep the original labels/IDs/digest, skip rejected tiers/routes, and never
/// let untrusted notes or model output rank or modify candidates.
/// This experiment deliberately uses bills, then surplus deposits, then HOLD;
/// reserve withdrawals remain available to the default chooser only.
pub fn policy_proposal(set: &CandidateSet) -> &Candidate {
    set.candidates
        .iter()
        .find(|c| {
            c.kind == ActionKind::PayObligation
                && c.tier != Tier::Reject
                && c.route != Route::Reject
        })
        .or_else(|| {
            set.candidates.iter().find(|c| {
                c.kind == ActionKind::Deposit && c.tier != Tier::Reject && c.route != Route::Reject
            })
        })
        .unwrap_or(&set.hold)
}

/// Rate information for vault economics (§10.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultRateInfo {
    pub vault: Address,
    pub rate_bps: u64,
    pub is_simulated: bool,
}

const SECONDS_PER_YEAR: u64 = 31_536_000;
const HORIZON_SECONDS: u64 = 30 * 86_400; // 30 days = 2,592,000 s

/// Computes integer net benefit over a 30-day horizon (§10.4).
/// gross = floor(assets * rateBps * horizonSec / (10_000 * 31_536_000))
/// net = gross - feeRaw
pub fn compute_net_benefit(assets: U256, rate_bps: u64, fee_raw: U256) -> U256 {
    let numerator = assets
        .saturating_mul(U256::from(rate_bps))
        .saturating_mul(U256::from(HORIZON_SECONDS));
    let denominator = U256::from(10_000).saturating_mul(U256::from(SECONDS_PER_YEAR));
    let gross = numerator / denominator;
    gross.saturating_sub(fee_raw)
}

/// Computes candidateSetDigest by hashing the canonical JSON summary of the candidate set.
pub fn compute_candidate_set_digest(summaries: &[CandidateSummary]) -> B256 {
    let json = serde_json::to_string(summaries).unwrap_or_default();
    keccak256(json.as_bytes())
}

/// Enumerates candidates deterministically per §10.4.
#[allow(clippy::too_many_arguments)]
pub fn enumerate_candidates(
    snapshot: &Snapshot,
    bills: &[Obligation],
    payees: &[Payee],
    bill_proofs: &std::collections::BTreeMap<B256, Vec<B256>>,
    rates: &[VaultRateInfo],
    chain_id: U256,
    safe: Address,
    module: Address,
    mandate_hash: B256,
    session_id: B256,
    is_full_profile: bool,
) -> CandidateSet {
    let now = snapshot.block_timestamp;
    let buckets_before = Buckets::from_snapshot(snapshot);
    let mut candidates: Vec<Candidate> = Vec::new();

    let labels = ["A", "B", "C", "D", "E", "F", "G"];
    let min_net = if is_full_profile {
        U256::from(1_000_000) // 1 USDC
    } else {
        U256::from(10_000) // 0.01 USDC
    };
    let fee_assumption_raw = U256::from(50_000); // 0.05 USDC scenario fee

    // 1. Due bills: committed unpaid bills with now >= dueAt - payEarlyWindow,
    // payee ACTIVE and not sanctioned, C >= amount.
    // Order: earliest dueAt, then obligationIdHash.
    let mut eligible_bills = Vec::new();
    for bill in bills {
        let window = snapshot.policy.pay_early_window_seconds as u64;
        let early_due = bill.due_at.saturating_sub(window);
        if now < early_due {
            continue;
        }

        // Check if payee is active and not cancelled
        let payee_match = payees.iter().find(|p| p.label_hash == bill.payee_id_hash);
        if let Some(payee) = payee_match {
            if payee.cancelled || now < payee.active_at {
                continue;
            }
            if snapshot.safe_cash >= bill.amount_raw {
                eligible_bills.push((bill, payee));
            }
        }
    }

    // Sort by earliest dueAt, then obligationIdHash
    eligible_bills.sort_by(|(b1, _), (b2, _)| {
        b1.due_at
            .cmp(&b2.due_at)
            .then_with(|| b1.obligation_id_hash.cmp(&b2.obligation_id_hash))
    });

    for (bill, payee) in eligible_bills {
        if candidates.len() >= 7 {
            break;
        }
        let label = labels[candidates.len()].to_string();
        let proof = bill_proofs
            .get(&bill.obligation_id_hash)
            .cloned()
            .unwrap_or_default();

        let action = Action {
            kind: ActionKind::PayObligation,
            source_vault: Address::ZERO,
            destination_vault: Address::ZERO,
            assets: bill.amount_raw,
            min_shares_out: U256::ZERO,
            max_shares_burn: U256::ZERO,
            obligation_id_hash: bill.obligation_id_hash,
            payee_id_hash: bill.payee_id_hash,
            recipient: payee.account,
            due_at: bill.due_at,
            expected_nonce: snapshot.module_state.action_nonce,
            policy_epoch: snapshot.module_state.policy_epoch,
            deadline: now + 120,
            decision_hash: B256::ZERO,
        };

        let tier = determine_tier(action.assets, &snapshot.policy);
        let route = determine_route(tier, snapshot.policy.permission_tier);
        let id = compute_candidate_id_from_action(
            chain_id,
            safe,
            module,
            mandate_hash,
            session_id,
            snapshot.block_hash,
            &action,
        );

        let after_cash = snapshot.safe_cash.saturating_sub(bill.amount_raw);
        let after_reserved = snapshot
            .module_state
            .reserved_raw
            .saturating_sub(bill.amount_raw);
        let buckets_after = Buckets::compute(
            after_cash,
            after_reserved,
            snapshot.policy.operating_buffer_raw,
            buckets_before.deployed,
        );

        candidates.push(Candidate {
            label,
            id,
            kind: ActionKind::PayObligation,
            action,
            proof,
            tier,
            route,
            reason: "BILL_DUE".to_string(),
            before_buckets: buckets_before,
            after_buckets: buckets_after,
            fee_assumptions: FeeAssumptions {
                fee_raw: fee_assumption_raw,
            },
        });
    }

    // 2. Safety withdraw: if C < R + B -> WITHDRAW min(R + B - C, maxWithdraw)
    // from vault with lowest rate first.
    let target_cash = snapshot
        .module_state
        .reserved_raw
        .saturating_add(snapshot.policy.operating_buffer_raw);
    if snapshot.safe_cash < target_cash && candidates.len() < 7 {
        let deficit = target_cash - snapshot.safe_cash;

        // Find vault with lowest rate that has assets
        let mut sorted_rates = rates.to_vec();
        sorted_rates.sort_by_key(|r| r.rate_bps);

        for rate_info in sorted_rates {
            let vault_assets = if rate_info.vault
                == snapshot
                    .policy
                    .vaults
                    .first()
                    .map(|v| v.vault)
                    .unwrap_or_default()
            {
                snapshot.vault_a_assets
            } else {
                snapshot.vault_b_assets
            };

            if vault_assets > U256::ZERO {
                let withdraw_amount = deficit.min(vault_assets);
                if withdraw_amount > U256::ZERO {
                    let label = labels[candidates.len()].to_string();
                    let action = Action {
                        kind: ActionKind::Withdraw,
                        source_vault: rate_info.vault,
                        destination_vault: Address::ZERO,
                        assets: withdraw_amount,
                        min_shares_out: U256::ZERO,
                        max_shares_burn: withdraw_amount
                            .saturating_add(withdraw_amount / U256::from(100)),
                        obligation_id_hash: B256::ZERO,
                        payee_id_hash: B256::ZERO,
                        recipient: Address::ZERO,
                        due_at: 0,
                        expected_nonce: snapshot.module_state.action_nonce,
                        policy_epoch: snapshot.module_state.policy_epoch,
                        deadline: now + 120,
                        decision_hash: B256::ZERO,
                    };

                    let tier = determine_tier(action.assets, &snapshot.policy);
                    let route = determine_route(tier, snapshot.policy.permission_tier);
                    let id = compute_candidate_id_from_action(
                        chain_id,
                        safe,
                        module,
                        mandate_hash,
                        session_id,
                        snapshot.block_hash,
                        &action,
                    );

                    let buckets_after = Buckets::compute(
                        snapshot.safe_cash.saturating_add(withdraw_amount),
                        snapshot.module_state.reserved_raw,
                        snapshot.policy.operating_buffer_raw,
                        buckets_before.deployed.saturating_sub(withdraw_amount),
                    );

                    candidates.push(Candidate {
                        label,
                        id,
                        kind: ActionKind::Withdraw,
                        action,
                        proof: vec![],
                        tier,
                        route,
                        reason: "DEFICIT_BELOW_RESERVE".to_string(),
                        before_buckets: buckets_before,
                        after_buckets: buckets_after,
                        fee_assumptions: FeeAssumptions {
                            fee_raw: fee_assumption_raw,
                        },
                    });
                    break;
                }
            }
        }
    }

    // 3. Invest surplus: if sweepable > 0 -> DEPOSIT min(sweepable, cap headroom)
    // into each eligible vault, if netBenefit > minNet.
    if buckets_before.sweepable > U256::ZERO && candidates.len() < 7 {
        for vault_limit in &snapshot.policy.vaults {
            if candidates.len() >= 7 {
                break;
            }

            let current_exposure = if vault_limit.vault
                == snapshot
                    .policy
                    .vaults
                    .first()
                    .map(|v| v.vault)
                    .unwrap_or_default()
            {
                snapshot.vault_a_assets
            } else {
                snapshot.vault_b_assets
            };

            let headroom = vault_limit
                .max_exposure_raw
                .saturating_sub(current_exposure);
            // Do not manufacture a REJECT candidate when a smaller policy-valid
            // allocation exists. Keep the economics test on the actual amount.
            let deposit_amount = buckets_before
                .sweepable
                .min(headroom)
                .min(snapshot.policy.hard_max_raw);

            if deposit_amount > U256::ZERO {
                let rate_bps = rates
                    .iter()
                    .find(|r| r.vault == vault_limit.vault)
                    .map(|r| r.rate_bps)
                    .unwrap_or(0);

                let net_benefit = compute_net_benefit(deposit_amount, rate_bps, fee_assumption_raw);
                if net_benefit >= min_net {
                    let label = labels[candidates.len()].to_string();
                    let slippage_bps = snapshot.policy.max_share_slippage_bps as u64;
                    let min_shares = deposit_amount.saturating_sub(
                        deposit_amount * U256::from(slippage_bps) / U256::from(10_000),
                    );

                    let action = Action {
                        kind: ActionKind::Deposit,
                        source_vault: Address::ZERO,
                        destination_vault: vault_limit.vault,
                        assets: deposit_amount,
                        min_shares_out: min_shares,
                        max_shares_burn: U256::ZERO,
                        obligation_id_hash: B256::ZERO,
                        payee_id_hash: B256::ZERO,
                        recipient: Address::ZERO,
                        due_at: 0,
                        expected_nonce: snapshot.module_state.action_nonce,
                        policy_epoch: snapshot.module_state.policy_epoch,
                        deadline: now + 120,
                        decision_hash: B256::ZERO,
                    };

                    let tier = determine_tier(action.assets, &snapshot.policy);
                    let route = determine_route(tier, snapshot.policy.permission_tier);
                    let id = compute_candidate_id_from_action(
                        chain_id,
                        safe,
                        module,
                        mandate_hash,
                        session_id,
                        snapshot.block_hash,
                        &action,
                    );

                    let buckets_after = Buckets::compute(
                        snapshot.safe_cash.saturating_sub(deposit_amount),
                        snapshot.module_state.reserved_raw,
                        snapshot.policy.operating_buffer_raw,
                        buckets_before.deployed.saturating_add(deposit_amount),
                    );

                    candidates.push(Candidate {
                        label,
                        id,
                        kind: ActionKind::Deposit,
                        action,
                        proof: vec![],
                        tier,
                        route,
                        reason: "SURPLUS_ABOVE_THRESHOLD".to_string(),
                        before_buckets: buckets_before,
                        after_buckets: buckets_after,
                        fee_assumptions: FeeAssumptions {
                            fee_raw: fee_assumption_raw,
                        },
                    });
                }
            }
        }
    }

    // 4. HOLD (always present, label H)
    let hold_action = Action {
        kind: ActionKind::Deposit,
        source_vault: Address::ZERO,
        destination_vault: Address::ZERO,
        assets: U256::ZERO,
        min_shares_out: U256::ZERO,
        max_shares_burn: U256::ZERO,
        obligation_id_hash: B256::ZERO,
        payee_id_hash: B256::ZERO,
        recipient: Address::ZERO,
        due_at: 0,
        expected_nonce: snapshot.module_state.action_nonce,
        policy_epoch: snapshot.module_state.policy_epoch,
        deadline: now + 120,
        decision_hash: B256::ZERO,
    };
    let hold_id = compute_candidate_id_from_action(
        chain_id,
        safe,
        module,
        mandate_hash,
        session_id,
        snapshot.block_hash,
        &hold_action,
    );

    let hold = Candidate {
        label: "H".to_string(),
        id: hold_id,
        kind: ActionKind::Deposit,
        action: hold_action,
        proof: vec![],
        tier: Tier::Instant,
        route: Route::Instant,
        reason: "HOLD".to_string(),
        before_buckets: buckets_before,
        after_buckets: buckets_before,
        fee_assumptions: FeeAssumptions {
            fee_raw: U256::ZERO,
        },
    };

    // Build summaries for candidateSetDigest
    let mut summaries: Vec<CandidateSummary> = candidates
        .iter()
        .map(|c| CandidateSummary {
            assets: c.action.assets,
            id: c.id,
            kind: c.kind.name().to_string(),
            label: c.label.clone(),
            reason: c.reason.clone(),
        })
        .collect();

    summaries.push(CandidateSummary {
        assets: hold.action.assets,
        id: hold.id,
        kind: hold.kind.name().to_string(),
        label: hold.label.clone(),
        reason: hold.reason.clone(),
    });

    let candidate_set_digest = compute_candidate_set_digest(&summaries);

    CandidateSet {
        candidate_set_digest,
        candidates,
        hold,
    }
}
