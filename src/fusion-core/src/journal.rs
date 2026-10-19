//! Durable append-only journal for in-flight transaction tracking and crash recovery (§13).
//!
//! Written BEFORE network broadcast with (digest, nonce, raw tx hash) and fsynced.
//! Reconciled against chain receipts on startup before any new decisions to prevent
//! duplicate execution or double-spending (§13).

use alloy::network::ReceiptResponse;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Status of an execution attempt in the durable journal (§13.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JournalStatus {
    Unknown,
    Mined,
    Confirmed,
    Reconciled,
    Reverted,
}

/// A single journal entry recorded in JSON Lines format (§13).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub timestamp: u64,
    pub action_digest: B256,
    pub nonce: u64,
    pub raw_tx_hash: B256,
    pub status: JournalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_hash: Option<B256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Durable append-only file journal (§13).
#[derive(Clone, Debug)]
pub struct DurableJournal {
    path: PathBuf,
}

impl DurableJournal {
    /// Creates a new journal referencing the given file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Path to the journal file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends an entry to the journal and immediately calls fsync (§13).
    pub fn append(&self, entry: &JournalEntry) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let json_line = serde_json::to_string(entry)?;
        writeln!(file, "{json_line}")?;
        file.sync_all()?;
        Ok(())
    }

    /// Reads all entries from the journal file in append order.
    pub fn read_all(&self) -> Result<Vec<JournalEntry>, std::io::Error> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<JournalEntry>(trimmed) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    /// Returns the latest state for each transaction hash.
    pub fn latest_entries(&self) -> Result<Vec<JournalEntry>, std::io::Error> {
        let entries = self.read_all()?;
        let mut by_tx: BTreeMap<B256, JournalEntry> = BTreeMap::new();
        for e in entries {
            by_tx.insert(e.raw_tx_hash, e);
        }
        Ok(by_tx.into_values().collect())
    }

    /// Returns all digests that are currently UNKNOWN (in-flight, awaiting receipt).
    pub fn in_flight_digests(&self) -> Result<BTreeSet<B256>, std::io::Error> {
        let latest = self.latest_entries()?;
        let mut in_flight = BTreeSet::new();
        for e in latest {
            if e.status == JournalStatus::Unknown {
                in_flight.insert(e.action_digest);
            }
        }
        Ok(in_flight)
    }

    /// Returns all digests that have been confirmed or reconciled as executed.
    pub fn completed_digests(&self) -> Result<BTreeSet<B256>, std::io::Error> {
        let latest = self.latest_entries()?;
        let mut completed = BTreeSet::new();
        for e in latest {
            if matches!(
                e.status,
                JournalStatus::Mined | JournalStatus::Confirmed | JournalStatus::Reconciled
            ) {
                completed.insert(e.action_digest);
            }
        }
        Ok(completed)
    }

    /// Returns all entries that are currently UNKNOWN.
    pub fn unknown_entries(&self) -> Result<Vec<JournalEntry>, std::io::Error> {
        let latest = self.latest_entries()?;
        Ok(latest
            .into_iter()
            .filter(|e| e.status == JournalStatus::Unknown)
            .collect())
    }

    /// Records broadcast of a transaction: writes UNKNOWN entry and fsyncs BEFORE network send (§13).
    pub fn record_broadcast_requested(
        &self,
        action_digest: B256,
        nonce: u64,
        raw_tx_hash: B256,
        decision_hash: Option<B256>,
        timestamp: u64,
    ) -> Result<(), std::io::Error> {
        let entry = JournalEntry {
            timestamp,
            action_digest,
            nonce,
            raw_tx_hash,
            status: JournalStatus::Unknown,
            decision_hash,
            block_number: None,
            gas_used: None,
            reason: None,
        };
        self.append(&entry)
    }

    /// Records successful receipt/confirmation of a transaction (§13).
    pub fn record_confirmed(
        &self,
        action_digest: B256,
        nonce: u64,
        raw_tx_hash: B256,
        block_number: u64,
        gas_used: u64,
        timestamp: u64,
    ) -> Result<(), std::io::Error> {
        let entry = JournalEntry {
            timestamp,
            action_digest,
            nonce,
            raw_tx_hash,
            status: JournalStatus::Confirmed,
            decision_hash: None,
            block_number: Some(block_number),
            gas_used: Some(gas_used),
            reason: None,
        };
        self.append(&entry)
    }

    /// Records reconciliation of an UNKNOWN transaction on restart (§13).
    pub fn record_reconciled(
        &self,
        action_digest: B256,
        nonce: u64,
        raw_tx_hash: B256,
        block_number: u64,
        gas_used: u64,
        timestamp: u64,
    ) -> Result<(), std::io::Error> {
        let entry = JournalEntry {
            timestamp,
            action_digest,
            nonce,
            raw_tx_hash,
            status: JournalStatus::Reconciled,
            decision_hash: None,
            block_number: Some(block_number),
            gas_used: Some(gas_used),
            reason: Some("Reconciled on startup".to_string()),
        };
        self.append(&entry)
    }

    /// Records a reverted transaction (§13).
    pub fn record_reverted(
        &self,
        action_digest: B256,
        nonce: u64,
        raw_tx_hash: B256,
        block_number: u64,
        timestamp: u64,
        reason: Option<String>,
    ) -> Result<(), std::io::Error> {
        let entry = JournalEntry {
            timestamp,
            action_digest,
            nonce,
            raw_tx_hash,
            status: JournalStatus::Reverted,
            decision_hash: None,
            block_number: Some(block_number),
            gas_used: None,
            reason,
        };
        self.append(&entry)
    }
}

