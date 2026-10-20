//! Preview parity and rule coverage tests (§7.7, §10.7).

mod common;
use alloy::primitives::{Address, B256, U256};
use fusion_core::candidates::{Candidate, FeeAssumptions, Route, Tier};
use fusion_core::preview::{preview_candidate, RuleStatus};
use fusion_core::types::*;

fn sample_setup() -> (Snapshot, Candidate) {
    let vault_a = Address::from_slice(&[0xA1; 20]);
    let policy = Policy {
        permission_tier: PermissionTier::Autonomous,
        operating_buffer_raw: U256::from(500_000_000), // 500 USDC
        micro_max_raw: U256::from(10_000_000),         // 10 USDC
        delayed_max_raw: U256::from(100_000_000),      // 100 USDC
        hard_max_raw: U256::from(2_500_000_000_u64),   // 2,500 USDC
        daily_limit_raw: U256::from(4_000_000_000_u64), // 4,000 USDC
        delay_seconds: 180,
        max_share_slippage_bps: 50,
        guardian: Address::from_slice(&[0x07; 20]),
        vaults: vec![VaultLimit {
            vault: vault_a,
            max_exposure_raw: U256::from(6_000_000_000_u64),
        }],
        payee_delay_seconds: 3600,
        pay_early_window_seconds: 259_200,
    };

    let module_state = ModuleState {
        configured: true,
        paused: false,
        policy_epoch: U256::from(1),
        policy_hash: B256::repeat_byte(0x11),
        mandate_hash: B256::repeat_byte(0x22),
        obligations_root: B256::repeat_byte(0x33),
        reserved_raw: U256::from(583_000_000), // 583 USDC
        obligation_count: U256::from(1),
        action_nonce: U256::from(0),
        daily_day_index: U256::from(1),
        daily_used_raw: U256::ZERO,
        session_id: B256::repeat_byte(0x44),
        session_start_epoch: U256::from(1),
        session_key: Address::from_slice(&[0x5E; 20]),
        session_expires_at: 1_700_086_400,
        session_limit_raw: U256::from(8_000_000_000_u64),
        session_used_raw: U256::ZERO,
    };

    let snapshot = Snapshot {
        quant: None,
        block_number: 100,
        block_hash: B256::repeat_byte(0x99),
        block_timestamp: 1_700_000_000,
        safe_cash: U256::from(3_000_000_000_u64), // 3,000 USDC
        vault_a_shares: U256::ZERO,
        vault_a_assets: U256::ZERO,
        vault_b_shares: U256::ZERO,
        vault_b_assets: U256::ZERO,
        module_state,
        policy,
        queue: Queue {
            exists: false,
            action_digest: B256::ZERO,
            session_id: B256::ZERO,
            ready_at: 0,
            expires_at: 0,
            action: Action {
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
                expected_nonce: U256::ZERO,
                policy_epoch: U256::ZERO,
                deadline: 0,
                decision_hash: B256::ZERO,
            },
        },
    };

    let action = Action {
        kind: ActionKind::Deposit,
        source_vault: Address::ZERO,
        destination_vault: vault_a,
        assets: U256::from(10_000_000), // 10 USDC (instant)
        min_shares_out: U256::from(9_950_000),
        max_shares_burn: U256::ZERO,
        obligation_id_hash: B256::ZERO,
        payee_id_hash: B256::ZERO,
        recipient: Address::ZERO,
        due_at: 0,
        expected_nonce: U256::from(0),
        policy_epoch: U256::from(1),
        deadline: 1_700_000_120,
        decision_hash: B256::ZERO,
    };

    let candidate = Candidate {
        label: "A".to_string(),
        id: B256::repeat_byte(0xaa),
        kind: ActionKind::Deposit,
        action,
        proof: vec![],
        tier: Tier::Instant,
        route: Route::Instant,
        reason: "SURPLUS_ABOVE_THRESHOLD".to_string(),
        before_buckets: Default::default(),
        after_buckets: Default::default(),
        fee_assumptions: FeeAssumptions {
            fee_raw: U256::ZERO,
        },
    };

    let mut snapshot = snapshot;
    snapshot.quant = Some(common::measured(
        snapshot.block_number,
        snapshot.block_timestamp,
        vault_a,
        candidate.action.assets,
    ));
    (snapshot, candidate)
}

