//! Money and accounting integer primitives for Kavachi V4.1.
//!
//! Enforces zero floating-point arithmetic across the engine.
//! All token amounts are represented in atomic raw units (e.g. USDC with 6 decimal places).

use alloy::primitives::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, Sub};

/// USDC standard decimal places on EVM chains.
pub const USDC_DECIMALS: u8 = 6;
const USDC_SCALE: u64 = 1_000_000;

/// Errors arising from amount parsing or arithmetic.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AmountError {
    #[error("Invalid amount string format")]
    InvalidFormat,
    #[error("Exceeded maximum decimal places (maximum {USDC_DECIMALS})")]
    TooManyDecimals,
    #[error("Amount overflowed U256")]
    Overflow,
    #[error("Amount string is empty")]
    Empty,
    #[error("Negative amounts are not permitted")]
    Negative,
}

/// Exact integer token amount in atomic raw units (6 decimal places).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawAmount(pub U256);

impl RawAmount {
    /// Zero amount.
    pub const ZERO: Self = Self(U256::ZERO);

    /// Construct from raw `U256`.
    pub const fn from_raw(raw: U256) -> Self {
        Self(raw)
    }

    /// Construct from `u64` raw units.
    pub const fn from_u64(raw: u64) -> Self {
        Self(U256::from_limbs([raw, 0, 0, 0]))
    }

    /// Construct from `u128` raw units.
    pub fn from_u128(raw: u128) -> Self {
        Self(U256::from(raw))
    }

    /// Returns the raw `U256` value.
    pub const fn raw(&self) -> U256 {
        self.0
    }

    /// Converts raw amount to `u128` if it fits.
    pub fn as_u128(&self) -> Result<u128, AmountError> {
        self.0.try_into().map_err(|_| AmountError::Overflow)
    }

    /// Converts raw amount to `u64` if it fits.
    pub fn as_u64(&self) -> Result<u64, AmountError> {
        self.0.try_into().map_err(|_| AmountError::Overflow)
    }

    /// Checked addition.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    /// Checked multiplication by scalar integer.
    pub fn checked_mul(self, rhs: U256) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    /// Checked division by scalar integer.
    pub fn checked_div(self, rhs: U256) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }

    /// Parses a base-10 string of atomic raw units without decimal point.
    /// Rejects leading zeros (except single "0") and signs per §9.
    pub fn parse_raw_str(s: &str) -> Result<Self, AmountError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AmountError::Empty);
        }
        if s.starts_with('-') || s.starts_with('+') {
            return Err(AmountError::InvalidFormat);
        }
        if s.len() > 1 && s.starts_with('0') {
            return Err(AmountError::InvalidFormat);
        }
        let val = U256::from_str_radix(s, 10).map_err(|_| AmountError::InvalidFormat)?;
        Ok(Self(val))
    }

    /// Parses a human-readable decimal string (e.g. "1.50", "100", "0.05") into 6-dp raw units.
    pub fn parse_decimal_str(s: &str) -> Result<Self, AmountError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AmountError::Empty);
        }
        if s.starts_with('-') {
            return Err(AmountError::Negative);
        }
        if s.starts_with('+') {
            return Err(AmountError::InvalidFormat);
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() > 2 {
            return Err(AmountError::InvalidFormat);
        }

        let whole_str = parts[0];
        let whole: U256 = if whole_str.is_empty() {
            U256::ZERO
        } else {
            if whole_str.len() > 1 && whole_str.starts_with('0') {
                return Err(AmountError::InvalidFormat);
            }
            U256::from_str_radix(whole_str, 10).map_err(|_| AmountError::InvalidFormat)?
        };

        let whole_scaled = whole
            .checked_mul(U256::from(USDC_SCALE))
            .ok_or(AmountError::Overflow)?;

        if parts.len() == 2 {
            let frac_str = parts[1];
            if frac_str.len() > USDC_DECIMALS as usize {
                return Err(AmountError::TooManyDecimals);
            }
            if !frac_str.chars().all(|c| c.is_ascii_digit()) {
                return Err(AmountError::InvalidFormat);
            }

            let mut frac_padded = frac_str.to_string();
            while frac_padded.len() < USDC_DECIMALS as usize {
                frac_padded.push('0');
            }

            let frac: U256 = if frac_padded.is_empty() {
                U256::ZERO
            } else {
                U256::from_str_radix(&frac_padded, 10).map_err(|_| AmountError::InvalidFormat)?
            };

            let total = whole_scaled
                .checked_add(frac)
                .ok_or(AmountError::Overflow)?;
            Ok(Self(total))
        } else {
            Ok(Self(whole_scaled))
        }
    }

    /// Formats as standard 6-decimal-place string (e.g. "1.500000").
    pub fn to_decimal_string(&self) -> String {
        let scale = U256::from(USDC_SCALE);
        let whole = self.0 / scale;
        let frac = self.0 % scale;
        format!("{whole}.{frac:06}")
    }

    /// Formats as 2-decimal-place display string (e.g. "1.50").
    pub fn to_display_string(&self) -> String {
        let scale = U256::from(USDC_SCALE);
        let whole = self.0 / scale;
        let frac_6 = (self.0 % scale).to::<u64>();
        let frac_2 = frac_6 / 10_000;
        format!("{whole}.{frac_2:02}")
    }

    /// Formats as raw base-10 string with no leading zeros (per §9 transport).
    pub fn to_raw_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Debug for RawAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawAmount({} raw, {})", self.0, self.to_decimal_string())
    }
}

