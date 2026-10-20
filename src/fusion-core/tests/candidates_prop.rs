//! Candidate enumeration determinism and property tests (§10.4).

use alloy::primitives::{Address, B256, U256};
use fusion_core::candidates::{enumerate_candidates, VaultRateInfo};
use fusion_core::types::*;
use std::collections::BTreeMap;

fn make_sample_snapshot() -> Snapshot {
    let vault_a = Address::from_slice(&[0xA1; 20]);
    let vault_b = Address::from_slice(&[0xB2; 20]);

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
        vaults: vec![
            VaultLimit {
                vault: vault_a,
                max_exposure_raw: U256::from(6_000_000_000_u64),
            },
            VaultLimit {
                vault: vault_b,
                max_exposure_raw: U256::from(6_000_000_000_u64),
            },
        ],
        payee_delay_seconds: 3600,
        pay_early_window_seconds: 259_200, // 3 days
    };

    let module_state = ModuleState {
        configured: true,
        paused: false,
        policy_epoch: U256::from(1),
        policy_hash: B256::ZERO,
        mandate_hash: B256::ZERO,
        obligations_root: B256::ZERO,
        reserved_raw: U256::from(583_000_000), // 583 USDC
        obligation_count: U256::from(2),
        action_nonce: U256::ZERO,
        daily_day_index: U256::from(1),
        daily_used_raw: U256::ZERO,
        session_id: B256::ZERO,
        session_start_epoch: U256::from(1),
        session_key: Address::ZERO,
        session_expires_at: 0,
        session_limit_raw: U256::ZERO,
        session_used_raw: U256::ZERO,
    };

    Snapshot {
        quant: None,
        block_number: 100,
        block_hash: B256::repeat_byte(0x33),
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
    }
}

#[test]
fn test_enumeration_determinism() {
    let snapshot = make_sample_snapshot();
    let rates = vec![
        VaultRateInfo {
            vault: Address::from_slice(&[0xA1; 20]),
            rate_bps: 500, // 5% APY
            is_simulated: false,
        },
        VaultRateInfo {
            vault: Address::from_slice(&[0xB2; 20]),
            rate_bps: 450, // 4.5% APY
            is_simulated: true,
        },
    ];

    let chain_id = U256::from(84532);
    let safe = Address::from_slice(&[0x11; 20]);
    let module = Address::from_slice(&[0x22; 20]);
    let mandate_hash = B256::repeat_byte(0x44);
    let session_id = B256::repeat_byte(0x55);

    let res1 = enumerate_candidates(
        &snapshot,
        &[],
        &[],
        &BTreeMap::new(),
        &rates,
        chain_id,
        safe,
        module,
        mandate_hash,
        session_id,
        true,
    );

    let res2 = enumerate_candidates(
        &snapshot,
        &[],
        &[],
        &BTreeMap::new(),
        &rates,
        chain_id,
        safe,
        module,
        mandate_hash,
        session_id,
        true,
    );

    assert_eq!(
        res1, res2,
        "Candidate enumeration must be 100% deterministic"
    );
    assert_eq!(res1.hold.label, "H");
}

#[test]
fn test_bill_ordering_earliest_due_at() {
    let mut snapshot = make_sample_snapshot();
    snapshot.safe_cash = U256::from(500_000_000); // 500 USDC

    let payee_account = Address::from_slice(&[0x99; 20]);
    let payee_id = B256::repeat_byte(0x77);

    let payees = vec![Payee {
        account: payee_account,
        label_hash: payee_id,
        active_at: 1_699_000_000,
        cancelled: false,
    }];

    let bills = vec![
        Obligation {
            obligation_id: "bill_later".to_string(),
            obligation_id_hash: B256::repeat_byte(0x02),
            payee_id_hash: payee_id,
            amount_raw: U256::from(20_000_000),
            due_at: snapshot.block_timestamp + 86_400, // Due in 1 day
        },
        Obligation {
            obligation_id: "bill_sooner".to_string(),
            obligation_id_hash: B256::repeat_byte(0x01),
            payee_id_hash: payee_id,
            amount_raw: U256::from(10_000_000),
            due_at: snapshot.block_timestamp + 3_600, // Due in 1 hour
        },
    ];

    let res = enumerate_candidates(
        &snapshot,
        &bills,
        &payees,
        &BTreeMap::new(),
        &[],
        U256::from(84532),
        Address::ZERO,
        Address::ZERO,
        B256::ZERO,
        B256::ZERO,
        true,
    );

    assert!(res.candidates.len() >= 2);
    assert_eq!(res.candidates[0].label, "A");
    assert_eq!(res.candidates[0].action.assets, U256::from(10_000_000));
    assert_eq!(res.candidates[1].label, "B");
    assert_eq!(res.candidates[1].action.assets, U256::from(20_000_000));
}

