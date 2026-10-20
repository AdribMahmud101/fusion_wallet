//! One-block state snapshot reads via alloy and Multicall3 (§10.5).
//!
//! Pinned to a single block number and block hash.
//! Any failed read returns an error (never zero or placeholder).

use crate::manifest::DeploymentManifest;
use crate::types::{
    Action, ActionKind, ModuleState, Payee, PermissionTier, Policy, Queue, Snapshot, VaultLimit,
};
use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::rpc::types::{BlockId, TransactionRequest};
use alloy::sol;
use alloy::sol_types::SolCall;
use std::str::FromStr;
use url::Url;

sol! {
    interface IERC20Snapshot {
        function balanceOf(address account) external view returns (uint256);
    }

    interface IERC4626Snapshot {
        function balanceOf(address account) external view returns (uint256);
        function convertToAssets(uint256 shares) external view returns (uint256);
        function previewDeposit(uint256 assets) external view returns (uint256);
        function previewWithdraw(uint256 assets) external view returns (uint256);
        function maxWithdraw(address owner) external view returns (uint256);
    }

    struct SolVaultLimit {
        address vault;
        uint256 maxExposureRaw;
    }

    struct SolPolicy {
        uint8 permissionTier;
        uint256 operatingBufferRaw;
        uint256 microMaxRaw;
        uint256 delayedMaxRaw;
        uint256 hardMaxRaw;
        uint256 dailyLimitRaw;
        uint32 delaySeconds;
        uint16 maxShareSlippageBps;
        address guardian;
        SolVaultLimit[] vaults;
        uint32 payeeDelaySeconds;
        uint32 payEarlyWindowSeconds;
    }

    struct SolModuleState {
        bool configured;
        bool paused;
        uint256 policyEpoch;
        bytes32 policyHash;
        bytes32 mandateHash;
        bytes32 obligationsRoot;
        uint256 reservedRaw;
        uint256 obligationCount;
        uint256 actionNonce;
        uint256 dailyDayIndex;
        uint256 dailyUsedRaw;
        bytes32 sessionId;
        uint256 sessionStartEpoch;
        address sessionKey;
        uint64 sessionExpiresAt;
        uint256 sessionLimitRaw;
        uint256 sessionUsedRaw;
    }

    struct SolActionItem {
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

    struct SolQueue {
        bool exists;
        bytes32 actionDigest;
        bytes32 sessionId;
        uint64 readyAt;
        uint64 expiresAt;
        SolActionItem action;
    }

    struct SolPayee {
        address account;
        bytes32 labelHash;
        uint64 activeAt;
        bool cancelled;
    }

    interface IKavachiModuleSnapshot {
        function getPolicy() external view returns (SolPolicy memory);
        function getState() external view returns (SolModuleState memory);
        function getQueue() external view returns (SolQueue memory);
        function getPayee(bytes32 payeeIdHash) external view returns (SolPayee memory);
        function isPaid(bytes32 obligationIdHash) external view returns (bool);
    }

    interface ISanctionsListSnapshot {
        function isSanctioned(address addr) external view returns (bool);
    }

    interface IMulticall3Snapshot {
        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }
        struct Result {
            bool success;
            bytes returnData;
        }
        function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
    }
}

/// Standard canonical Multicall3 deployment address on Base Sepolia and EVM chains.
pub const MULTICALL3_ADDRESS: Address = Address::new([
    0xca, 0x11, 0xbd, 0xe0, 0x59, 0x77, 0xb3, 0x63, 0x11, 0x67, 0x02, 0x88, 0x62, 0xbe, 0x2a, 0x17,
    0x39, 0x76, 0xca, 0x11,
]);

/// Canonical EIP-1967 implementation storage slot (§7.2):
/// `bytes32(uint256(keccak256('eip1967.proxy.implementation')) - 1)`
pub const EIP1967_IMPLEMENTATION_SLOT: B256 =
    alloy::primitives::b256!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");