impl fmt::Display for RawAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl Add for RawAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).expect("RawAmount overflow on add")
    }
}

impl Sub for RawAmount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).expect("RawAmount underflow on sub")
    }
}

impl Serialize for RawAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_raw_string())
    }
}

impl<'de> Deserialize<'de> for RawAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse_raw_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_decimals() {
        let a = RawAmount::parse_decimal_str("1.5").unwrap();
        assert_eq!(a.raw(), U256::from(1_500_000));
        assert_eq!(a.to_decimal_string(), "1.500000");
        assert_eq!(a.to_display_string(), "1.50");

        let b = RawAmount::parse_decimal_str("0.000001").unwrap();
        assert_eq!(b.raw(), U256::from(1));
        assert_eq!(b.to_decimal_string(), "0.000001");

        let zero = RawAmount::parse_decimal_str("0").unwrap();
        assert_eq!(zero.raw(), U256::ZERO);
        assert_eq!(zero.to_decimal_string(), "0.000000");

        let c = RawAmount::parse_decimal_str("100").unwrap();
        assert_eq!(c.raw(), U256::from(100_000_000));
    }

    #[test]
    fn parses_raw_strings() {
        let a = RawAmount::parse_raw_str("1500000").unwrap();
        assert_eq!(a.raw(), U256::from(1_500_000));
        assert_eq!(a.to_raw_string(), "1500000");

        let zero = RawAmount::parse_raw_str("0").unwrap();
        assert_eq!(zero.raw(), U256::ZERO);

        assert!(RawAmount::parse_raw_str("01").is_err());
        assert!(RawAmount::parse_raw_str("-1").is_err());
        assert!(RawAmount::parse_raw_str("").is_err());
    }

    #[test]
    fn checked_arithmetic() {
        let a = RawAmount::from_u64(100);
        let b = RawAmount::from_u64(50);
        assert_eq!(a.checked_add(b).unwrap(), RawAmount::from_u64(150));
        assert_eq!(a.checked_sub(b).unwrap(), RawAmount::from_u64(50));
        assert!(b.checked_sub(a).is_none());
    }

    #[test]
    fn rejects_more_than_6_decimals() {
        assert_eq!(
            RawAmount::parse_decimal_str("1.1234567"),
            Err(AmountError::TooManyDecimals)
        );
    }
}
