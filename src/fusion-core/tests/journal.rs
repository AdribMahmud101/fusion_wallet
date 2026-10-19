//! Tests for durable journal and startup reconciliation (§13, A13).

use alloy::primitives::{Address, B256};
use alloy::providers::ProviderBuilder;
use alloy::transports::mock::Asserter;
use fusion_core::journal::{reconcile_unknown_entries, DurableJournal, JournalStatus};
use tempfile::NamedTempFile;

#[test]
fn test_journal_append_and_fsync() {
    let tmp = NamedTempFile::new().unwrap();
    let journal = DurableJournal::new(tmp.path());

    let digest = B256::repeat_byte(0xaa);
    let tx_hash = B256::repeat_byte(0x11);
    let nonce = 3;

    // 1. Record broadcast requested (UNKNOWN)
    journal
        .record_broadcast_requested(digest, nonce, tx_hash, None, 1_700_000_000)
        .expect("record broadcast");

    let entries = journal.read_all().expect("read entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].status, JournalStatus::Unknown);
    assert_eq!(entries[0].action_digest, digest);
    assert_eq!(entries[0].nonce, nonce);
    assert_eq!(entries[0].raw_tx_hash, tx_hash);

    assert!(journal.in_flight_digests().unwrap().contains(&digest));
    assert!(!journal.completed_digests().unwrap().contains(&digest));

    // 2. Record confirmation
    journal
        .record_confirmed(digest, nonce, tx_hash, 150, 65000, 1_700_000_010)
        .expect("record confirmed");

    let all = journal.read_all().expect("read all");
    assert_eq!(all.len(), 2, "append-only log preserves history");

    let latest = journal.latest_entries().expect("latest");
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].status, JournalStatus::Confirmed);
    assert_eq!(latest[0].block_number, Some(150));
    assert_eq!(latest[0].gas_used, Some(65000));

    assert!(!journal.in_flight_digests().unwrap().contains(&digest));
    assert!(journal.completed_digests().unwrap().contains(&digest));
}

#[tokio::test]
async fn test_reconcile_mined_unknown_entry_on_restart() {
    let tmp = NamedTempFile::new().unwrap();
    let journal = DurableJournal::new(tmp.path());

    let digest = B256::repeat_byte(0xbb);
    let tx_hash = B256::repeat_byte(0x22);
    let nonce = 7;

    // Crash simulation: runner wrote UNKNOWN to disk before broadcast and crashed before receipt
    journal
        .record_broadcast_requested(digest, nonce, tx_hash, None, 1_700_000_000)
        .expect("record broadcast");

    // On restart: mock chain returns mined receipt
    let asserter = Asserter::new();

    // 1. eth_getTransactionCount
    asserter.push_success(&"0x8");

    // 2. eth_getTransactionReceipt returns success status=1
    // 2. eth_getTransactionReceipt returns success status=1
    asserter.push_success(&serde_json::json!({
        "transactionHash": format!("{tx_hash:#x}"),
        "transactionIndex": "0x0",
        "blockHash": format!("{:#x}", B256::repeat_byte(0x99)),
        "blockNumber": "0xc8",
        "from": "0x5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E",
        "to": "0x1111111111111111111111111111111111111111",
        "cumulativeGasUsed": "0x13880",
        "gasUsed": "0x13880",
        "effectiveGasPrice": "0x1",
        "status": "0x1",
        "logs": [],
        "logsBloom": format!("0x{:0512x}", 0),
        "type": "0x2"
    }));

    let provider = ProviderBuilder::new().connect_mocked_client(asserter);
    let signer: Address = "0x5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E"
        .parse()
        .unwrap();

    let summary = reconcile_unknown_entries(&journal, &provider, signer, 1_700_000_030)
        .await
        .expect("reconcile");

    assert_eq!(summary.reconciled_mined, 1);
    assert_eq!(summary.reconciled_reverted, 0);
    assert_eq!(summary.still_in_flight, 0);

    // Completed digests now contains the digest; preventing double-execution!
    let completed = journal.completed_digests().expect("completed");
    assert!(
        completed.contains(&digest),
        "Must be marked completed after reconciliation"
    );
    assert!(journal.in_flight_digests().expect("in flight").is_empty());
}

#[tokio::test]
async fn test_reconcile_reverted_unknown_entry_on_restart() {
    let tmp = NamedTempFile::new().unwrap();
    let journal = DurableJournal::new(tmp.path());

    let digest = B256::repeat_byte(0xcc);
    let tx_hash = B256::repeat_byte(0x33);
    let nonce = 2;

    journal
        .record_broadcast_requested(digest, nonce, tx_hash, None, 1_700_000_000)
        .expect("record broadcast");

    let asserter = Asserter::new();
    // 1. eth_getTransactionCount
    asserter.push_success(&"0x3");

    // 2. eth_getTransactionReceipt returns failure status=0
    asserter.push_success(&serde_json::json!({
        "transactionHash": format!("{tx_hash:#x}"),
        "transactionIndex": "0x0",
        "blockHash": format!("{:#x}", B256::repeat_byte(0x99)),
        "blockNumber": "0xd0",
        "from": "0x5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E",
        "to": "0x1111111111111111111111111111111111111111",
        "cumulativeGasUsed": "0x10000",
        "gasUsed": "0x10000",
        "effectiveGasPrice": "0x1",
        "status": "0x0",
        "logs": [],
        "logsBloom": format!("0x{:0512x}", 0),
        "type": "0x2"
    }));

    let provider = ProviderBuilder::new().connect_mocked_client(asserter);
    let signer: Address = "0x5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E"
        .parse()
        .unwrap();

    let summary = reconcile_unknown_entries(&journal, &provider, signer, 1_700_000_030)
        .await
        .expect("reconcile");

    assert_eq!(summary.reconciled_mined, 0);
    assert_eq!(summary.reconciled_reverted, 1);
    assert_eq!(summary.still_in_flight, 0);

    let latest = journal.latest_entries().unwrap();
    assert_eq!(latest[0].status, JournalStatus::Reverted);
}

#[tokio::test]
async fn test_reconcile_still_in_flight_when_no_receipt_and_nonce_equal() {
    let tmp = NamedTempFile::new().unwrap();
    let journal = DurableJournal::new(tmp.path());

    let digest = B256::repeat_byte(0xdd);
    let tx_hash = B256::repeat_byte(0x44);
    let nonce = 10;

    journal
        .record_broadcast_requested(digest, nonce, tx_hash, None, 1_700_000_000)
        .expect("record broadcast");

    let asserter = Asserter::new();
    // 1. eth_getTransactionCount: account nonce is still 10 (tx may still be pending in mempool)
    asserter.push_success(&"0xa");

    // 2. eth_getTransactionReceipt returns null (not yet mined)
    asserter.push_success(&serde_json::Value::Null);

    let provider = ProviderBuilder::new().connect_mocked_client(asserter);
    let signer: Address = "0x5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E5E"
        .parse()
        .unwrap();

    let summary = reconcile_unknown_entries(&journal, &provider, signer, 1_700_000_030)
        .await
        .expect("reconcile");

    assert_eq!(summary.still_in_flight, 1);
    assert_eq!(summary.reconciled_mined, 0);
    assert_eq!(summary.reconciled_reverted, 0);

    // Remains in flight so the runner refuses to re-sign or double-send
    assert!(journal.in_flight_digests().unwrap().contains(&digest));
}
