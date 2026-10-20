//! Data Transfer Objects (DTOs) and protocol types for Kavachi V4.1.
//!
//! Strictly enforces zero floating-point arithmetic across all types.

use alloy::primitives::{Address, B256, U256};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Permission tiers supported by the Kavachi module (§7.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PermissionTier {
    ReadOnly = 0,
    OwnerApproved = 1,
    Autonomous = 2,
}

impl PermissionTier {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ReadOnly => "READ_ONLY",
            Self::OwnerApproved => "OWNER_APPROVED",
            Self::Autonomous => "AUTONOMOUS",
        }
    }
}

impl TryFrom<u8> for PermissionTier {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::ReadOnly),
            1 => Ok(Self::OwnerApproved),
            2 => Ok(Self::Autonomous),
            other => Err(format!("Unknown permission tier: {other}")),
        }
    }
}

impl Serialize for PermissionTier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for PermissionTier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TierVisitor;

        impl<'de> de::Visitor<'de> for TierVisitor {
            type Value = PermissionTier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("integer 0-2 or tier string name")
            }

            fn visit_u64<E>(self, val: u64) -> Result<PermissionTier, E>
            where
                E: de::Error,
            {
                if val <= 2 {
                    PermissionTier::try_from(val as u8).map_err(E::custom)
                } else {
                    Err(E::custom(format!("Invalid tier number: {val}")))
                }
            }

            fn visit_str<E>(self, s: &str) -> Result<PermissionTier, E>
            where
                E: de::Error,
            {
                match s.trim() {
                    "0" | "READ_ONLY" => Ok(PermissionTier::ReadOnly),
                    "1" | "OWNER_APPROVED" => Ok(PermissionTier::OwnerApproved),
                    "2" | "AUTONOMOUS" => Ok(PermissionTier::Autonomous),
                    other => Err(E::custom(format!("Unknown permission tier: {other}"))),
                }
            }
        }

        deserializer.deserialize_any(TierVisitor)
    }
}

/// Action kinds executed or queued by the Kavachi module (§7.6).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ActionKind {
    Deposit = 0,
    Withdraw = 1,
    Rebalance = 2,
    PayObligation = 3,
}

impl ActionKind {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Deposit => "DEPOSIT",
            Self::Withdraw => "WITHDRAW",
            Self::Rebalance => "REBALANCE",
            Self::PayObligation => "PAY_OBLIGATION",
        }
    }
}

impl TryFrom<u8> for ActionKind {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Withdraw),
            2 => Ok(Self::Rebalance),
            3 => Ok(Self::PayObligation),
            other => Err(format!("Unknown action kind: {other}")),
        }
    }
}

impl Serialize for ActionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for ActionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionKindVisitor;

        impl<'de> de::Visitor<'de> for ActionKindVisitor {
            type Value = ActionKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("action kind 0-3 or name string")
            }

            fn visit_u64<E>(self, val: u64) -> Result<ActionKind, E>
            where
                E: de::Error,
            {
                if val <= 3 {
                    ActionKind::try_from(val as u8).map_err(E::custom)
                } else {
                    Err(E::custom(format!("Invalid action kind: {val}")))
                }
            }

            fn visit_str<E>(self, s: &str) -> Result<ActionKind, E>
            where
                E: de::Error,
            {
                match s.trim() {
                    "0" | "DEPOSIT" => Ok(ActionKind::Deposit),
                    "1" | "WITHDRAW" => Ok(ActionKind::Withdraw),
                    "2" | "REBALANCE" => Ok(ActionKind::Rebalance),
                    "3" | "PAY_OBLIGATION" => Ok(ActionKind::PayObligation),
                    other => Err(E::custom(format!("Unknown action kind: {other}"))),
                }
            }
        }

        deserializer.deserialize_any(ActionKindVisitor)
    }
}

/// Helper deserializer for U256 from string or number.
pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    struct U256Visitor;

    impl<'de> de::Visitor<'de> for U256Visitor {
        type Value = U256;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a U256 as decimal/hex string or integer")
        }

        fn visit_u64<E>(self, val: u64) -> Result<U256, E>
        where
            E: de::Error,
        {
            Ok(U256::from(val))
        }

        fn visit_u128<E>(self, val: u128) -> Result<U256, E>
        where
            E: de::Error,
        {
            Ok(U256::from(val))
        }

        fn visit_str<E>(self, s: &str) -> Result<U256, E>
        where
            E: de::Error,
        {
            let s = s.trim();
            if let Some(stripped) = s.strip_prefix("0x") {
                U256::from_str_radix(stripped, 16).map_err(E::custom)
            } else {
                U256::from_str_radix(s, 10).map_err(E::custom)
            }
        }
    }

    deserializer.deserialize_any(U256Visitor)
}

/// Helper serializer for U256 as base-10 string per §9.
pub fn serialize_u256<S>(val: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&val.to_string())
}

