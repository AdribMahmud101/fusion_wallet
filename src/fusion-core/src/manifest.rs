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
    #[serde(default, deserialize_with = "deserialize_vault_implementations")]
    pub vault_implementations: BTreeMap<Address, Address>,
    pub deployment_block: u64,
    pub explorer_base_url: String,
}

fn deserialize_vault_implementations<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<Address, Address>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<BTreeMap<String, String>> = Option::deserialize(deserializer)?;
    let mut map = BTreeMap::new();
    if let Some(raw_map) = opt {
        for (k, v) in raw_map {
            if let (Ok(k_addr), Ok(v_addr)) = (k.parse::<Address>(), v.parse::<Address>()) {
                map.insert(k_addr, v_addr);
            }
        }
    }
    Ok(map)
}

impl DeploymentManifest {
    /// Parses a manifest from a JSON string and validates it.
    pub fn parse_and_validate(json_str: &str) -> Result<Self, ManifestError> {
        let mut manifest: Self =
            serde_json::from_str(json_str).map_err(|e| ManifestError::JsonError(e.to_string()))?;

        // Also check if json has contracts.vaultA.implementation or contracts.vaultB.implementation
        // or vaultImplementations with "vaultA" / "vaultB" keys
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(contracts) = val.get("contracts") {
                if let Some(va_impl) = contracts
                    .get("vaultA")
                    .and_then(|v| v.get("implementation"))
                    .and_then(|i| i.as_str())
                {
                    if let Ok(addr) = va_impl.parse::<Address>() {
                        manifest
                            .vault_implementations
                            .insert(manifest.vault_a, addr);
                    }
                }
                if let Some(vb_impl) = contracts
                    .get("vaultB")
                    .and_then(|v| v.get("implementation"))
                    .and_then(|i| i.as_str())
                {
                    if let Ok(addr) = vb_impl.parse::<Address>() {
                        manifest
                            .vault_implementations
                            .insert(manifest.vault_b, addr);
                    }
                }
            }
            if let Some(v_impls) = val
                .get("vaultImplementations")
                .or_else(|| val.get("vault_implementations"))
            {
                if let Some(obj) = v_impls.as_object() {
                    for (k, v) in obj {
                        if let Some(v_str) = v.as_str() {
                            if let Ok(impl_addr) = v_str.parse::<Address>() {
                                if k.eq_ignore_ascii_case("vaultA") || k == "vault_a" {
                                    manifest
                                        .vault_implementations
                                        .insert(manifest.vault_a, impl_addr);
                                } else if k.eq_ignore_ascii_case("vaultB") || k == "vault_b" {
                                    manifest
                                        .vault_implementations
                                        .insert(manifest.vault_b, impl_addr);
                                } else if let Ok(vault_addr) = k.parse::<Address>() {
                                    manifest.vault_implementations.insert(vault_addr, impl_addr);
                                }
                            }
                        }
                    }
                }
            }
        }

        manifest.validate()?;
        Ok(manifest)
    }

    /// Returns the expected EIP-1967 implementation address for a vault if pinned.
    pub fn get_vault_implementation(&self, vault: &Address) -> Option<Address> {
        self.vault_implementations.get(vault).copied()
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

    #[test]
    fn parses_vault_implementations_from_contracts_and_field() {
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
            "contracts": {
                "vaultA": {
                    "address": "0x00000000000000000000000000000000000000A1",
                    "implementation": "0x00000000000000000000000000000000000001A1"
                }
            },
            "codeHashes": {},
            "deploymentBlock": 12345,
            "explorerBaseUrl": "https://sepolia.basescan.org"
        }"#;

        let m = DeploymentManifest::parse_and_validate(json).unwrap();
        let expected_impl: Address = "0x00000000000000000000000000000000000001A1"
            .parse()
            .unwrap();
        assert_eq!(m.get_vault_implementation(&m.vault_a), Some(expected_impl));
    }
}
