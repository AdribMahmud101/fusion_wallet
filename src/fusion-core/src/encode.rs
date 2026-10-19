//! Calldata encoding for Kavachi module and Safe smart account transactions (§10.2).
//!
//! Encodes exact ABI calldata for execute, queueAction, cancelQueuedAction, pause,
//! emergencyUnwind, session control, and Safe multi-sig transactions.

use crate::types::Action;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};

sol! {
    struct SolActionCalldata {
        uint8 kind;
        address sourceVault;
        address destinationVault;
        uint256 assets;
        uint256 minSharesOut;
        uint256 maxSharesBurn;
        bytes32 obligationIdHash;
        bytes32 payeeIdHash;
        address recipient;
        uint64 dueAt;
        uint256 expectedNonce;
        uint256 policyEpoch;
        uint64 deadline;
        bytes32 decisionHash;
    }

    interface IKavachiModuleCalls {
        function execute(SolActionCalldata action, bytes32[] calldata proof) external returns (bytes32 digest);
        function queueAction(SolActionCalldata action, bytes32[] calldata proof) external returns (bytes32 digest);
        function cancelQueuedAction(bytes32 digest) external;
        function pause(bytes32 reasonCode) external;
        function emergencyUnwind(bytes32 reasonCode) external;
        function startSession(address key, uint64 expiresAt, uint256 sessionLimitRaw) external;
        function resume(uint256 expectedEpoch) external;
        function approveAction(SolActionCalldata action, uint64 validUntil) external returns (bytes32 digest);
        function ownerPayObligation(bytes32 obligationIdHash, bytes32 payeeIdHash, uint256 assets, uint64 dueAt, bytes32[] calldata proof) external;
        function commitObligations(bytes32 root, uint256 reservedRaw, uint256 count) external;
    }

    interface ISafeTransaction {
        function execTransaction(
            address to,
            uint256 value,
            bytes calldata data,
            uint8 operation,
            uint256 safeTxGas,
            uint256 baseGas,
            uint256 gasPrice,
            address gasToken,
            address refundReceiver,
            bytes calldata signatures
        ) external returns (bool success);
    }
}

/// Encoded transaction calldata targeting either the module or Safe smart account (§10.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedCall {
    pub to: Address,
    pub data: String, // hex encoded with 0x prefix
}

fn to_sol_action_calldata(action: &Action) -> SolActionCalldata {
    SolActionCalldata {
        kind: action.kind.as_u8(),
        sourceVault: action.source_vault,
        destinationVault: action.destination_vault,
        assets: action.assets,
        minSharesOut: action.min_shares_out,
        maxSharesBurn: action.max_shares_burn,
        obligationIdHash: action.obligation_id_hash,
        payeeIdHash: action.payee_id_hash,
        recipient: action.recipient,
        dueAt: action.due_at,
        expectedNonce: action.expected_nonce,
        policyEpoch: action.policy_epoch,
        deadline: action.deadline,
        decisionHash: action.decision_hash,
    }
}