/// Summary of a startup journal reconciliation run (§13).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationSummary {
    pub reconciled_mined: usize,
    pub reconciled_reverted: usize,
    pub still_in_flight: usize,
}

/// Reconciles every UNKNOWN entry in the journal against the chain on startup (§13).
pub async fn reconcile_unknown_entries<P: Provider>(
    journal: &DurableJournal,
    provider: &P,
    signer_address: Address,
    now: u64,
) -> Result<ReconciliationSummary, Box<dyn std::error::Error>> {
    let unknowns = journal.unknown_entries()?;
    if unknowns.is_empty() {
        return Ok(ReconciliationSummary::default());
    }

    let mut summary = ReconciliationSummary::default();
    let onchain_account_nonce = provider.get_transaction_count(signer_address).await?;

    for entry in unknowns {
        // Query receipt by hash
        match provider.get_transaction_receipt(entry.raw_tx_hash).await {
            Ok(Some(receipt)) => {
                let st = receipt.status();
                let bn = receipt.block_number().unwrap_or(0);
                let gu = receipt.gas_used();
                if st {
                    println!(
                        "Reconciliation: UNKNOWN tx {:#x} (digest {:#x}) was MINED at block #{bn} (status=1). Marking RECONCILED.",
                        entry.raw_tx_hash, entry.action_digest
                    );
                    journal.record_reconciled(
                        entry.action_digest,
                        entry.nonce,
                        entry.raw_tx_hash,
                        bn,
                        gu,
                        now,
                    )?;
                    summary.reconciled_mined += 1;
                } else {
                    println!(
                        "Reconciliation: UNKNOWN tx {:#x} (digest {:#x}) REVERTED at block #{bn}. Marking REVERTED.",
                        entry.raw_tx_hash, entry.action_digest
                    );
                    journal.record_reverted(
                        entry.action_digest,
                        entry.nonce,
                        entry.raw_tx_hash,
                        bn,
                        now,
                        Some("Tx reverted on-chain".to_string()),
                    )?;
                    summary.reconciled_reverted += 1;
                }
            }
            Ok(None) => {
                // Receipt not yet available. Check if on-chain nonce moved past entry.nonce:
                if onchain_account_nonce > entry.nonce {
                    println!(
                        "Reconciliation: tx {:#x} receipt missing and on-chain nonce {} > entry nonce {}. Marking REVERTED.",
                        entry.raw_tx_hash, onchain_account_nonce, entry.nonce
                    );
                    journal.record_reverted(
                        entry.action_digest,
                        entry.nonce,
                        entry.raw_tx_hash,
                        0,
                        now,
                        Some("Nonce superseded without receipt".to_string()),
                    )?;
                    summary.reconciled_reverted += 1;
                } else {
                    println!(
                        "Reconciliation: tx {:#x} (digest {:#x}) is STILL IN FLIGHT (account nonce {} <= entry nonce {}). Will NOT re-sign.",
                        entry.raw_tx_hash, entry.action_digest, onchain_account_nonce, entry.nonce
                    );
                    summary.still_in_flight += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to query receipt for tx {:#x}: {e}",
                    entry.raw_tx_hash
                );
            }
        }
    }

    Ok(summary)
}
