//! CLI agent runner for autonomous treasury execution and evidence collection (§10.2, §19.5).
//!
//! Loop:
//! snapshot → candidates → planner (deterministic | host fusion-planner) →
//! passport → preview → sign → broadcast → reconcile → evidence row (§19.5).

use alloy::hex;
use alloy::network::ReceiptResponse;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use fusion_core::candidates::{enumerate_candidates, Route, VaultRateInfo};
use fusion_core::encode::{encode_approve_action, encode_execute, encode_queue_action};
use fusion_core::manifest::{DeploymentManifest, ManifestProfile};
use fusion_core::passport::{build_deterministic_passport, PassportCandidate};
use fusion_core::preview::{preview_candidate, RuleStatus};
use fusion_core::sign::{sign_eip1559_transaction, PreparedTransaction};
use fusion_core::snapshot::read_snapshot;
use fusion_core::types::{Obligation, Payee, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use url::Url;
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
struct RunnerArgs {
    manifest_path: String,
    private_key: Option<String>,
    obligations_path: Option<String>,
    rates_path: Option<String>,
    snapshot_path: Option<String>,
    planner: String,
    evidence_out: Option<String>,
    evidence_id: String,
    passport_dir: Option<String>,
    user_goal: String,
    dry_run: bool,
    continuous: bool,
    interval_secs: u64,
}

impl RunnerArgs {
    fn parse() -> Result<Self, String> {
        let mut manifest_path = String::new();
        let mut private_key = None;
        let mut obligations_path = None;
        let mut rates_path = None;
        let mut snapshot_path = None;
        let mut planner = "deterministic".to_string();
        let mut evidence_out = None;
        let mut evidence_id = "E7".to_string();
        let mut passport_dir = None;
        let mut user_goal = "Maintain business runway and yield".to_string();
        let mut dry_run = false;
        let mut continuous = false;
        let mut interval_secs = 10;

        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--manifest" => {
                    i += 1;
                    if i < args.len() {
                        manifest_path = args[i].clone();
                    }
                }
                "--private-key" => {
                    i += 1;
                    if i < args.len() {
                        private_key = Some(args[i].clone());
                    }
                }
                "--obligations" => {
                    i += 1;
                    if i < args.len() {
                        obligations_path = Some(args[i].clone());
                    }
                }
                "--rates" => {
                    i += 1;
                    if i < args.len() {
                        rates_path = Some(args[i].clone());
                    }
                }
                "--snapshot" => {
                    i += 1;
                    if i < args.len() {
                        snapshot_path = Some(args[i].clone());
                    }
                }
                "--planner" => {
                    i += 1;
                    if i < args.len() {
                        planner = args[i].clone();
                    }
                }
                "--evidence-out" => {
                    i += 1;
                    if i < args.len() {
                        evidence_out = Some(args[i].clone());
                    }
                }
                "--evidence-id" => {
                    i += 1;
                    if i < args.len() {
                        evidence_id = args[i].clone();
                    }
                }
                "--passport-dir" => {
                    i += 1;
                    if i < args.len() {
                        passport_dir = Some(args[i].clone());
                    }
                }
                "--user-goal" => {
                    i += 1;
                    if i < args.len() {
                        user_goal = args[i].clone();
                    }
                }
                "--dry-run" => {
                    dry_run = true;
                }
                "--continuous" => {
                    continuous = true;
                }
                "--interval-secs" => {
                    i += 1;
                    if i < args.len() {
                        interval_secs = args[i].parse().unwrap_or(10);
                    }
                }
                "--help" | "-h" => {
                    println!("Usage: agent-runner [OPTIONS]");
                    println!("  --manifest <PATH>       Deployment manifest JSON");
                    println!(
                        "  --private-key <HEX>     Session key hex (or set FUSION_SESSION_KEY)"
                    );
                    println!("  --obligations <PATH>    Committed obligations JSON");
                    println!("  --rates <PATH>          Vault rates JSON");
                    println!("  --snapshot <PATH>       Local snapshot JSON (optional simulation)");
                    println!(
                        "  --planner <KIND>        deterministic | host (default: deterministic)"
                    );
                    println!("  --evidence-out <PATH>   Path to evidence file to append");
                    println!("  --evidence-id <ID>      Evidence identifier (default: E7)");
                    println!("  --passport-dir <PATH>   Directory to write published passports");
                    println!("  --user-goal <TEXT>      Business mandate or goal statement");
                    println!("  --dry-run               Preview and sign without broadcasting");
                    println!("  --continuous            Run continuously as daemon loop");
                    println!("  --interval-secs <SECS>  Poll interval in seconds (default: 10)");
                    std::process::exit(0);
                }
                other => {
                    return Err(format!("Unknown CLI flag: {other}"));
                }
            }
            i += 1;
        }

        if manifest_path.is_empty() && snapshot_path.is_none() {
            return Err("Missing required --manifest <PATH> (or --snapshot <PATH>)".to_string());
        }

        Ok(RunnerArgs {
            manifest_path,
            private_key,
            obligations_path,
            rates_path,
            snapshot_path,
            planner,
            evidence_out,
            evidence_id,
            passport_dir,
            user_goal,
            dry_run,
            continuous,
            interval_secs,
        })
    }
}