#[test]
fn test_preview_instant_deposit_passes_all_checks() {
    let (snapshot, candidate) = sample_setup();
    let decision_hash = B256::repeat_byte(0xdd);
    let chain_id = U256::from(84532);
    let safe = Address::from_slice(&[0x11; 20]);
    let module = Address::from_slice(&[0x22; 20]);

    let res = preview_candidate(&candidate, &snapshot, decision_hash, chain_id, safe, module);

    assert!(res.passed, "Valid candidate must pass preview");
    assert_eq!(res.exact_action.decision_hash, decision_hash);
    for check in &res.checks {
        assert_ne!(
            check.status,
            RuleStatus::Fail,
            "Check failed: {}: {:?}",
            check.name,
            check.error_code
        );
    }
}

#[test]
fn test_preview_catches_reserve_raid() {
    let (mut snapshot, mut candidate) = sample_setup();
    // Set cash such that C - assets < R + B
    // R = 583, B = 500, R + B = 1083 USDC.
    // Safe cash = 1085 USDC.
    // Deposit = 5 USDC (<= 10 USDC micro_max_raw, so Instant route).
    // Cash after = 1080 < 1083 -> ReserveViolation!
    snapshot.safe_cash = U256::from(1_085_000_000);
    candidate.action.assets = U256::from(5_000_000);

    let res = preview_candidate(
        &candidate,
        &snapshot,
        B256::repeat_byte(0xdd),
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
    );

    assert!(!res.passed);
    let fail = res
        .checks
        .iter()
        .find(|c| c.status == RuleStatus::Fail)
        .unwrap();
    assert_eq!(fail.error_code.as_deref(), Some("ReserveViolation"));
}

#[test]
fn test_preview_catches_paused_module() {
    let (mut snapshot, candidate) = sample_setup();
    snapshot.module_state.paused = true;

    let res = preview_candidate(
        &candidate,
        &snapshot,
        B256::repeat_byte(0xdd),
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
    );

    assert!(!res.passed);
    let fail = res
        .checks
        .iter()
        .find(|c| c.status == RuleStatus::Fail)
        .unwrap();
    assert_eq!(fail.error_code.as_deref(), Some("ModulePaused"));
}

#[test]
fn test_preview_catches_bad_nonce() {
    let (snapshot, mut candidate) = sample_setup();
    candidate.action.expected_nonce = U256::from(5); // Nonce on chain is 0

    let res = preview_candidate(
        &candidate,
        &snapshot,
        B256::repeat_byte(0xdd),
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
    );

    assert!(!res.passed);
    let fail = res
        .checks
        .iter()
        .find(|c| c.status == RuleStatus::Fail)
        .unwrap();
    assert_eq!(fail.error_code.as_deref(), Some("BadNonce"));
}

#[test]
fn test_preview_catches_missing_decision_hash() {
    let (snapshot, candidate) = sample_setup();

    let res = preview_candidate(
        &candidate,
        &snapshot,
        B256::ZERO, // Missing decision hash
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
    );

    assert!(!res.passed);
    let fail = res
        .checks
        .iter()
        .find(|c| c.status == RuleStatus::Fail)
        .unwrap();
    assert_eq!(fail.error_code.as_deref(), Some("MissingDecisionHash"));
}

#[test]
fn daily_usage_resets_at_snapshot_utc_midnight_but_session_usage_does_not() {
    let (mut snapshot, candidate) = sample_setup();
    snapshot.module_state.daily_used_raw = snapshot.policy.daily_limit_raw;
    snapshot.module_state.daily_day_index = U256::from(snapshot.block_timestamp / 86_400);
    let check = |s: &Snapshot| {
        preview_candidate(
            &candidate,
            s,
            B256::repeat_byte(0xdd),
            U256::from(84532),
            Address::repeat_byte(0x11),
            Address::repeat_byte(0x22),
        )
        .checks
        .into_iter()
        .find(|c| c.rule_id == "RULE_5_VELOCITY")
        .unwrap()
    };
    assert_eq!(check(&snapshot).status, RuleStatus::Fail);
    snapshot.module_state.daily_day_index -= U256::from(1);
    let reset = check(&snapshot);
    assert_eq!(reset.status, RuleStatus::Pass);
    assert!(reset.message.starts_with("daily_used=0/"));
    assert_eq!(
        snapshot.module_state.daily_used_raw,
        snapshot.policy.daily_limit_raw
    );
    snapshot.module_state.session_used_raw = snapshot.module_state.session_limit_raw;
    assert_eq!(check(&snapshot).status, RuleStatus::Fail);
}