/// Encodes `IKavachiModule.execute(action, proof)`.
pub fn encode_execute(module: Address, action: &Action, proof: &[B256]) -> EncodedCall {
    let call = IKavachiModuleCalls::executeCall {
        action: to_sol_action_calldata(action),
        proof: proof.to_vec(),
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.queueAction(action, proof)`.
pub fn encode_queue_action(module: Address, action: &Action, proof: &[B256]) -> EncodedCall {
    let call = IKavachiModuleCalls::queueActionCall {
        action: to_sol_action_calldata(action),
        proof: proof.to_vec(),
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.cancelQueuedAction(digest)`.
pub fn encode_cancel_queued_action(module: Address, digest: B256) -> EncodedCall {
    let call = IKavachiModuleCalls::cancelQueuedActionCall { digest };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.pause(reasonCode)`.
pub fn encode_pause(module: Address, reason_code: B256) -> EncodedCall {
    let call = IKavachiModuleCalls::pauseCall {
        reasonCode: reason_code,
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.emergencyUnwind(reasonCode)`.
pub fn encode_emergency_unwind(module: Address, reason_code: B256) -> EncodedCall {
    let call = IKavachiModuleCalls::emergencyUnwindCall {
        reasonCode: reason_code,
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.startSession(key, expiresAt, sessionLimitRaw)`.
pub fn encode_start_session(
    module: Address,
    key: Address,
    expires_at: u64,
    session_limit_raw: U256,
) -> EncodedCall {
    let call = IKavachiModuleCalls::startSessionCall {
        key,
        expiresAt: expires_at,
        sessionLimitRaw: session_limit_raw,
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.resume(expectedEpoch)`.
pub fn encode_resume(module: Address, expected_epoch: U256) -> EncodedCall {
    let call = IKavachiModuleCalls::resumeCall {
        expectedEpoch: expected_epoch,
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes `IKavachiModule.commitObligations(root, reservedRaw, count)`.
pub fn encode_commit_obligations(
    module: Address,
    root: B256,
    reserved_raw: U256,
    count: U256,
) -> EncodedCall {
    let call = IKavachiModuleCalls::commitObligationsCall {
        root,
        reservedRaw: reserved_raw,
        count,
    };
    EncodedCall {
        to: module,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Encodes Safe `execTransaction` call.
#[allow(clippy::too_many_arguments)]
pub fn encode_safe_exec_transaction(
    safe: Address,
    to: Address,
    value: U256,
    data: Vec<u8>,
    operation: u8,
    signatures: Vec<u8>,
) -> EncodedCall {
    let call = ISafeTransaction::execTransactionCall {
        to,
        value,
        data: Bytes::from(data),
        operation,
        safeTxGas: U256::ZERO,
        baseGas: U256::ZERO,
        gasPrice: U256::ZERO,
        gasToken: Address::ZERO,
        refundReceiver: Address::ZERO,
        signatures: Bytes::from(signatures),
    };
    EncodedCall {
        to: safe,
        data: format!("{:#x}", Bytes::from(call.abi_encode())),
    }
}

/// Dispatches call encoding based on kind and JSON arguments (Appendix B).
pub fn encode_call(kind: &str, args_json: &str) -> Result<EncodedCall, String> {
    let args: serde_json::Value =
        serde_json::from_str(args_json).map_err(|e| format!("Invalid args JSON: {e}"))?;

    match kind.to_uppercase().as_str() {
        "EXECUTE" => {
            let module: Address = args["module"]
                .as_str()
                .ok_or("Missing module")?
                .parse()
                .map_err(|e| format!("Invalid module: {e}"))?;
            let action: Action = serde_json::from_value(args["action"].clone())
                .map_err(|e| format!("Invalid action: {e}"))?;
            let proof_vals = args["proof"].as_array().ok_or("Missing proof array")?;
            let mut proof = Vec::new();
            for p in proof_vals {
                let s = p.as_str().ok_or("Proof item must be hex string")?;
                let b: B256 = s.parse().map_err(|e| format!("Invalid proof B256: {e}"))?;
                proof.push(b);
            }
            Ok(encode_execute(module, &action, &proof))
        }
        "QUEUE" => {
            let module: Address = args["module"]
                .as_str()
                .ok_or("Missing module")?
                .parse()
                .map_err(|e| format!("Invalid module: {e}"))?;
            let action: Action = serde_json::from_value(args["action"].clone())
                .map_err(|e| format!("Invalid action: {e}"))?;
            let proof_vals = args["proof"].as_array().ok_or("Missing proof array")?;
            let mut proof = Vec::new();
            for p in proof_vals {
                let s = p.as_str().ok_or("Proof item must be hex string")?;
                let b: B256 = s.parse().map_err(|e| format!("Invalid proof B256: {e}"))?;
                proof.push(b);
            }
            Ok(encode_queue_action(module, &action, &proof))
        }
        "CANCEL" => {
            let module: Address = args["module"]
                .as_str()
                .ok_or("Missing module")?
                .parse()
                .map_err(|e| format!("Invalid module: {e}"))?;
            let digest: B256 = args["digest"]
                .as_str()
                .ok_or("Missing digest")?
                .parse()
                .map_err(|e| format!("Invalid digest: {e}"))?;
            Ok(encode_cancel_queued_action(module, digest))
        }
        "PAUSE" => {
            let module: Address = args["module"]
                .as_str()
                .ok_or("Missing module")?
                .parse()
                .map_err(|e| format!("Invalid module: {e}"))?;
            let reason_code: B256 = args["reasonCode"]
                .as_str()
                .ok_or("Missing reasonCode")?
                .parse()
                .map_err(|e| format!("Invalid reasonCode: {e}"))?;
            Ok(encode_pause(module, reason_code))
        }
        "UNWIND" => {
            let module: Address = args["module"]
                .as_str()
                .ok_or("Missing module")?
                .parse()
                .map_err(|e| format!("Invalid module: {e}"))?;
            let reason_code: B256 = args["reasonCode"]
                .as_str()
                .ok_or("Missing reasonCode")?
                .parse()
                .map_err(|e| format!("Invalid reasonCode: {e}"))?;
            Ok(encode_emergency_unwind(module, reason_code))
        }
        other => Err(format!("Unsupported encode call kind: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ActionKind;

    #[test]
    fn encodes_execute_calldata() {
        let module = Address::from_slice(&[0xB0; 20]);
        let action = Action {
            kind: ActionKind::Deposit,
            source_vault: Address::ZERO,
            destination_vault: Address::from_slice(&[0xA1; 20]),
            assets: U256::from(10_000_000),
            min_shares_out: U256::from(9_950_000),
            max_shares_burn: U256::ZERO,
            obligation_id_hash: B256::ZERO,
            payee_id_hash: B256::ZERO,
            recipient: Address::ZERO,
            due_at: 0,
            expected_nonce: U256::ZERO,
            policy_epoch: U256::from(1),
            deadline: 1_700_000_120,
            decision_hash: B256::repeat_byte(0x55),
        };

        let encoded = encode_execute(module, &action, &[]);
        assert_eq!(encoded.to, module);
        assert!(encoded.data.starts_with("0x"));
        // Check function selector for execute(SolActionCalldata,bytes32[])
        let expected_selector = &IKavachiModuleCalls::executeCall::SELECTOR;
        let selector_hex = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            expected_selector[0], expected_selector[1], expected_selector[2], expected_selector[3]
        );
        assert!(encoded.data.starts_with(&format!("0x{selector_hex}")));
    }
}