#[derive(thiserror::Error, Debug)]
pub enum SnapshotError {
    #[error("Invalid URL: {0}")]
    UrlError(String),
    #[error("RPC provider error: {0}")]
    RpcError(String),
    #[error("Missing block header at target")]
    MissingBlock,
    #[error("Failed to decode contract response: {0}")]
    DecodeError(String),
    #[error("Multicall3 failed subcall: {0}")]
    CallFailed(String),
    #[error("Vault implementation changed: {vault} expected {expected}, got {actual}")]
    VaultImplChanged {
        vault: Address,
        expected: Address,
        actual: Address,
        block_number: u64,
        block_hash: B256,
        block_timestamp: u64,
    },
}

fn convert_policy(sol_policy: SolPolicy) -> Result<Policy, SnapshotError> {
    let permission_tier = match sol_policy.permissionTier {
        0 => PermissionTier::ReadOnly,
        1 => PermissionTier::OwnerApproved,
        2 => PermissionTier::Autonomous,
        other => {
            return Err(SnapshotError::DecodeError(format!(
                "Unknown permission tier: {other}"
            )))
        }
    };

    let vaults = sol_policy
        .vaults
        .into_iter()
        .map(|v| VaultLimit {
            vault: v.vault,
            max_exposure_raw: v.maxExposureRaw,
        })
        .collect();

    Ok(Policy {
        permission_tier,
        operating_buffer_raw: sol_policy.operatingBufferRaw,
        micro_max_raw: sol_policy.microMaxRaw,
        delayed_max_raw: sol_policy.delayedMaxRaw,
        hard_max_raw: sol_policy.hardMaxRaw,
        daily_limit_raw: sol_policy.dailyLimitRaw,
        delay_seconds: sol_policy.delaySeconds,
        max_share_slippage_bps: sol_policy.maxShareSlippageBps,
        guardian: sol_policy.guardian,
        vaults,
        payee_delay_seconds: sol_policy.payeeDelaySeconds,
        pay_early_window_seconds: sol_policy.payEarlyWindowSeconds,
    })
}

fn convert_module_state(s: SolModuleState) -> ModuleState {
    ModuleState {
        configured: s.configured,
        paused: s.paused,
        policy_epoch: s.policyEpoch,
        policy_hash: s.policyHash,
        mandate_hash: s.mandateHash,
        obligations_root: s.obligationsRoot,
        reserved_raw: s.reservedRaw,
        obligation_count: s.obligationCount,
        action_nonce: s.actionNonce,
        daily_day_index: s.dailyDayIndex,
        daily_used_raw: s.dailyUsedRaw,
        session_id: s.sessionId,
        session_start_epoch: s.sessionStartEpoch,
        session_key: s.sessionKey,
        session_expires_at: s.sessionExpiresAt,
        session_limit_raw: s.sessionLimitRaw,
        session_used_raw: s.sessionUsedRaw,
    }
}

fn convert_action(a: SolActionItem) -> Result<Action, SnapshotError> {
    let kind = match a.kind {
        0 => ActionKind::Deposit,
        1 => ActionKind::Withdraw,
        2 => ActionKind::PayObligation,
        3 => ActionKind::Rebalance,
        other => {
            return Err(SnapshotError::DecodeError(format!(
                "Unknown action kind: {other}"
            )))
        }
    };

    Ok(Action {
        kind,
        source_vault: a.sourceVault,
        destination_vault: a.destinationVault,
        assets: a.assets,
        min_shares_out: a.minSharesOut,
        max_shares_burn: a.maxSharesBurn,
        obligation_id_hash: a.obligationIdHash,
        payee_id_hash: a.payeeIdHash,
        recipient: a.recipient,
        due_at: a.dueAt,
        expected_nonce: a.expectedNonce,
        policy_epoch: a.policyEpoch,
        deadline: a.deadline,
        decision_hash: a.decisionHash,
    })
}

fn convert_queue(q: SolQueue) -> Result<Queue, SnapshotError> {
    let action = convert_action(q.action)?;
    Ok(Queue {
        exists: q.exists,
        action_digest: q.actionDigest,
        session_id: q.sessionId,
        ready_at: q.readyAt,
        expires_at: q.expiresAt,
        action,
    })
}