#[test]
fn measured_economics_filter_precedes_planner_and_reports_rejections() {
    let mut snapshot = make_sample_snapshot();
    let vault = snapshot.policy.vaults[0].vault;
    let enumerate = |s: &Snapshot| {
        enumerate_candidates(
            s,
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
        )
    };
    assert!(
        enumerate(&snapshot).candidates.is_empty(),
        "Missing measurements must not offer a deposit"
    );
    snapshot.quant = Some(serde_json::from_value(serde_json::json!({"blockNumber":100,"vault":vault,
        "medianGasWei":"1000000000000","quotedSizeRaw":"10000000","depositSlippageRaw":"1",
        "oracle":{"feed":fusion_core::quant::ETH_USD_FEED,"decimals":8,"answer":"200000000000","roundId":"1","answeredInRound":"1","updatedAt":1700000000},
        "reserve":{"availableRaw":"1000000000","variableDebtRaw":"100000000","stableDebtRaw":"0","liquidityRateRay":"2000000000000000000000000","liquidityIndex":"1000000000000000000000000000"}})).unwrap());
    let set = enumerate(&snapshot);
    assert!(set.candidates.is_empty());
    assert_eq!(set.quant.candidates[0].status, "UNPROFITABLE_AFTER_GAS");
    snapshot.quant.as_mut().unwrap().median_gas_wei = Some("1".into());
    let set = enumerate(&snapshot);
    assert_eq!(set.candidates.len(), 1);
    assert_eq!(
        set.candidates[0].action.assets,
        snapshot.policy.micro_max_raw
    );
    assert_eq!(set.quant.candidates[0].status, "ACTIONABLE");
    snapshot
        .quant
        .as_mut()
        .unwrap()
        .reserve
        .as_mut()
        .unwrap()
        .variable_debt_raw = "9000000000".into();
    let set = enumerate(&snapshot);
    assert!(set.candidates.is_empty());
    assert_eq!(
        set.quant.candidates[0].status,
        "UTILIZATION_ABOVE_85_PERCENT"
    );
}

#[test]
fn safety_withdraw_is_capped_at_available_underlying_not_total_deposits() {
    let mut snapshot = make_sample_snapshot();
    snapshot.safe_cash = U256::ZERO;
    snapshot.vault_a_assets = U256::from(1_000_000_000);
    let vault = snapshot.policy.vaults[0].vault;
    snapshot.quant=Some(serde_json::from_value(serde_json::json!({"blockNumber":100,"vault":vault,
        "reserve":{"availableRaw":"17","variableDebtRaw":"9000000000","stableDebtRaw":"0","liquidityRateRay":"0","liquidityIndex":"1000000000000000000000000000"}})).unwrap());
    let rates = [VaultRateInfo {
        vault,
        rate_bps: 0,
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
    assert_eq!(set.candidates.len(), 1);
    assert_eq!(set.candidates[0].kind, ActionKind::Withdraw);
    assert_eq!(set.candidates[0].action.assets, U256::from(17));
}

#[test]
fn surplus_deposit_respects_micro_limit_and_still_requires_profit() {
    let mut snapshot = make_sample_snapshot();
    snapshot.safe_cash = U256::from(9_399_001_235_u64);
    snapshot.module_state.reserved_raw = U256::ZERO;
    let vault = snapshot.policy.vaults[0].vault;
    let rates = [VaultRateInfo { vault, rate_bps: 500, is_simulated: false }];
    let enumerate = |snapshot: &Snapshot| {
        enumerate_candidates(
            snapshot, &[], &[], &BTreeMap::new(), &rates, U256::from(84532),
            Address::repeat_byte(0x11), Address::repeat_byte(0x22),
            snapshot.module_state.mandate_hash, snapshot.module_state.session_id, true,
        )
    };
    // K44's measured economics gate wins: an unmeasured rate never authorizes a deposit.
    assert!(enumerate(&snapshot).candidates.is_empty());
    snapshot.quant = Some(serde_json::from_value(serde_json::json!({
        "blockNumber":100,"vault":vault,"medianGasWei":"1","quotedSizeRaw":"10000000","depositSlippageRaw":"1",
        "oracle":{"feed":fusion_core::quant::ETH_USD_FEED,"decimals":8,"answer":"200000000000","roundId":"1","answeredInRound":"1","updatedAt":1700000000},
        "reserve":{"availableRaw":"1000000000","variableDebtRaw":"100000000","stableDebtRaw":"0","liquidityRateRay":"2000000000000000000000000","liquidityIndex":"1000000000000000000000000000"}
    })).unwrap());
    let set = enumerate(&snapshot);
    let deposit = set.candidates.iter().find(|c| c.kind == ActionKind::Deposit).unwrap();
    assert_eq!(deposit.action.assets, snapshot.policy.micro_max_raw);
    assert_eq!(deposit.route, fusion_core::candidates::Route::Instant);
    assert_eq!(set.quant.candidates[0].status, "ACTIONABLE");
}
