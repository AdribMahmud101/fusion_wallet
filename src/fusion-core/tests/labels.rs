//! Candidate label mapping and prompt facts tests (D14, §10.4, §10.6).

use alloy::primitives::{Address, B256, U256};
use fusion_core::candidates::{enumerate_candidates, VaultRateInfo};
use fusion_core::planner_io::{build_planner_prompt, validate_planner_output};
use fusion_core::types::*;
use std::collections::BTreeMap;

#[test]
fn test_hold_is_always_label_h() {
    let policy = Policy {
        permission_tier: PermissionTier::Autonomous,
        operating_buffer_raw: U256::from(500_000_000),
        micro_max_raw: U256::from(10_000_000),
        delayed_max_raw: U256::from(100_000_000),
        hard_max_raw: U256::from(2_500_000_000_u64),
        daily_limit_raw: U256::from(4_000_000_000_u64),
        delay_seconds: 180,
        max_share_slippage_bps: 50,
        guardian: Address::ZERO,
        vaults: vec![],
        payee_delay_seconds: 3600,
        pay_early_window_seconds: 259_200,
    };

    let snapshot = Snapshot {
        block_number: 1,
        block_hash: B256::ZERO,
        block_timestamp: 1000,
        safe_cash: U256::ZERO,
        vault_a_shares: U256::ZERO,
        vault_a_assets: U256::ZERO,
        vault_b_shares: U256::ZERO,
        vault_b_assets: U256::ZERO,
        module_state: ModuleState {
            configured: true,
            paused: false,
            policy_epoch: U256::from(1),
            policy_hash: B256::ZERO,
            mandate_hash: B256::ZERO,
            obligations_root: B256::ZERO,
            reserved_raw: U256::ZERO,
            obligation_count: U256::ZERO,
            action_nonce: U256::ZERO,
            daily_day_index: U256::from(1),
            daily_used_raw: U256::ZERO,
            session_id: B256::ZERO,
            session_start_epoch: U256::from(1),
            session_key: Address::ZERO,
            session_expires_at: 0,
            session_limit_raw: U256::ZERO,
            session_used_raw: U256::ZERO,
        },
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

    let set = enumerate_candidates(
        &snapshot,
        &[],
        &[],
        &BTreeMap::new(),
        &[],
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
        B256::ZERO,
        B256::ZERO,
        true,
    );

    assert_eq!(set.hold.label, "H");
}

#[test]
fn test_prompt_facts_contain_no_addresses_or_hashes() {
    let vault_a = Address::from_slice(&[0xA1; 20]);
    let policy = Policy {
        permission_tier: PermissionTier::Autonomous,
        operating_buffer_raw: U256::from(500_000_000),
        micro_max_raw: U256::from(10_000_000),
        delayed_max_raw: U256::from(100_000_000),
        hard_max_raw: U256::from(2_500_000_000_u64),
        daily_limit_raw: U256::from(4_000_000_000_u64),
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

    let snapshot = Snapshot {
        block_number: 100,
        block_hash: B256::repeat_byte(0x33),
        block_timestamp: 1_700_000_000,
        safe_cash: U256::from(3_000_000_000_u64),
        vault_a_shares: U256::ZERO,
        vault_a_assets: U256::ZERO,
        vault_b_shares: U256::ZERO,
        vault_b_assets: U256::ZERO,
        module_state: ModuleState {
            configured: true,
            paused: false,
            policy_epoch: U256::from(1),
            policy_hash: B256::ZERO,
            mandate_hash: B256::ZERO,
            obligations_root: B256::ZERO,
            reserved_raw: U256::from(500_000_000),
            obligation_count: U256::ZERO,
            action_nonce: U256::ZERO,
            daily_day_index: U256::from(1),
            daily_used_raw: U256::ZERO,
            session_id: B256::ZERO,
            session_start_epoch: U256::from(1),
            session_key: Address::ZERO,
            session_expires_at: 0,
            session_limit_raw: U256::ZERO,
            session_used_raw: U256::ZERO,
        },
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

    let rates = vec![VaultRateInfo {
        vault: vault_a,
        rate_bps: 500,
        is_simulated: false,
    }];

    let set = enumerate_candidates(
        &snapshot,
        &[],
        &[],
        &BTreeMap::new(),
        &rates,
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
        B256::ZERO,
        B256::ZERO,
        true,
    );

    let prompt_res = build_planner_prompt(&set, Some("Pay upcoming expenses if needed"));

    // Ensure no 0x hex addresses/hashes appear in the prompt text
    assert!(!prompt_res.prompt.contains("0x"));
    // Ensure display strings like USDC are used
    assert!(prompt_res.prompt.contains("USDC"));

    // Check strict output validation
    let valid_choice = validate_planner_output(r#"{"choice":"A"}"#, &set);
    assert_eq!(valid_choice.choice_label, "A");
    assert!(!valid_choice.retryable);

    let hold_choice = validate_planner_output(r#"{"choice":"H"}"#, &set);
    assert_eq!(hold_choice.choice_label, "H");
    assert!(!hold_choice.retryable);
}