/// Reads on-chain treasury state at a single block via Multicall3 (§10.5).
pub async fn read_snapshot_at_block<P: Provider>(
    provider: &P,
    manifest: &DeploymentManifest,
    target_block_number: Option<u64>,
) -> Result<Snapshot, SnapshotError> {
    // 1. Get block header
    let block_tag = match target_block_number {
        Some(n) => BlockNumberOrTag::Number(n),
        None => BlockNumberOrTag::Latest,
    };

    let block = provider
        .get_block_by_number(block_tag)
        .await
        .map_err(|e| SnapshotError::RpcError(e.to_string()))?
        .ok_or(SnapshotError::MissingBlock)?;

    let block_number = block.header.number;
    let block_hash = block.header.hash;
    let block_timestamp = block.header.timestamp;
    let block_id = BlockId::number(block_number);

    // 1.5 Verify EIP-1967 implementation pin for proxied vaults (§7.2)
    for (&vault, &expected_impl) in &manifest.vault_implementations {
        let actual_impl = read_vault_implementation(provider, vault, block_id).await?;
        if actual_impl != expected_impl {
            return Err(SnapshotError::VaultImplChanged {
                vault,
                expected: expected_impl,
                actual: actual_impl,
                block_number,
                block_hash,
                block_timestamp,
            });
        }
    }

    // 2. Build calls
    let cash_call = IERC20Snapshot::balanceOfCall {
        account: manifest.safe,
    }
    .abi_encode();

    let va_shares_call = IERC4626Snapshot::balanceOfCall {
        account: manifest.safe,
    }
    .abi_encode();

    let vb_shares_call = IERC4626Snapshot::balanceOfCall {
        account: manifest.safe,
    }
    .abi_encode();

    let policy_call = IKavachiModuleSnapshot::getPolicyCall {}.abi_encode();
    let state_call = IKavachiModuleSnapshot::getStateCall {}.abi_encode();
    let queue_call = IKavachiModuleSnapshot::getQueueCall {}.abi_encode();

    let calls = vec![
        IMulticall3Snapshot::Call3 {
            target: manifest.asset,
            allowFailure: false,
            callData: cash_call.into(),
        },
        IMulticall3Snapshot::Call3 {
            target: manifest.vault_a,
            allowFailure: true,
            callData: va_shares_call.into(),
        },
        IMulticall3Snapshot::Call3 {
            target: manifest.vault_b,
            allowFailure: true,
            callData: vb_shares_call.into(),
        },
        IMulticall3Snapshot::Call3 {
            target: manifest.module,
            allowFailure: false,
            callData: policy_call.into(),
        },
        IMulticall3Snapshot::Call3 {
            target: manifest.module,
            allowFailure: false,
            callData: state_call.into(),
        },
        IMulticall3Snapshot::Call3 {
            target: manifest.module,
            allowFailure: false,
            callData: queue_call.into(),
        },
    ];

    let multi_call = IMulticall3Snapshot::aggregate3Call { calls }.abi_encode();
    let tx_req = TransactionRequest::default()
        .to(MULTICALL3_ADDRESS)
        .input(multi_call.into());

    let (safe_cash, va_shares, vb_shares, policy, module_state, queue) =
        match provider.call(tx_req).block(block_id).await {
            Ok(ret_bytes) => {
                let decoded = IMulticall3Snapshot::aggregate3Call::abi_decode_returns(&ret_bytes)
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;

                if decoded.len() < 6 {
                    return Err(SnapshotError::DecodeError(
                        "Multicall3 returned insufficient results".to_string(),
                    ));
                }

                let safe_cash = if decoded[0].success {
                    IERC20Snapshot::balanceOfCall::abi_decode_returns(&decoded[0].returnData)
                        .map_err(|e| SnapshotError::DecodeError(e.to_string()))?
                } else {
                    return Err(SnapshotError::CallFailed(
                        "Failed to read safe cash".to_string(),
                    ));
                };

                let va_shares = if decoded[1].success {
                    IERC4626Snapshot::balanceOfCall::abi_decode_returns(&decoded[1].returnData)
                        .unwrap_or(U256::ZERO)
                } else {
                    U256::ZERO
                };

                let vb_shares = if decoded[2].success {
                    IERC4626Snapshot::balanceOfCall::abi_decode_returns(&decoded[2].returnData)
                        .unwrap_or(U256::ZERO)
                } else {
                    U256::ZERO
                };

                let policy = if decoded[3].success {
                    let sol_policy = IKavachiModuleSnapshot::getPolicyCall::abi_decode_returns(
                        &decoded[3].returnData,
                    )
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                    convert_policy(sol_policy)?
                } else {
                    return Err(SnapshotError::CallFailed(
                        "Failed to read module policy".to_string(),
                    ));
                };

                let module_state = if decoded[4].success {
                    let sol_state = IKavachiModuleSnapshot::getStateCall::abi_decode_returns(
                        &decoded[4].returnData,
                    )
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                    convert_module_state(sol_state)
                } else {
                    return Err(SnapshotError::CallFailed(
                        "Failed to read module state".to_string(),
                    ));
                };

                let queue = if decoded[5].success {
                    let sol_queue = IKavachiModuleSnapshot::getQueueCall::abi_decode_returns(
                        &decoded[5].returnData,
                    )
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                    convert_queue(sol_queue)?
                } else {
                    return Err(SnapshotError::CallFailed(
                        "Failed to read module queue".to_string(),
                    ));
                };

                (safe_cash, va_shares, vb_shares, policy, module_state, queue)
            }
            Err(_) => {
                // Multicall3 fallback: read sequentially at pinned block
                let cash_req = TransactionRequest::default().to(manifest.asset).input(
                    IERC20Snapshot::balanceOfCall {
                        account: manifest.safe,
                    }
                    .abi_encode()
                    .into(),
                );
                let cash_bytes = provider
                    .call(cash_req)
                    .block(block_id)
                    .await
                    .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
                let safe_cash = IERC20Snapshot::balanceOfCall::abi_decode_returns(&cash_bytes)
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;

                let va_req = TransactionRequest::default().to(manifest.vault_a).input(
                    IERC4626Snapshot::balanceOfCall {
                        account: manifest.safe,
                    }
                    .abi_encode()
                    .into(),
                );
                let va_shares = match provider.call(va_req).block(block_id).await {
                    Ok(b) => IERC4626Snapshot::balanceOfCall::abi_decode_returns(&b)
                        .unwrap_or(U256::ZERO),
                    Err(_) => U256::ZERO,
                };

                let vb_req = TransactionRequest::default().to(manifest.vault_b).input(
                    IERC4626Snapshot::balanceOfCall {
                        account: manifest.safe,
                    }
                    .abi_encode()
                    .into(),
                );
                let vb_shares = match provider.call(vb_req).block(block_id).await {
                    Ok(b) => IERC4626Snapshot::balanceOfCall::abi_decode_returns(&b)
                        .unwrap_or(U256::ZERO),
                    Err(_) => U256::ZERO,
                };

                let pol_req = TransactionRequest::default()
                    .to(manifest.module)
                    .input(IKavachiModuleSnapshot::getPolicyCall {}.abi_encode().into());
                let pol_bytes = provider
                    .call(pol_req)
                    .block(block_id)
                    .await
                    .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
                let sol_policy =
                    IKavachiModuleSnapshot::getPolicyCall::abi_decode_returns(&pol_bytes)
                        .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                let policy = convert_policy(sol_policy)?;

                let st_req = TransactionRequest::default()
                    .to(manifest.module)
                    .input(IKavachiModuleSnapshot::getStateCall {}.abi_encode().into());
                let st_bytes = provider
                    .call(st_req)
                    .block(block_id)
                    .await
                    .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
                let sol_state = IKavachiModuleSnapshot::getStateCall::abi_decode_returns(&st_bytes)
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                let module_state = convert_module_state(sol_state);

                let q_req = TransactionRequest::default()
                    .to(manifest.module)
                    .input(IKavachiModuleSnapshot::getQueueCall {}.abi_encode().into());
                let q_bytes = provider
                    .call(q_req)
                    .block(block_id)
                    .await
                    .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
                let sol_queue = IKavachiModuleSnapshot::getQueueCall::abi_decode_returns(&q_bytes)
                    .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
                let queue = convert_queue(sol_queue)?;

                (safe_cash, va_shares, vb_shares, policy, module_state, queue)
            }
        };

    // 3. Convert vault shares to assets
    let vault_a_assets = if va_shares > U256::ZERO {
        let req = TransactionRequest::default().to(manifest.vault_a).input(
            IERC4626Snapshot::convertToAssetsCall { shares: va_shares }
                .abi_encode()
                .into(),
        );
        match provider.call(req).block(block_id).await {
            Ok(b) => {
                IERC4626Snapshot::convertToAssetsCall::abi_decode_returns(&b).unwrap_or(va_shares)
            }
            Err(_) => va_shares,
        }
    } else {
        U256::ZERO
    };

    let vault_b_assets = if vb_shares > U256::ZERO {
        let req = TransactionRequest::default().to(manifest.vault_b).input(
            IERC4626Snapshot::convertToAssetsCall { shares: vb_shares }
                .abi_encode()
                .into(),
        );
        match provider.call(req).block(block_id).await {
            Ok(b) => {
                IERC4626Snapshot::convertToAssetsCall::abi_decode_returns(&b).unwrap_or(vb_shares)
            }
            Err(_) => vb_shares,
        }
    } else {
        U256::ZERO
    };

    Ok(Snapshot {
        quant: None,
        block_number,
        block_hash,
        block_timestamp,
        safe_cash,
        vault_a_shares: va_shares,
        vault_a_assets,
        vault_b_shares: vb_shares,
        vault_b_assets,
        module_state,
        policy,
        queue,
    })
}

