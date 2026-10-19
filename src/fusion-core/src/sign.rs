//! EIP-1559 transaction signing with zeroized secp256k1 keys (§10.2, §11.3).
//!
//! Keys are zeroized on drop. Private keys never cross FFI into the UI process.

use alloy::consensus::{SignableTransaction, TxEip1559, TxEnvelope};
use alloy::hex;
use alloy::network::eip2718::Encodable2718;
use alloy::network::TxSignerSync;
use alloy::primitives::{Address, Bytes, TxHash, TxKind, B256, U256};
use alloy::signers::local::PrivateKeySigner;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(thiserror::Error, Debug)]
pub enum SignError {
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
) -> Result<SignedTransactionOutput, SignError> {
    let trimmed = key_hex.trim();
    let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let mut key_bytes = [0u8; 32];
    hex::decode_to_slice(hex_str, &mut key_bytes).map_err(|_| SignError::InvalidKey)?;
    let zeroized_key = Zeroizing::new(key_bytes);

    let tx: PreparedTransaction =
        serde_json::from_str(tx_json).map_err(|e| SignError::SerdeError(e.to_string()))?;

    sign_eip1559_transaction(&tx, &zeroized_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_eip1559_deterministic() {
        let key = Zeroizing::new([0x42u8; 32]);
        let tx = PreparedTransaction {
            chain_id: 84532,
            nonce: 7,
            to: Address::repeat_byte(0x11),
            value: U256::ZERO,
            data: "0x12345678".to_string(),
            gas_limit: 100_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000,
        };

        let res1 = sign_eip1559_transaction(&tx, &key).expect("signing succeeds");
        let res2 = sign_eip1559_transaction(&tx, &key).expect("signing succeeds");

        assert_eq!(res1.raw_tx, res2.raw_tx);
        assert_eq!(res1.tx_hash, res2.tx_hash);
        assert!(res1.raw_tx.starts_with("0x02"));
    }

    #[test]
    fn test_sign_prepared_json() {
        let key_hex = "42".repeat(32);
        let tx_json = r#"{
            "chainId": 31337,
            "nonce": 0,
            "to": "0x0000000000000000000000000000000000000001",
            "value": "0",
            "data": "0x",
            "gasLimit": 21000,
            "maxFeePerGas": 1000000000,
            "maxPriorityFeePerGas": 1000000
        }"#;

        let res = sign_prepared_json(tx_json, &key_hex).expect("valid sign");
        assert!(res.raw_tx.starts_with("0x02"));
    }
}
