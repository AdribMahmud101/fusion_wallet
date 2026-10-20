//! EIP-1559 transaction signing with zeroized secp256k1 keys (§10.2, §11.3).
//!
//! Keys are zeroized on drop. Private keys never cross FFI into the UI process.

use alloy::consensus::{SignableTransaction, TxEip1559, TxEnvelope};
use alloy::hex;
use alloy::network::eip2718::Encodable2718;
use alloy::network::TxSignerSync;
use alloy::primitives::{Address, Bytes, TxHash, TxKind, B256, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

// Read the authoritative contract interface, including its Action tuple. No copied selectors.
alloy::sol!("../contracts/src/IKavachiModule.sol");

/// Trusted deployment configuration, supplied independently of the prepared transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionSigningPolicy {
    pub module: Address,
    pub chain_id: u64,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SignerRefused {
    #[error("SignerRefused::NotModuleCall")]
    NotModuleCall,
    #[error("SignerRefused::NonzeroValue")]
    NonzeroValue,
    #[error("SignerRefused::WrongChain")]
    WrongChain,
    #[error("SignerRefused::UnsupportedSelector")]
    UnsupportedSelector,
    #[error("SignerRefused::MalformedCalldata")]
    MalformedCalldata,
}

impl SessionSigningPolicy {
    fn envelope(&self, tx: &PreparedTransaction) -> Result<Vec<u8>, SignerRefused> {
        if self.module == Address::ZERO || tx.to != self.module {
            return Err(SignerRefused::NotModuleCall);
        }
        if tx.value != U256::ZERO {
            return Err(SignerRefused::NonzeroValue);
        }
        if self.chain_id == 0 || tx.chain_id != self.chain_id {
            return Err(SignerRefused::WrongChain);
        }
        hex::decode(tx.data.strip_prefix("0x").unwrap_or(&tx.data))
            .map_err(|_| SignerRefused::MalformedCalldata)
    }

    pub fn validate(&self, tx: &PreparedTransaction) -> Result<(), SignerRefused> {
        let data = self.envelope(tx)?;
        let selector = data.get(..4).ok_or(SignerRefused::MalformedCalldata)?;
        // Decode and re-encode too: a selector prefix alone is not a module call.
        let canonical = if selector == IKavachiModule::executeCall::SELECTOR {
            IKavachiModule::executeCall::abi_decode_validate(&data).map(|c| c.abi_encode())
        } else if selector == IKavachiModule::queueActionCall::SELECTOR {
            IKavachiModule::queueActionCall::abi_decode_validate(&data).map(|c| c.abi_encode())
        } else {
            return Err(SignerRefused::UnsupportedSelector);
        }
        .map_err(|_| SignerRefused::MalformedCalldata)?;
        if canonical != data {
            return Err(SignerRefused::MalformedCalldata);
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignError {
    #[error(transparent)]
    Refused(#[from] SignerRefused),
    #[error("Invalid private key format")]
    InvalidKey,
    #[error("Serialization error: {0}")]
    SerdeError(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

/// Prepared EIP-1559 transaction request for signing (§10.2, Appendix B).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedTransaction {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: Address,
    #[serde(default)]
    pub value: U256,
    pub data: String, // hex encoded
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
}

/// Output of signing a prepared transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedTransactionOutput {
    pub raw_tx: String, // hex encoded 0x02 || rlp(...)
    pub tx_hash: TxHash,
}

/// Signs a prepared EIP-1559 transaction with a zeroized 32-byte private key.
pub fn sign_eip1559_transaction(
    tx: &PreparedTransaction,
    private_key: &Zeroizing<[u8; 32]>,
    policy: &SessionSigningPolicy,
) -> Result<SignedTransactionOutput, SignError> {
    policy.validate(tx)?;
    sign_checked_transaction(tx, private_key)
}

/// Separate guardian authority: only a zero-value module pause, never a session entry point.
pub fn sign_guardian_pause_transaction(
    tx: &PreparedTransaction,
    private_key: &Zeroizing<[u8; 32]>,
    policy: &SessionSigningPolicy,
) -> Result<SignedTransactionOutput, SignError> {
    let data = policy.envelope(tx)?;
    if data.get(..4) != Some(IKavachiModule::pauseCall::SELECTOR.as_slice()) {
        return Err(SignerRefused::UnsupportedSelector.into());
    }
    let call = IKavachiModule::pauseCall::abi_decode_validate(&data)
        .map_err(|_| SignerRefused::MalformedCalldata)?;
    if call.abi_encode() != data {
        return Err(SignerRefused::MalformedCalldata.into());
    }
    sign_checked_transaction(tx, private_key)
}

// Only reachable after an authority-specific allowlist check.
fn sign_checked_transaction(
    tx: &PreparedTransaction,
    private_key: &Zeroizing<[u8; 32]>,
) -> Result<SignedTransactionOutput, SignError> {
    let key_b256 = B256::from_slice(private_key.as_ref());
    let signer = PrivateKeySigner::from_bytes(&key_b256).map_err(|_| SignError::InvalidKey)?;

    let call_data = if let Some(stripped) = tx.data.strip_prefix("0x") {
        hex::decode(stripped).map_err(|e| SignError::SerdeError(e.to_string()))?
    } else {
        hex::decode(&tx.data).map_err(|e| SignError::SerdeError(e.to_string()))?
    };

    let mut eip1559_tx = TxEip1559 {
        chain_id: tx.chain_id,
        nonce: tx.nonce,
        gas_limit: tx.gas_limit,
        max_fee_per_gas: tx.max_fee_per_gas,
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
        to: TxKind::Call(tx.to),
        value: tx.value,
        access_list: Default::default(),
        input: Bytes::from(call_data),
    };

    let sig = signer
        .sign_transaction_sync(&mut eip1559_tx)
        .map_err(|e| SignError::SigningFailed(e.to_string()))?;

    let envelope: TxEnvelope = eip1559_tx.into_signed(sig).into();
    let tx_hash = *envelope.tx_hash();
    let raw_tx = format!("{:#x}", Bytes::from(envelope.encoded_2718()));

    Ok(SignedTransactionOutput { raw_tx, tx_hash })
}

/// Parses key from hex and signs transaction JSON (§10.2, Appendix B).
pub fn sign_prepared_json(
    tx_json: &str,
    key_hex: &str,
    policy_json: &str,
) -> Result<SignedTransactionOutput, SignError> {
    let policy: SessionSigningPolicy =
        serde_json::from_str(policy_json).map_err(|e| SignError::SerdeError(e.to_string()))?;
    let trimmed = key_hex.trim();
    let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let mut key_bytes = [0u8; 32];
    hex::decode_to_slice(hex_str, &mut key_bytes).map_err(|_| SignError::InvalidKey)?;
    let zeroized_key = Zeroizing::new(key_bytes);

    let tx: PreparedTransaction =
        serde_json::from_str(tx_json).map_err(|e| SignError::SerdeError(e.to_string()))?;

    sign_eip1559_transaction(&tx, &zeroized_key, &policy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // Resolve the real transaction through golden_app.json rather than a hand-built selector.
    fn golden() -> (PreparedTransaction, SessionSigningPolicy) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rows: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("evidence/app/agent/golden_app.json")).unwrap(),
        )
        .unwrap();
        let tx: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(rows[0]["transaction"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let q = |name: &str| {
            u64::from_str_radix(tx[name].as_str().unwrap().trim_start_matches("0x"), 16).unwrap()
        };
        let policy = SessionSigningPolicy {
            module: rows[0]["to"].as_str().unwrap().parse().unwrap(),
            chain_id: rows[0]["chainId"].as_u64().unwrap(),
        };
        let prepared = PreparedTransaction {
            chain_id: q("chainId"),
            nonce: q("nonce"),
            to: tx["to"].as_str().unwrap().parse().unwrap(),
            value: U256::ZERO,
            data: tx["input"].as_str().unwrap().into(),
            gas_limit: q("gas"),
            max_fee_per_gas: q("maxFeePerGas") as u128,
            max_priority_fee_per_gas: q("maxPriorityFeePerGas") as u128,
        };
        (prepared, policy)
    }
    fn refused(reason: SignerRefused, change: impl FnOnce(&mut PreparedTransaction)) {
        let (mut tx, policy) = golden();
        change(&mut tx);
        // Invalid key proves the refusal runs before key parsing / signing.
        let error = sign_eip1559_transaction(&tx, &Zeroizing::new([0; 32]), &policy).unwrap_err();
        assert!(matches!(error, SignError::Refused(actual) if actual == reason));
    }
    #[test]
    fn real_golden_app_transaction_is_still_signable_and_deterministic() {
        let (tx, policy) = golden();
        let key = Zeroizing::new([0x42; 32]);
        let a = sign_eip1559_transaction(&tx, &key, &policy).unwrap();
        let b = sign_eip1559_transaction(&tx, &key, &policy).unwrap();
        assert_eq!(a, b);
        assert!(a.raw_tx.starts_with("0x02"));
        let json_result = sign_prepared_json(
            &serde_json::to_string(&tx).unwrap(),
            &"42".repeat(32),
            &serde_json::to_string(&policy).unwrap(),
        )
        .unwrap();
        assert_eq!(a, json_result);
    }
    #[test]
    fn queue_entrypoint_is_signable() {
        let (mut tx, policy) = golden();
        tx.data.replace_range(
            2..10,
            &hex::encode(IKavachiModule::queueActionCall::SELECTOR),
        );
        assert!(sign_eip1559_transaction(&tx, &Zeroizing::new([0x42; 32]), &policy).is_ok());
    }
    #[test]
    fn refuses_wrong_destination() {
        refused(SignerRefused::NotModuleCall, |tx| {
            tx.to = Address::repeat_byte(0xee)
        });
    }
    #[test]
    fn refuses_native_value() {
        refused(SignerRefused::NonzeroValue, |tx| tx.value = U256::from(1));
    }
    #[test]
    fn refuses_wrong_chain() {
        refused(SignerRefused::WrongChain, |tx| tx.chain_id = 1);
    }
    #[test]
    fn refuses_erc20_even_at_module_address() {
        refused(SignerRefused::UnsupportedSelector, |tx| {
            tx.data = format!("0xa9059cbb{:0>64}{:064x}", "ee".repeat(20), 99_000_000u64)
        });
    }
    #[test]
    fn refuses_owner_and_guardian_entrypoints() {
        for data in [
            IKavachiModule::resumeCall {
                expectedEpoch: U256::from(1),
            }
            .abi_encode(),
            IKavachiModule::pauseCall {
                reasonCode: B256::ZERO,
            }
            .abi_encode(),
            IKavachiModule::startSessionCall {
                key: Address::repeat_byte(0x42),
                expiresAt: 123,
                sessionLimitRaw: U256::from(1),
            }
            .abi_encode(),
        ] {
            refused(SignerRefused::UnsupportedSelector, |tx| {
                tx.data = format!("0x{}", hex::encode(data))
            });
        }
    }
    #[test]
    fn refuses_missing_truncated_invalid_and_noncanonical_calldata() {
        for data in ["0x", "0x1234", "0xzzzzzzzz"] {
            refused(SignerRefused::MalformedCalldata, |tx| tx.data = data.into());
        }
        refused(SignerRefused::MalformedCalldata, |tx| tx.data.truncate(10));
        refused(SignerRefused::MalformedCalldata, |tx| {
            tx.data.push_str("00")
        });
    }
    #[test]
    fn a12_exact_prepared_transaction_is_refused() {
        let (mut tx, mut policy) = golden();
        policy.chain_id = 31337;
        tx.chain_id = 31337;
        tx.nonce = 0;
        tx.to = Address::repeat_byte(0xee);
        tx.data = format!("0xa9059cbb{:0>64}{:064x}", "ee".repeat(20), 99_000_000u64);
        tx.gas_limit = 100_000;
        tx.max_fee_per_gas = 1_000_000_000;
        tx.max_priority_fee_per_gas = 1_000_000;
        assert!(matches!(
            sign_eip1559_transaction(&tx, &Zeroizing::new([0x42; 32]), &policy),
            Err(SignError::Refused(SignerRefused::NotModuleCall))
        ));
    }
    #[test]
    fn guardian_pause_is_separate_and_bounded() {
        let (mut tx, policy) = golden();
        let key = Zeroizing::new([0x42; 32]);
        assert!(sign_guardian_pause_transaction(&tx, &key, &policy).is_err());
        tx.data = format!(
            "0x{}",
            hex::encode(
                IKavachiModule::pauseCall {
                    reasonCode: B256::ZERO
                }
                .abi_encode()
            )
        );
        assert!(sign_guardian_pause_transaction(&tx, &key, &policy).is_ok());
        tx.value = U256::from(1);
        assert!(sign_guardian_pause_transaction(&tx, &key, &policy).is_err());
    }
}