/// Helper deserializer for u64 from string or number.
pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64Visitor;

    impl<'de> de::Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 as decimal string or integer")
        }

        fn visit_u64<E>(self, val: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            Ok(val)
        }

        fn visit_str<E>(self, s: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            s.trim().parse::<u64>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(U64Visitor)
}

/// Helper deserializer for u32 from string or number.
pub fn deserialize_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct U32Visitor;

    impl<'de> de::Visitor<'de> for U32Visitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u32 as decimal string or integer")
        }

        fn visit_u64<E>(self, val: u64) -> Result<u32, E>
        where
            E: de::Error,
        {
            u32::try_from(val).map_err(E::custom)
        }

        fn visit_str<E>(self, s: &str) -> Result<u32, E>
        where
            E: de::Error,
        {
            s.trim().parse::<u32>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(U32Visitor)
}

/// Helper deserializer for u16 from string or number.
pub fn deserialize_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    struct U16Visitor;

    impl<'de> de::Visitor<'de> for U16Visitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u16 as decimal string or integer")
        }

        fn visit_u64<E>(self, val: u64) -> Result<u16, E>
        where
            E: de::Error,
        {
            u16::try_from(val).map_err(E::custom)
        }

        fn visit_str<E>(self, s: &str) -> Result<u16, E>
        where
            E: de::Error,
        {
            s.trim().parse::<u16>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(U16Visitor)
}

/// Exposure limit per vault (§7.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultLimit {
    pub vault: Address,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub max_exposure_raw: U256,
}

/// Policy configured on the Kavachi module (§7.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub permission_tier: PermissionTier,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub operating_buffer_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub micro_max_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub delayed_max_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub hard_max_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub daily_limit_raw: U256,
    #[serde(deserialize_with = "deserialize_u32")]
    pub delay_seconds: u32,
    #[serde(deserialize_with = "deserialize_u16")]
    pub max_share_slippage_bps: u16,
    pub guardian: Address,
    pub vaults: Vec<VaultLimit>,
    #[serde(deserialize_with = "deserialize_u32")]
    pub payee_delay_seconds: u32,
    #[serde(deserialize_with = "deserialize_u32")]
    pub pay_early_window_seconds: u32,
}

/// Agent or owner action (§7.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub kind: ActionKind,
    pub source_vault: Address,
    pub destination_vault: Address,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub assets: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub min_shares_out: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub max_shares_burn: U256,
    pub obligation_id_hash: B256,
    pub payee_id_hash: B256,
    pub recipient: Address,
    #[serde(deserialize_with = "deserialize_u64")]
    pub due_at: u64,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub expected_nonce: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub policy_epoch: U256,
    #[serde(deserialize_with = "deserialize_u64")]
    pub deadline: u64,
    pub decision_hash: B256,
}

/// Registered payee in the Kavachi module (§8.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payee {
    pub account: Address,
    pub label_hash: B256,
    #[serde(deserialize_with = "deserialize_u64")]
    pub active_at: u64,
    pub cancelled: bool,
}

/// Queued action record on-chain (§7.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub exists: bool,
    pub action_digest: B256,
    pub session_id: B256,
    #[serde(deserialize_with = "deserialize_u64")]
    pub ready_at: u64,
    #[serde(deserialize_with = "deserialize_u64")]
    pub expires_at: u64,
    pub action: Action,
}

/// Full state read from the Kavachi module (§7.7, §10.5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleState {
    pub configured: bool,
    pub paused: bool,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub policy_epoch: U256,
    pub policy_hash: B256,
    pub mandate_hash: B256,
    pub obligations_root: B256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub reserved_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub obligation_count: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub action_nonce: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub daily_day_index: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub daily_used_raw: U256,
    pub session_id: B256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub session_start_epoch: U256,
    pub session_key: Address,
    #[serde(deserialize_with = "deserialize_u64")]
    pub session_expires_at: u64,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub session_limit_raw: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub session_used_raw: U256,
}

/// Single obligation (bill) record (§8.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Obligation {
    pub obligation_id: String,
    pub obligation_id_hash: B256,
    pub payee_id_hash: B256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub amount_raw: U256,
    #[serde(deserialize_with = "deserialize_u64")]
    pub due_at: u64,
}

/// Snapshot of on-chain state read at a single block (§10.5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quant: Option<crate::quant::QuantInputs>,
    #[serde(deserialize_with = "deserialize_u64")]
    pub block_number: u64,
    pub block_hash: B256,
    #[serde(deserialize_with = "deserialize_u64")]
    pub block_timestamp: u64,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub safe_cash: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub vault_a_shares: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub vault_a_assets: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub vault_b_shares: U256,
    #[serde(
        deserialize_with = "deserialize_u256",
        serialize_with = "serialize_u256"
    )]
    pub vault_b_assets: U256,
    pub module_state: ModuleState,
    pub policy: Policy,
    pub queue: Queue,
}