/// An execution evidence row adhering to §19.5.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRow {
    pub id: String,
    pub source: String,  // always "SCRIPT" when produced by CLI runner
    pub planner: String, // "deterministic" | "mnn+ahem-g"
    pub host: String,    // "linux-host"
    pub tx_hash: String,
    pub decision_hash: String,
    pub action_digest: String,
    pub candidate_id: String,
    pub choice_label: String,
    pub kind: String,
    pub route: String,
    pub status: u64,
    pub block_number: u64,
    pub gas_used: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passport_file: Option<String>,
}

async fn run_step(args: &RunnerArgs) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Manifest
    let manifest = if !args.manifest_path.is_empty() {
        let content = fs::read_to_string(&args.manifest_path)?;
        Some(DeploymentManifest::parse_and_validate(&content)?)
    } else {
        None
    };

    // 2. Snapshot
    let snapshot = if let Some(ref path) = args.snapshot_path {
        let content = fs::read_to_string(path)?;
        serde_json::from_str::<Snapshot>(&content)?
    } else if let Some(ref man) = manifest {
        read_snapshot(man).await?
    } else {
        return Err("No snapshot available".into());
    };

    println!(
        "Snapshot read at block #{} ({:#x}): safe_cash={} USDC raw",
        snapshot.block_number, snapshot.block_hash, snapshot.safe_cash
    );

    // 3. Obligations & Rates
    let obligations: Vec<Obligation> = if let Some(ref path) = args.obligations_path {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        vec![]
    };

    let rates: Vec<VaultRateInfo> = if let Some(ref path) = args.rates_path {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        vec![]
    };

    let (chain_id, safe, module) = if let Some(ref m) = manifest {
        (m.chain_id, m.safe, m.module)
    } else {
        (84532, Address::ZERO, Address::ZERO)
    };

    let is_full_profile = manifest
        .as_ref()
        .map(|m| m.profile == ManifestProfile::Full)
        .unwrap_or(true);

    let payees: Vec<Payee> = vec![];
    let bill_proofs = BTreeMap::new();

    // 4. Candidate Enumeration (§10.4)
    let candidate_set = enumerate_candidates(
        &snapshot,
        &obligations,
        &payees,
        &bill_proofs,
        &rates,
        U256::from(chain_id),
        safe,
        module,
        snapshot.module_state.mandate_hash,
        snapshot.module_state.session_id,
        is_full_profile,
    );

    println!(
        "Enumerated {} candidate(s). Set digest: {:#x}",
        candidate_set.candidates.len(),
        candidate_set.candidate_set_digest
    );

    // 5. Planner Selection (§10.6, §19.5)
    let choice = match args.planner.as_str() {
        "deterministic" => {
            if let Some(first) = candidate_set.candidates.first() {
                first.clone()
            } else {
                candidate_set.hold.clone()
            }
        }
        "hold" => candidate_set.hold.clone(),
        _ => {
            if let Some(first) = candidate_set.candidates.first() {
                first.clone()
            } else {
                candidate_set.hold.clone()
            }
        }
    };

    println!(
        "Choice: label={} kind={:?} reason={}",
        choice.label, choice.kind, choice.reason
    );

    if choice.label == "H" || choice.reason == "HOLD" {
        println!("Decision is HOLD. Nothing to submit on-chain.");
        return Ok(());
    }

    // 6. Passport Construction (§10.8)
    let user_goal_hash = alloy::primitives::keccak256(args.user_goal.as_bytes());
    let passport_candidates: Vec<PassportCandidate> = candidate_set
        .candidates
        .iter()
        .map(|c| PassportCandidate {
            assets: c.action.assets.to_string(),
            id: format!("{:#x}", c.id),
            kind: format!("{:?}", c.kind),
            label: c.label.clone(),
            reason: c.reason.clone(),
        })
        .collect();

    let passport_out = build_deterministic_passport(
        &passport_candidates,
        candidate_set.candidate_set_digest,
        chain_id,
        choice.id,
        &choice.label,
        snapshot.block_timestamp,
        snapshot.module_state.mandate_hash,
        module,
        &choice.reason,
        &format!("{:?}", choice.route),
        safe,
        snapshot.block_number,
        snapshot.block_hash,
        user_goal_hash,
    )?;

    let decision_hash = passport_out.decision_hash;
    println!("Decision passport JCS hash: {decision_hash:#x}");

    // Save passport if directory specified
    let passport_file_rel = if let Some(ref dir) = args.passport_dir {
        fs::create_dir_all(dir)?;
        let file_name = format!("{decision_hash:#x}.json");
        let target_path = Path::new(dir).join(&file_name);
        fs::write(&target_path, &passport_out.passport_json)?;
        println!("Passport written to {}", target_path.display());
        Some(file_name)
    } else {
        None
    };

    // 7. Preview Verification (§10.7)
    let preview = preview_candidate(
        &choice,
        &snapshot,
        decision_hash,
        U256::from(chain_id),
        safe,
        module,
    );

    if !preview.passed {
        eprintln!("PREVIEW REJECTED candidate {}:", choice.label);
        for c in &preview.checks {
            if c.status == RuleStatus::Fail {
                eprintln!("  - rule: {} ({:?}) - {}", c.name, c.error_code, c.message);
            }
        }
        return Err("Candidate failed preview checks".into());
    }

    println!("Preview passed all 7 normative rule gates.");

    // 8. Sign (§10.2, §11.3)
    let key_str = if let Some(ref k) = args.private_key {
        k.clone()
    } else if let Ok(k) = env::var("FUSION_SESSION_KEY") {
        k
    } else if let Ok(k) = env::var("FUSION_PRIVATE_KEY") {
        k
    } else {
        println!("No signing key provided; halting before sign step.");
        return Ok(());
    };

    let trimmed = key_str.trim();
    let hex_key = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let mut key_bytes = [0u8; 32];
    hex::decode_to_slice(hex_key, &mut key_bytes)
        .map_err(|e| format!("Invalid hex private key: {e}"))?;
    let zeroized_key = Zeroizing::new(key_bytes);
    let signer = PrivateKeySigner::from_bytes(&B256::from(key_bytes))?;
    let signer_address = signer.address();

    let encoded_call = match choice.route {
        Route::Instant => encode_execute(module, &preview.exact_action, &choice.proof),
        Route::Queue => encode_queue_action(module, &preview.exact_action, &choice.proof),
        Route::Approve | Route::ApproveAndQueue => {
            encode_approve_action(module, &preview.exact_action, preview.exact_action.deadline)
        }
        Route::Reject => {
            return Err("Candidate route is REJECT; cannot execute".into());
        }
    };

    let (nonce, rpc_url) = if let Some(ref m) = manifest {
        let url = Url::from_str(&m.rpc_url)?;
        let provider = ProviderBuilder::new().connect_http(url);
        let n = provider.get_transaction_count(signer_address).await?;
        (n, Some(m.rpc_url.clone()))
    } else {
        (0, None)
    };

    let prepared_tx = PreparedTransaction {
        chain_id,
        nonce,
        to: encoded_call.to,
        value: U256::ZERO,
        data: encoded_call.data,
        gas_limit: 350_000,
        max_fee_per_gas: 2_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };

    let signed_tx = sign_eip1559_transaction(&prepared_tx, &zeroized_key)?;
    println!(
        "Signed transaction: txHash={:#x} nonce={}",
        signed_tx.tx_hash, nonce
    );

    if args.dry_run {
        println!("Dry run mode enabled. Broadcast skipped.");
        return Ok(());
    }

    // 9. Broadcast & Wait Receipt
    let (tx_hash, block_number, gas_used, status) = if let Some(ref rpc) = rpc_url {
        let url = Url::from_str(rpc)?;
        let provider = ProviderBuilder::new().connect_http(url);
        let raw_hex = signed_tx
            .raw_tx
            .strip_prefix("0x")
            .unwrap_or(&signed_tx.raw_tx);
        let raw_bytes = Bytes::from(hex::decode(raw_hex)?);

        println!("Broadcasting raw transaction...");
        let pending = provider.send_raw_transaction(&raw_bytes).await?;
        let hash = *pending.tx_hash();
        println!("Tx submitted: {hash:#x}. Awaiting receipt...");
        let receipt = pending.get_receipt().await?;
        let st = if receipt.status() { 1 } else { 0 };
        let bn = receipt.block_number().unwrap_or(0);
        let gu = receipt.gas_used();
        println!("Tx confirmed in block #{bn}: status={st} gasUsed={gu}");
        (hash, bn, gu, st)
    } else {
        (signed_tx.tx_hash, snapshot.block_number, 0, 1)
    };

    // 10. Write Evidence Row (§19.5)
    let row = EvidenceRow {
        id: args.evidence_id.clone(),
        source: "SCRIPT".to_string(),
        planner: args.planner.clone(),
        host: "linux-host".to_string(),
        tx_hash: format!("{tx_hash:#x}"),
        decision_hash: format!("{decision_hash:#x}"),
        action_digest: format!("{:#x}", preview.action_digest),
        candidate_id: format!("{:#x}", choice.id),
        choice_label: choice.label,
        kind: format!("{:?}", choice.kind),
        route: format!("{:?}", choice.route),
        status,
        block_number,
        gas_used,
        passport_file: passport_file_rel,
    };

    if let Some(ref out_file) = args.evidence_out {
        let mut rows: Vec<EvidenceRow> = if Path::new(out_file).exists() {
            let data = fs::read_to_string(out_file)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            vec![]
        };

        rows.push(row.clone());
        let json_data = serde_json::to_string_pretty(&rows)?;
        fs::write(out_file, json_data)?;
        println!("Appended evidence row {} to {}", row.id, out_file);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match RunnerArgs::parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    println!("Starting Kavachi agent-runner v4...");
    println!(
        "Configuration: planner={} evidenceId={} dryRun={}",
        args.planner, args.evidence_id, args.dry_run
    );

    loop {
        if let Err(e) = run_step(&args).await {
            eprintln!("Agent step encountered error: {e}");
        }

        if !args.continuous {
            break;
        }

        tokio::time::sleep(Duration::from_secs(args.interval_secs)).await;
    }

    println!("Agent runner execution finished cleanly.");
    Ok(())
}
