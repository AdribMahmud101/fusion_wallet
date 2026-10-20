use alloy::primitives::{Address, U256};
use fusion_core::quant::*;
fn u(v: u64) -> U256 {
    U256::from(v)
}
fn input() -> QuantInputs {
    serde_json::from_value(serde_json::json!({
    "blockNumber":50000,"vault":Address::repeat_byte(0xa1),"medianGasWei":"1000000000000",
    "quotedSizeRaw":"10000000","depositSlippageRaw":"1",
    "oracle":{"feed":ETH_USD_FEED,"decimals":8,"answer":"200000000000","roundId":"1","answeredInRound":"1","updatedAt":100000},
    "reserve":{"availableRaw":"150","variableDebtRaw":"850","stableDebtRaw":"0","liquidityRateRay":"2000000000000000000000000","liquidityIndex":"1000000000000000000000000000"}
})).unwrap()
}
#[test]
fn utilization_boundary_and_available_underlying() {
    let mut q = input();
    let report = QuantReport::from_inputs(50000, 100000, Some(&q));
    assert_eq!(report.utilization_bps, Some(8500));
    assert!(report.deposit_allowed);
    assert_eq!(report.withdrawable_now_raw.as_deref(), Some("150"));
    q.reserve.as_mut().unwrap().stable_debt_raw = "1".into();
    assert!(!QuantReport::from_inputs(50000, 100000, Some(&q)).deposit_allowed);
    q.reserve.as_mut().unwrap().available_raw = "0".into();
    let report = QuantReport::from_inputs(50000, 100000, Some(&q));
    assert_eq!(report.utilization_bps, Some(10000));
    assert!(!report.deposit_allowed);
    q.reserve.as_mut().unwrap().variable_debt_raw = "0".into();
    q.reserve.as_mut().unwrap().stable_debt_raw = "0".into();
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q)).utilization_bps,
        None
    );
    assert_eq!(
        QuantReport::from_inputs(49999, 100000, Some(&q)).withdrawable_now_raw,
        None
    );
}
#[test]
fn gas_usd_is_ceiled_and_oracle_fails_closed() {
    let mut q = input();
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q))
            .gas_usd_raw
            .as_deref(),
        Some("2000")
    );
    q.median_gas_wei = Some("1000000000001".into());
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q))
            .gas_usd_raw
            .as_deref(),
        Some("2001")
    );
    for now in [101201, 99999] {
        assert_eq!(
            QuantReport::from_inputs(50000, now, Some(&q)).gas_usd_raw,
            None
        );
    }
    for answer in ["0", "-1", "garbage"] {
        q.oracle.as_mut().unwrap().answer = answer.into();
        assert_eq!(
            QuantReport::from_inputs(50000, 100000, Some(&q)).gas_usd_raw,
            None
        );
    }
    q = input();
    q.oracle.as_mut().unwrap().answered_in_round = "0".into();
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q)).gas_usd_raw,
        None
    );
    q = input();
    q.oracle.as_mut().unwrap().decimals = 18;
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q)).gas_usd_raw,
        None
    );
    q = input();
    q.oracle.as_mut().unwrap().feed = Address::ZERO.to_string();
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q)).gas_usd_raw,
        None
    );
}
#[test]
fn break_even_exact_fourteen_days_and_nonpositive_delta() {
    // 365 USDC * 1% / 365 = 0.01 USDC/day; 0.10 gas + 0.04 slip = 14 days.
    let e = break_even(u(365_000_000), 10_000_000, 0, Some(u(100_000)), u(40_000));
    assert_eq!(e.status, "ACTIONABLE");
    assert_eq!(e.break_even_days_hundredths.as_deref(), Some("1400"));
    assert_eq!(e.actionable_size_raw.as_deref(), Some("365000000"));
    assert_eq!(
        break_even(u(365_000_000), 10_000_000, 0, Some(u(100_001)), u(40_000)).status,
        "UNPROFITABLE_AFTER_GAS"
    );
    for (new, current) in [(0, 0), (1, 1), (0, 1)] {
        let e = break_even(u(10_000_000), new, current, Some(u(100)), u(0));
        assert_eq!(e.status, "UNPROFITABLE_AFTER_GAS");
        assert!(e.break_even_days_hundredths.is_none());
    }
    assert_eq!(
        break_even(u(0), 1, 0, Some(u(1)), u(0)).status,
        "UNPROFITABLE_AFTER_GAS"
    );
    assert_eq!(break_even(u(1), 1, 0, None, u(0)).status, "UNMEASURED");
    assert_eq!(
        break_even(U256::MAX, 1, 0, Some(U256::MAX), u(1)).status,
        "UNMEASURED"
    );
}
#[test]
fn realized_apy_caps_spot_and_spot_only_is_explicit() {
    let mut q = input();
    let report = QuantReport::from_inputs(50000, 100000, Some(&q));
    assert_eq!(report.apy_label, "spot only");
    assert!(report.spot_apy_ppb.unwrap() > 2_000_000);
    let r = q.reserve.as_mut().unwrap();
    r.past_index = Some(r.liquidity_index.clone());
    r.past_timestamp = Some(13600);
    r.past_block = Some(6800);
    let report = QuantReport::from_inputs(50000, 100000, Some(&q));
    assert_eq!(report.realized_apy_ppb, Some(0));
    assert_eq!(report.conservative_apy_ppb, Some(0));
    assert_eq!(report.apy_label, "min(spot, realized ~24h)");
    assert_eq!(annualized_ratio(u(2), u(1), 0), None);
    assert_eq!(annualized_ratio(u(1), u(2), 86400), None);
    assert_eq!(
        annualized_ratio(u(10001), u(10000), 86400),
        Some(37_172_411)
    );
    assert_eq!(spot_apy(U256::ZERO), Some(0));
    q.reserve.as_mut().unwrap().past_index = Some("2000000000000000000000000000".into());
    assert_eq!(
        QuantReport::from_inputs(50000, 100000, Some(&q)).conservative_apy_ppb,
        None,
        "An invalid supplied historical observation must not silently fall back to spot"
    );
}
#[test]
fn missing_or_wrong_quote_cannot_authorize() {
    let mut q = input();
    let report = QuantReport::from_inputs(50000, 100000, Some(&q));
    let e = report.evaluate(q.vault, u(10_000_000), 0, Some(&q));
    assert_eq!(e.status, "UNPROFITABLE_AFTER_GAS");
    assert_eq!(
        report.evaluate(q.vault, u(11_000_000), 0, Some(&q)).status,
        "UNMEASURED"
    );
    let other = report.evaluate(Address::repeat_byte(0xb2), u(10_000_000), 0, Some(&q));
    assert_eq!(other.status, "UNMEASURED");
    assert!(other.break_even_days_hundredths.is_none());
    assert!(other.slippage_usd_raw.is_none());
    assert!(other.actionable_size_raw.is_none());
    q.deposit_slippage_raw = None;
    assert_eq!(
        report.evaluate(q.vault, u(10_000_000), 0, Some(&q)).status,
        "UNMEASURED"
    );
}