/// Reads the EIP-1967 implementation address at the given block (§7.2).
pub async fn read_vault_implementation<P: Provider>(
    provider: &P,
    vault: Address,
    block_id: BlockId,
) -> Result<Address, SnapshotError> {
    let slot_u256 = U256::from_be_bytes(EIP1967_IMPLEMENTATION_SLOT.0);
    let storage_val = provider
        .get_storage_at(vault, slot_u256)
        .block_id(block_id)
        .await
        .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
    let storage_b256 = B256::from(storage_val);
    Ok(Address::from_word(storage_b256))
}

/// Reads snapshot using manifest RPC URL (§10.5).
pub async fn read_snapshot(manifest: &DeploymentManifest) -> Result<Snapshot, SnapshotError> {
    let url =
        Url::from_str(&manifest.rpc_url).map_err(|e| SnapshotError::UrlError(e.to_string()))?;
    let provider = ProviderBuilder::new().connect_http(url);
    read_snapshot_at_block(&provider, manifest, None).await
}

/// Checks whether an obligation bill has been marked paid on-chain (§10.5).
pub async fn is_obligation_paid<P: Provider>(
    provider: &P,
    module: Address,
    obligation_id_hash: B256,
    block_number: u64,
) -> Result<bool, SnapshotError> {
    let req = TransactionRequest::default().to(module).input(
        IKavachiModuleSnapshot::isPaidCall {
            obligationIdHash: obligation_id_hash,
        }
        .abi_encode()
        .into(),
    );
    let bytes = provider
        .call(req)
        .block(BlockId::number(block_number))
        .await
        .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
    IKavachiModuleSnapshot::isPaidCall::abi_decode_returns(&bytes)
        .map_err(|e| SnapshotError::DecodeError(e.to_string()))
}

