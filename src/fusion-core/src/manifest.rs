//! Deployment manifest parse and validation (§10.3).
//!
//! Signed build configuration for Kavachi deployments. Verified before connecting.

use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Profile of the deployment (§10.3).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ManifestProfile {
    Full,
    Scaled,
}

impl fmt::Display for ManifestProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "FULL"),
            Self::Scaled => write!(f, "SCALED"),
        }
    }
}

/// Errors occurring during manifest parsing or validation.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    #[error("Failed to parse manifest JSON: {0}")]
    JsonError(String),
    #[error(
        "Unsupported chain ID: {0}. Supported chains are 31337 (Anvil) and 84532 (Base Sepolia)"
    )]
    UnsupportedChainId(u64),
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
    #[error("Address is zero for required field: {0}")]
    ZeroAddress(&'static str),
    #[error("Missing code hash for required contract: {0}")]
    MissingCodeHash(String),
}

/// Deployment manifest (§10.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentManifest {
    pub profile: ManifestProfile,
    pub chain_id: u64,
    pub rpc_url: String,
    pub safe: Address,
    pub module: Address,
    pub asset: Address,
    pub vault_a: Address,
    pub vault_b: Address,
    pub sanctions_list: Address,
    #[serde(default)]
    pub code_hashes: BTreeMap<String, B256>,
    pub deployment_block: u64,
    pub explorer_base_url: String,
}

impl DeploymentManifest {
    /// Parses a manifest from a JSON string and validates it.
    pub fn parse_and_validate(json_str: &str) -> Result<Self, ManifestError> {
        let manifest: Self =
            serde_json::from_str(json_str).map_err(|e| ManifestError::JsonError(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates configuration invariants per §10.3.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // chainId ∈ {31337, 84532}
        if self.chain_id != 31337 && self.chain_id != 84532 {
            return Err(ManifestError::UnsupportedChainId(self.chain_id));
        }

        // Validate RPC URL format
        if !self.rpc_url.starts_with("http://")
            && !self.rpc_url.starts_with("https://")
            && !self.rpc_url.starts_with("ws://")
            && !self.rpc_url.starts_with("wss://")
        {
            return Err(ManifestError::InvalidRpcUrl(self.rpc_url.clone()));
        }

        // Validate critical contract addresses are non-zero
        if self.safe == Address::ZERO {
            return Err(ManifestError::ZeroAddress("safe"));
        }
        if self.module == Address::ZERO {
            return Err(ManifestError::ZeroAddress("module"));
        }
        if self.asset == Address::ZERO {
            return Err(ManifestError::ZeroAddress("asset"));
        }
        if self.vault_a == Address::ZERO {
            return Err(ManifestError::ZeroAddress("vaultA"));
        }
        if self.vault_b == Address::ZERO {
            return Err(ManifestError::ZeroAddress("vaultB"));
        }
        if self.sanctions_list == Address::ZERO {
            return Err(ManifestError::ZeroAddress("sanctionsList"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_valid_manifest() {
        let json = r#"{
            "profile": "FULL",
            "chainId": 84532,
            "rpcUrl": "https://sepolia.base.org",
            "safe": "0x000000000000000000000000000000000000dEaD",
            "module": "0x00000000000000000000000000000000000000B0",
            "asset": "0x0000000000000000000000000000000000000001",
            "vaultA": "0x00000000000000000000000000000000000000A1",
            "vaultB": "0x00000000000000000000000000000000000000b2",
            "sanctionsList": "0x0000000000000000000000000000000000000002",
            "codeHashes": {},
            "deploymentBlock": 12345,
            "explorerBaseUrl": "https://sepolia.basescan.org"
        }"#;

        let m = DeploymentManifest::parse_and_validate(json).unwrap();
        assert_eq!(m.profile, ManifestProfile::Full);
        assert_eq!(m.chain_id, 84532);
    }

    #[test]
    fn rejects_unsupported_chain_id() {
        let json = r#"{
            "profile": "FULL",
            "chainId": 1,
            "rpcUrl": "https://mainnet.infura.io",
            "safe": "0x000000000000000000000000000000000000dEaD",
            "module": "0x00000000000000000000000000000000000000B0",
            "asset": "0x0000000000000000000000000000000000000001",
            "vaultA": "0x00000000000000000000000000000000000000A1",
            "vaultB": "0x00000000000000000000000000000000000000b2",
            "sanctionsList": "0x0000000000000000000000000000000000000002",
            "codeHashes": {},
            "deploymentBlock": 12345,
            "explorerBaseUrl": "https://etherscan.io"
        }"#;

        assert!(matches!(
            DeploymentManifest::parse_and_validate(json),
            Err(ManifestError::UnsupportedChainId(1))
        ));
    }
}
