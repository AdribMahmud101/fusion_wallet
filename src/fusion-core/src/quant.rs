//! K44: measured economics, in integer USD micro-units and APY parts per billion.
//! Missing measurements never authorize a yield move. No policy is modified.
use crate::types::Snapshot;
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

pub const ETH_USD_FEED: &str = "0x4adc67696ba383f43dd60a9e78f2c97fbbfc7cb1";
pub const ORACLE_MAX_AGE: u64 = 1_200; // published Base Sepolia ETH/USD heartbeat
const YEAR: u64 = 31_536_000;
const PPB: u64 = 1_000_000_000;
fn raw(s: &str) -> Option<U256> {
    s.parse().ok()
}
fn ceil(n: U256, d: U256) -> Option<U256> {
    if d.is_zero() {
        return None;
    }
    Some(n / d + U256::from(n % d != U256::ZERO))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleRound {
    pub feed: String,
    pub decimals: u8,
    pub answer: String,
    pub round_id: String,
    pub answered_in_round: String,
    pub updated_at: u64,
}
impl OracleRound {
    pub fn price(&self, now: u64) -> Option<U256> {
        let answer = raw(&self.answer)?;
        let round = raw(&self.round_id)?;
        (self.feed.eq_ignore_ascii_case(ETH_USD_FEED)
            && self.decimals == 8
            && !answer.is_zero()
            && !round.is_zero()
            && raw(&self.answered_in_round)? >= round
            && self.updated_at > 0
            && self.updated_at <= now
            && now - self.updated_at <= ORACLE_MAX_AGE)
            .then_some(answer)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveMeasurement {
    pub available_raw: String,
    pub variable_debt_raw: String,
    pub stable_debt_raw: String,
    pub liquidity_rate_ray: String,
    pub liquidity_index: String,
    #[serde(default)]
    pub past_index: Option<String>,
    #[serde(default)]
    pub past_timestamp: Option<u64>,
    #[serde(default)]
    pub past_block: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantInputs {
    pub block_number: u64,
    pub vault: Address,
    #[serde(default)]
    pub reserve: Option<ReserveMeasurement>,
    #[serde(default)]
    pub oracle: Option<OracleRound>,
    /// K33 median actual receipt fee, rounded UP to a wei. Includes L1/operator fees.
    #[serde(default)]
    pub median_gas_wei: Option<String>,
    #[serde(default)]
    pub quoted_size_raw: Option<String>,
    #[serde(default)]
    pub deposit_slippage_raw: Option<String>,
}

// Floor every fixed-point multiplication: APY is a conservative lower bound.
fn mul(a: U256, b: U256, scale: U256) -> Option<U256> {
    Some(a.checked_mul(b)? / scale)
}
fn pow(mut base: U256, mut exponent: u64, scale: U256) -> Option<U256> {
    let mut result = scale;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = mul(result, base, scale)?;
        }
        exponent >>= 1;
        if exponent > 0 {
            base = mul(base, base, scale)?;
        }
    }
    Some(result)
}
fn sqrt(n: U256) -> U256 {
    if n.is_zero() {
        return n;
    }
    let mut x = (n >> 1) + U256::from(1);
    let mut y = n;
    while x < y {
        y = x;
        x = (x + n / x) >> 1;
    }
    y
}
/// Annual compound return from a ratio over elapsed seconds. Fractional exponent
/// uses 32 binary square roots; rounding can only lower the reported rate.
pub fn annualized_ratio(now: U256, past: U256, elapsed: u64) -> Option<u64> {
    if past.is_zero() || now < past || !(23 * 3600..=25 * 3600).contains(&elapsed) {
        return None;
    }
    let scale = U256::from(10u64.pow(18));
    let ratio = now.checked_mul(scale)? / past;
    let mut result = pow(ratio, YEAR / elapsed, scale)?;
    let mut root = ratio;
    let mut remainder = YEAR % elapsed;
    for _ in 0..32 {
        root = sqrt(root.checked_mul(scale)?);
        remainder *= 2;
        if remainder >= elapsed {
            result = mul(result, root, scale)?;
            remainder -= elapsed;
        }
    }
    ((result.checked_sub(scale)?).checked_mul(U256::from(PPB))? / scale)
        .try_into()
        .ok()
}
pub fn spot_apy(ray: U256) -> Option<u64> {
    let scale = U256::from(10u64.pow(18));
    let per_second = ray / U256::from(PPB) / U256::from(YEAR);
    let value = pow(scale.checked_add(per_second)?, YEAR, scale)?;
    (value.checked_sub(scale)?.checked_mul(U256::from(PPB))? / scale)
        .try_into()
        .ok()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Economics {
    pub vault: Address,
    pub size_raw: String,
    pub status: String,
    pub gas_usd_raw: Option<String>,
    pub slippage_usd_raw: Option<String>,
    /// Ceiled hundredths of a day; classification uses the exact fraction.
    pub break_even_days_hundredths: Option<String>,
    pub actionable_size_raw: Option<String>,
}
/// days = (gas + slippage) * 365 / (size * delta APY).
/// Slippage is the measured previewDeposit -> convertToAssets round-trip loss.
pub fn break_even(
    size: U256,
    new_ppb: u64,
    current_ppb: u64,
    gas: Option<U256>,
    slip: U256,
) -> Economics {
    let mut out = Economics {
        size_raw: size.to_string(),
        status: "UNMEASURED".into(),
        slippage_usd_raw: Some(slip.to_string()),
        gas_usd_raw: gas.map(|v| v.to_string()),
        ..Default::default()
    };
    let Some(gas) = gas else {
        return out;
    };
    out.status = "UNPROFITABLE_AFTER_GAS".into();
    if size.is_zero() || new_ppb <= current_ppb {
        return out;
    }
    let delta = U256::from(new_ppb - current_ppb);
    let Some(numerator) = gas
        .checked_add(slip)
        .and_then(|v| v.checked_mul(U256::from(365 * PPB)))
    else {
        out.status = "UNMEASURED".into();
        return out;
    };
    let Some(denominator) = size.checked_mul(delta) else {
        out.status = "UNMEASURED".into();
        return out;
    };
    out.break_even_days_hundredths = numerator
        .checked_mul(U256::from(100))
        .and_then(|n| ceil(n, denominator))
        .map(|v| v.to_string());
    if denominator
        .checked_mul(U256::from(14))
        .is_some_and(|d| numerator <= d)
    {
        out.status = "ACTIONABLE".into();
    }
    // Conditional threshold at these measured gas/slippage costs; re-quote a
    // changed size before any future action. This never changes owner policy.
    out.actionable_size_raw = gas
        .checked_add(slip)
        .and_then(|v| v.checked_mul(U256::from(365 * PPB)))
        .and_then(|n| ceil(n, delta * U256::from(14)))
        .map(|v| v.to_string());
    out
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantReport {
    pub block_number: u64,
    pub utilization_bps: Option<u64>,
    pub deposit_allowed: bool,
    pub withdrawable_now_raw: Option<String>,
    pub spot_apy_ppb: Option<u64>,
    pub realized_apy_ppb: Option<u64>,
    pub conservative_apy_ppb: Option<u64>,
    pub apy_label: String,
    pub gas_usd_raw: Option<String>,
    pub gas_status: String,
    pub candidates: Vec<Economics>,
}
impl QuantReport {
    pub fn read(snapshot: &Snapshot) -> Self {
        Self::from_inputs(
            snapshot.block_number,
            snapshot.block_timestamp,
            snapshot.quant.as_ref(),
        )
    }
    pub fn from_inputs(block: u64, timestamp: u64, inputs: Option<&QuantInputs>) -> Self {
        let mut out = Self {
            block_number: block,
            apy_label: "UNMEASURED".into(),
            gas_status: "UNMEASURED".into(),
            ..Default::default()
        };
        let Some(q) = inputs else {
            return out;
        };
        if q.block_number != block {
            return out;
        }
        if let Some(r) = &q.reserve {
            if let (Some(available), Some(variable), Some(stable)) = (
                raw(&r.available_raw),
                raw(&r.variable_debt_raw),
                raw(&r.stable_debt_raw),
            ) {
                out.withdrawable_now_raw = Some(available.to_string());
                if let Some(debt) = variable.checked_add(stable) {
                    if let Some(total) = available.checked_add(debt).filter(|v| !v.is_zero()) {
                        out.utilization_bps = debt
                            .checked_mul(U256::from(10_000))
                            .and_then(|n| ceil(n, total))
                            .and_then(|v| v.try_into().ok());
                        out.deposit_allowed = debt
                            .checked_mul(U256::from(100))
                            .zip(total.checked_mul(U256::from(85)))
                            .is_some_and(|(d, t)| d <= t);
                    }
                }
            }
            out.spot_apy_ppb = raw(&r.liquidity_rate_ray).and_then(spot_apy);
            out.realized_apy_ppb = r
                .past_timestamp
                .zip(r.past_block)
                .filter(|(t, b)| *t < timestamp && *b < block)
                .and_then(|(t, _)| {
                    annualized_ratio(
                        raw(&r.liquidity_index)?,
                        raw(r.past_index.as_ref()?)?,
                        timestamp - t,
                    )
                });
            out.conservative_apy_ppb = out
                .spot_apy_ppb
                .map(|spot| out.realized_apy_ppb.map_or(spot, |real| real.min(spot)));
            if (r.past_index.is_some() || r.past_timestamp.is_some() || r.past_block.is_some())
                && out.realized_apy_ppb.is_none()
            {
                out.conservative_apy_ppb = None;
            }
            if out.conservative_apy_ppb.is_some() {
                out.apy_label = if out.realized_apy_ppb.is_some() {
                    "min(spot, realized ~24h)"
                } else {
                    "spot only"
                }
                .into();
            }
        }
        let gas = q
            .median_gas_wei
            .as_deref()
            .and_then(raw)
            .filter(|v| !v.is_zero());
        let price = q.oracle.as_ref().and_then(|o| o.price(timestamp));
        out.gas_usd_raw = gas
            .zip(price)
            .and_then(|(g, p)| {
                ceil(
                    g.checked_mul(p)?,
                    U256::from(10u64.pow(18)) * U256::from(100),
                )
            })
            .map(|v| v.to_string());
        if out.gas_usd_raw.is_some() {
            out.gas_status = "K33 median actual fee + Chainlink ETH/USD".into();
        }
        out
    }
    pub fn evaluate(
        &self,
        vault: Address,
        size: U256,
        current_ppb: u64,
        inputs: Option<&QuantInputs>,
    ) -> Economics {
        let quote = inputs
            .filter(|q| q.quoted_size_raw.as_deref().and_then(raw) == Some(size))
            .and_then(|q| q.deposit_slippage_raw.as_deref().and_then(raw))
            .filter(|v| *v <= size);
        let mut economics = break_even(
            size,
            self.conservative_apy_ppb.unwrap_or(0),
            current_ppb,
            self.gas_usd_raw.as_deref().and_then(raw),
            quote.unwrap_or_default(),
        );
        economics.vault = vault;
        if quote.is_none() {
            economics.slippage_usd_raw = None;
            economics.break_even_days_hundredths = None;
            economics.actionable_size_raw = None;
        }
        if inputs.map(|q| q.vault) != Some(vault)
            || quote.is_none()
            || self.conservative_apy_ppb.is_none()
            || self.utilization_bps.is_none()
        {
            economics.status = "UNMEASURED".into();
            // A quote/rate belongs only to its measured vault and block.
            economics.slippage_usd_raw = None;
            economics.break_even_days_hundredths = None;
            economics.actionable_size_raw = None;
        } else if !self.deposit_allowed {
            economics.status = "UTILIZATION_ABOVE_85_PERCENT".into();
        }
        economics
    }
}
