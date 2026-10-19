//! Tests for EIP-1967 off-chain implementation pin (§7.2, A17).

use alloy::primitives::{Address, B256};
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::BlockId;
use alloy::transports::mock::Asserter;
use fusion_core::manifest::DeploymentManifest;
use fusion_core::snapshot::{
    read_snapshot_at_block, read_vault_implementation, SnapshotError, EIP1967_IMPLEMENTATION_SLOT,
};
use std::fs;
use std::path::Path;

#[test]
fn test_eip1967_slot_constant() {
    let expected_slot = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
    assert_eq!(format!("{EIP1967_IMPLEMENTATION_SLOT:#x}"), expected_slot);
}

#[test]
fn test_manifest_parses_deployments_84532() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deployments/84532.json");
    if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path).expect("read 84532.json");
        let manifest = DeploymentManifest::parse_and_validate(&content).expect("parse manifest");
        let expected_wausdc_impl: Address = "0x208747c806b13022db31622c0c9bd3979532a8d6"
            .parse()
            .unwrap();
        assert_eq!(
            manifest.get_vault_implementation(&manifest.vault_a),
            Some(expected_wausdc_impl),
            "vaultA waUSDC implementation must be pinned in manifest"
        );
    }
}

#[tokio::test]
async fn test_read_vault_implementation_matching() {
    let asserter = Asserter::new();
    let expected_impl: Address = "0x208747c806b13022db31622c0c9bd3979532a8d6"
        .parse()
        .unwrap();

    // 32-byte storage slot returns the 20-byte address right-aligned
    let storage_word = format!("{:#066x}", B256::from(expected_impl.into_word()));
    asserter.push_success(&storage_word);

    let provider = ProviderBuilder::new().connect_mocked_client(asserter);
    let vault: Address = "0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a"
        .parse()
        .unwrap();

    let actual = read_vault_implementation(&provider, vault, BlockId::number(100))
        .await
        .expect("read implementation");
    assert_eq!(actual, expected_impl);
}

fn mock_block_json(number: u64, hash: B256, timestamp: u64) -> serde_json::Value {
    let mut block = alloy::rpc::types::eth::Block::<alloy::rpc::types::eth::Transaction>::default();
    block.header.number = number;
    block.header.hash = hash;
    block.header.timestamp = timestamp;
    serde_json::to_value(&block).unwrap()
}

#[tokio::test]
async fn test_read_snapshot_at_block_mismatch_raises_vault_impl_changed() {
    let asserter = Asserter::new();
    let expected_impl: Address = "0x208747c806b13022db31622c0c9bd3979532a8d6"
        .parse()
        .unwrap();
    let swapped_impl: Address = "0x3333333333333333333333333333333333333333"
        .parse()
        .unwrap();

    let block_hash = B256::repeat_byte(0x99);
    // 1. eth_getBlockByNumber
    asserter.push_success(&mock_block_json(100, block_hash, 0x65000000));

    // 2. eth_getStorageAt for vault_a: returns the swapped implementation!
    let swapped_storage_word = format!("{:#066x}", B256::from(swapped_impl.into_word()));
    asserter.push_success(&swapped_storage_word);

    let provider = ProviderBuilder::new().connect_mocked_client(asserter);

    let vault_a: Address = "0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a"
        .parse()
        .unwrap();
    let json = format!(
        r#"{{
        "profile": "FULL",
        "chainId": 84532,
        "rpcUrl": "https://sepolia.base.org",
        "safe": "0x000000000000000000000000000000000000dEaD",
        "module": "0x00000000000000000000000000000000000000B0",
        "asset": "0x0000000000000000000000000000000000000001",
        "vaultA": "{vault_a}",
        "vaultB": "0x00000000000000000000000000000000000000b2",
        "sanctionsList": "0x0000000000000000000000000000000000000002",
        "vaultImplementations": {{
            "{vault_a}": "{expected_impl}"
        }},
        "deploymentBlock": 12345,
        "explorerBaseUrl": "https://sepolia.basescan.org"
    }}"#
    );

    let manifest = DeploymentManifest::parse_and_validate(&json).unwrap();
    let res = read_snapshot_at_block(&provider, &manifest, Some(100)).await;

    match res {
        Err(SnapshotError::VaultImplChanged {
            vault,
            expected,
            actual,
            block_number,
            ..
        }) => {
            assert_eq!(vault, vault_a);
            assert_eq!(expected, expected_impl);
            assert_eq!(actual, swapped_impl);
            assert_eq!(block_number, 100);
        }
        other => panic!("Expected SnapshotError::VaultImplChanged, got {other:?}"),
    }
}