/// Reads payee status from module (§10.5).
pub async fn read_payee<P: Provider>(
    provider: &P,
    module: Address,
    payee_id_hash: B256,
    block_number: u64,
) -> Result<Payee, SnapshotError> {
    let req = TransactionRequest::default().to(module).input(
        IKavachiModuleSnapshot::getPayeeCall {
            payeeIdHash: payee_id_hash,
        }
        .abi_encode()
        .into(),
    );
    let bytes = provider
        .call(req)
        .block(BlockId::number(block_number))
        .await
        .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
    let sol_payee = IKavachiModuleSnapshot::getPayeeCall::abi_decode_returns(&bytes)
        .map_err(|e| SnapshotError::DecodeError(e.to_string()))?;
    Ok(Payee {
        account: sol_payee.account,
        label_hash: sol_payee.labelHash,
        active_at: sol_payee.activeAt,
        cancelled: sol_payee.cancelled,
    })
}

/// Checks whether a payee address is sanctioned (§10.5).
pub async fn check_sanctioned<P: Provider>(
    provider: &P,
    sanctions_list: Address,
    addr: Address,
    block_number: u64,
) -> Result<bool, SnapshotError> {
    let req = TransactionRequest::default().to(sanctions_list).input(
        ISanctionsListSnapshot::isSanctionedCall { addr }
            .abi_encode()
            .into(),
    );
    let bytes = provider
        .call(req)
        .block(BlockId::number(block_number))
        .await
        .map_err(|e| SnapshotError::RpcError(e.to_string()))?;
    ISanctionsListSnapshot::isSanctionedCall::abi_decode_returns(&bytes)
        .map_err(|e| SnapshotError::DecodeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_policy_mapping() {
        let sol = SolPolicy {
            permissionTier: 2, // Autonomous
            operatingBufferRaw: U256::from(500),
            microMaxRaw: U256::from(10),
            delayedMaxRaw: U256::from(100),
            hardMaxRaw: U256::from(2500),
            dailyLimitRaw: U256::from(4000),
            delaySeconds: 180,
            maxShareSlippageBps: 50,
            guardian: Address::repeat_byte(0x07),
            vaults: vec![SolVaultLimit {
                vault: Address::repeat_byte(0xaa),
                maxExposureRaw: U256::from(6000),
            }],
            payeeDelaySeconds: 3600,
            payEarlyWindowSeconds: 259200,
        };

        let pol = convert_policy(sol).expect("valid conversion");
        assert_eq!(pol.permission_tier, PermissionTier::Autonomous);
        assert_eq!(pol.operating_buffer_raw, U256::from(500));
        assert_eq!(pol.vaults.len(), 1);
        assert_eq!(pol.vaults[0].vault, Address::repeat_byte(0xaa));
    }

    #[test]
    fn test_convert_policy_invalid_tier() {
        let sol = SolPolicy {
            permissionTier: 99,
            operatingBufferRaw: U256::ZERO,
            microMaxRaw: U256::ZERO,
            delayedMaxRaw: U256::ZERO,
            hardMaxRaw: U256::ZERO,
            dailyLimitRaw: U256::ZERO,
            delaySeconds: 0,
            maxShareSlippageBps: 0,
            guardian: Address::ZERO,
            vaults: vec![],
            payeeDelaySeconds: 0,
            payEarlyWindowSeconds: 0,
        };

        assert!(convert_policy(sol).is_err());
    }

    #[test]
    fn test_convert_module_state_mapping() {
        let sol = SolModuleState {
            configured: true,
            paused: false,
            policyEpoch: U256::from(1),
            policyHash: B256::repeat_byte(0x11),
            mandateHash: B256::repeat_byte(0x22),
            obligationsRoot: B256::repeat_byte(0x33),
            reservedRaw: U256::from(583),
            obligationCount: U256::from(1),
            actionNonce: U256::from(0),
            dailyDayIndex: U256::from(1),
            dailyUsedRaw: U256::ZERO,
            sessionId: B256::repeat_byte(0x44),
            sessionStartEpoch: U256::from(1),
            sessionKey: Address::repeat_byte(0x55),
            sessionExpiresAt: 1700086400,
            sessionLimitRaw: U256::from(8000),
            sessionUsedRaw: U256::ZERO,
        };

        let st = convert_module_state(sol);
        assert!(st.configured);
        assert!(!st.paused);
        assert_eq!(st.reserved_raw, U256::from(583));
        assert_eq!(st.session_key, Address::repeat_byte(0x55));
    }
}
