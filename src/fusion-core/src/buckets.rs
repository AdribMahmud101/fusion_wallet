//! Reserve accounting and bucket calculations (§7.9).
//!
//! Tracks free, reserved, sweepable, shortfall, deployed, and total balances using exact
//! integer arithmetic.

use crate::types::{deserialize_u256, serialize_u256, Snapshot};
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

/// Treasury cash and reserve balance buckets (§7.9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buckets {
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub free: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub reserved: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub sweepable: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub shortfall: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub deployed: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub total: U256,
}

impl Buckets {
    /// Computes treasury buckets from atomic amounts (§7.9).
    pub fn compute(
        safe_cash: U256,
        reserved_raw: U256,
        operating_buffer_raw: U256,
        deployed_vault_assets: U256,
    ) -> Self {
        let c = safe_cash;
        let r = reserved_raw;
        let b = operating_buffer_raw;
        let v = deployed_vault_assets;

        let (free, reserved, sweepable, shortfall) = if c >= r {
            let free = c - r;
            let reserved = r;
            let rb = r.saturating_add(b);
            let sweepable = if c > rb { c - rb } else { U256::ZERO };
            let shortfall = U256::ZERO;
            (free, reserved, sweepable, shortfall)
        } else {
            let free = U256::ZERO;
            let reserved = c;
            let sweepable = U256::ZERO;
            let shortfall = r - c;
            (free, reserved, sweepable, shortfall)
        };

        let deployed = v;
        let total = c.saturating_add(v);

        Self {
            free,
            reserved,
            sweepable,
            shortfall,
            deployed,
            total,
        }
    }

    /// Computes treasury buckets directly from a Snapshot (§10.5, Appendix B).
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        let deployed_vault_assets = snapshot
            .vault_a_assets
            .saturating_add(snapshot.vault_b_assets);
        Self::compute(
            snapshot.safe_cash,
            snapshot.module_state.reserved_raw,
            snapshot.policy.operating_buffer_raw,
            deployed_vault_assets,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_treasury_with_surplus() {
        // C = 3000, R = 583, B = 500 -> sweepable = 3000 - 583 - 500 = 1917
        let b = Buckets::compute(
            U256::from(3000),
            U256::from(583),
            U256::from(500),
            U256::from(1000),
        );

        assert_eq!(b.free, U256::from(2417));
        assert_eq!(b.reserved, U256::from(583));
        assert_eq!(b.sweepable, U256::from(1917));
        assert_eq!(b.shortfall, U256::ZERO);
        assert_eq!(b.deployed, U256::from(1000));
        assert_eq!(b.total, U256::from(4000));
    }

    #[test]
    fn test_underfunded_treasury() {
        // C = 400, R = 583, B = 500 -> underfunded: free = 0, shortfall = 183
        let b = Buckets::compute(
            U256::from(400),
            U256::from(583),
            U256::from(500),
            U256::from(200),
        );

        assert_eq!(b.free, U256::ZERO);
        assert_eq!(b.reserved, U256::from(400));
        assert_eq!(b.sweepable, U256::ZERO);
        assert_eq!(b.shortfall, U256::from(183));
        assert_eq!(b.deployed, U256::from(200));
        assert_eq!(b.total, U256::from(600));
    }
}
