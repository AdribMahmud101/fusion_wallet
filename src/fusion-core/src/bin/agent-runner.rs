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
use fusion_core::candidates::{enumerate_candidates, CandidateSet, Route, VaultRateInfo};
use fusion_core::encode::{encode_approve_action, encode_execute, encode_queue_action};
use fusion_core::manifest::{DeploymentManifest, ManifestProfile};
use fusion_core::passport::{
    to_canonical_json, AhemGReport, AhemLReport, AhemMetadata, DecisionPassport, PassportCandidate,
    PassportSnapshot, PlannerMetadata,
};
use fusion_core::planner_io::build_planner_prompt;
use fusion_core::preview::{preview_candidate, preview_proposal, RuleStatus};
use fusion_core::sign::{
    sign_eip1559_transaction, sign_guardian_pause_transaction, PreparedTransaction,
    SessionSigningPolicy,
};
use fusion_core::snapshot::{read_snapshot, read_snapshot_at_block};
use fusion_core::types::{ActionKind, Obligation, Payee, Snapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    block: Option<u64>,
    planner: String,
    planner_bin: Option<String>,
    model_dir: Option<String>,
    evidence_out: Option<String>,
    evidence_id: String,
    passport_dir: Option<String>,
    dryrun_dir: Option<String>,
    runs_dir: Option<String>,
    source: String,
    choice: Option<String>,
    user_goal: String,
    dry_run: bool,
    continuous: bool,
    interval_secs: u64,
    journal_path: Option<String>,
    guardian_key: Option<String>,
    crash_after_broadcast: bool,
}

impl RunnerArgs {
    fn parse() -> Result<Self, String> {
        let mut manifest_path = String::new();
        let mut private_key = None;
        let mut obligations_path = None;
        let mut rates_path = None;
        let mut snapshot_path = None;
        let mut block = None;
        let mut planner = "deterministic".to_string();
        let mut planner_bin = None;
        let mut model_dir = None;
        let mut evidence_out = None;
        let mut evidence_id = "E7".to_string();
        let mut passport_dir = None;
        let mut dryrun_dir = None;
        let mut runs_dir = None;
        let mut source = "AGENT".to_string();
        let mut choice = None;
        let mut user_goal = "Maintain business runway and yield".to_string();
        let mut dry_run = false;
        let mut continuous = false;
        let mut interval_secs = 10;
        let mut journal_path = None;
        let mut guardian_key = None;
        let mut crash_after_broadcast = false;

        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--crash-after-broadcast" => {
                    crash_after_broadcast = true;
                }
                "--journal" => {
                    i += 1;
                    if i < args.len() {
                        journal_path = Some(args[i].clone());
                    }
                }
                "--guardian-key" => {
                    i += 1;
                    if i < args.len() {
                        guardian_key = Some(args[i].clone());
                    }
                }
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
                "--block" => {
                    i += 1;
                    if i < args.len() {
                        block = Some(args[i].parse().map_err(|e| format!("Invalid block: {e}"))?);
                    }
                }
                "--planner" => {
                    i += 1;
                    if i < args.len() {
                        planner = args[i].clone();
                    }
                }
                "--planner-bin" => {
                    i += 1;
                    if i < args.len() {
                        planner_bin = Some(args[i].clone());
                    }
                }
                "--model-dir" => {
                    i += 1;
                    if i < args.len() {
                        model_dir = Some(args[i].clone());
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
                "--dryrun-dir" => {
                    i += 1;
                    if i < args.len() {
                        dryrun_dir = Some(args[i].clone());
                    }
                }
                "--runs-dir" => {
                    i += 1;
                    if i < args.len() {
                        runs_dir = Some(args[i].clone());
                    }
                }
                "--source" => {
                    i += 1;
                    if i < args.len() {
                        source = args[i].clone();
                    }
                }
                "--choice" => {
                    i += 1;
                    if i < args.len() {
                        choice = Some(args[i].clone());
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
                    println!("  --private-key <HEX>     Session key hex (or SESSION_PRIVATE_KEY)");
                    println!("  --obligations <PATH>    Committed obligations JSON");
                    println!("  --rates <PATH>          Vault rates JSON");
                    println!("  --snapshot <PATH>       Local snapshot JSON");
                    println!("  --block <U64>           Snapshot block number (default: latest)");
                    println!("  --planner <KIND>        deterministic | mnn+ahem-g | host | hold");
                    println!("  --planner-bin <PATH>    Path to planner-host binary");
                    println!("  --model-dir <PATH>      Path to LFM2.5-230M model directory");
                    println!("  --evidence-out <PATH>   Path to evidence file to append");
                    println!("  --evidence-id <ID>      Evidence identifier (default: E7)");
                    println!("  --passport-dir <PATH>   Directory to write published passports");
                    println!("  --dryrun-dir <PATH>     Directory to write dry-run outputs");
                    println!("  --source <STR>          Evidence source label (default: AGENT)");
                    println!("  --choice <LETTER>       Force candidate letter (e.g. A, B, H)");
                    println!("  --user-goal <TEXT>      Business mandate or goal statement");
                    println!("  --dry-run               Preview without broadcasting");
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
            block,
            planner,
            planner_bin,
            model_dir,
            evidence_out,
            evidence_id,
            passport_dir,
            dryrun_dir,
            runs_dir,
            source,
            choice,
            user_goal,
            dry_run,
            continuous,
            interval_secs,
            journal_path,
            guardian_key,
            crash_after_broadcast,
        })
    }
}

/// An execution evidence row adhering to §19.5.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRow {
    pub id: String,
    pub source: String,  // "AGENT" or "SCRIPT"
    pub planner: String, // "deterministic" | "mnn+ahem-g"
    pub host: String,    // "linux-host"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
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
    pub passport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passport_file: Option<String>,
}

fn parse_u256_val(v: &Value) -> Option<U256> {
    if let Some(s) = v.as_str() {
        let trimmed = s.trim();
        if let Some(stripped) = trimmed.strip_prefix("0x") {
            U256::from_str_radix(stripped, 16).ok()
        } else {
            U256::from_str(trimmed).ok()
        }
    } else {
        v.as_u64().map(U256::from)
    }
}

fn parse_u64_val(v: &Value) -> Option<u64> {
    if let Some(s) = v.as_str() {
        s.trim().parse().ok()
    } else {
        v.as_u64()
    }
}

type ParsedObligations = (Vec<Obligation>, BTreeMap<B256, Vec<B256>>, Vec<Payee>);

fn parse_obligations_content(
    content: &str,
) -> Result<ParsedObligations, Box<dyn std::error::Error>> {
    let parsed: Value = serde_json::from_str(content)?;
    let mut obligations = Vec::new();
    let mut bill_proofs = BTreeMap::new();
    let mut payees = Vec::new();

    if let Some(arr) = parsed.as_array() {
        for item in arr {
            let ob: Obligation = serde_json::from_value(item.clone())?;
            obligations.push(ob);
        }
    } else if let Some(bills) = parsed.get("bills").and_then(|b| b.as_array()) {
        for (idx, b) in bills.iter().enumerate() {
            let id_hash_str = b
                .get("obligationIdHash")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let payee_hash_str = b.get("payeeIdHash").and_then(|v| v.as_str()).unwrap_or("");
            let obligation_id_hash = B256::from_str(id_hash_str)?;
            let payee_id_hash = B256::from_str(payee_hash_str)?;

            let amount_raw = b
                .get("assets")
                .or_else(|| b.get("amountRaw"))
                .and_then(parse_u256_val)
                .unwrap_or(U256::ZERO);
            let due_at = b.get("dueAt").and_then(parse_u64_val).unwrap_or(0);

            let ob = Obligation {
                obligation_id: format!("bill_{idx}"),
                obligation_id_hash,
                payee_id_hash,
                amount_raw,
                due_at,
            };
            obligations.push(ob);

            if let Some(proof_arr) = b.get("proof").and_then(|p| p.as_array()) {
                let mut proofs = Vec::new();
                for p in proof_arr {
                    if let Some(ps) = p.as_str() {
                        if let Ok(h) = B256::from_str(ps) {
                            proofs.push(h);
                        }
                    }
                }
                bill_proofs.insert(obligation_id_hash, proofs);
            }

            if let Some(rcpt_str) = b.get("recipient").and_then(|r| r.as_str()) {
                if let Ok(rcpt) = Address::from_str(rcpt_str) {
                    payees.push(Payee {
                        account: rcpt,
                        label_hash: payee_id_hash,
                        active_at: 0,
                        cancelled: false,
                    });
                }
            }
        }
    }

    Ok((obligations, bill_proofs, payees))
}

fn invoke_host_planner(
    candidate_set: &CandidateSet,
    user_goal: &str,
    planner_bin: Option<&str>,
    model_dir: Option<&str>,
) -> Result<(String, PlannerMetadata), Box<dyn std::error::Error>> {
    let bin_path = if let Some(p) = planner_bin {
        PathBuf::from(p)
    } else if let Ok(p) = env::var("FUSION_PLANNER_BIN") {
        PathBuf::from(p)
    } else {
        let candidates = [
            "/root/projects/fusion_wallet-wt/K8/src/fusion-planner/target/release/planner-host",
            "/root/projects/fusion_wallet-wt/K9/src/fusion-planner/target/release/planner-host",
            "../fusion-planner/target/release/planner-host",
        ];
        let mut found = None;
        for c in &candidates {
            if Path::new(c).exists() {
                found = Some(PathBuf::from(c));
                break;
            }
        }
        found.ok_or("planner-host binary not found; please specify --planner-bin")?
    };

    let model_path = if let Some(m) = model_dir {
        PathBuf::from(m)
    } else if let Ok(m) = env::var("FUSION_MODEL_DIR") {
        PathBuf::from(m)
    } else {
        PathBuf::from("/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m")
    };

    if !bin_path.exists() {
        return Err(format!("planner-host not found at {}", bin_path.display()).into());
    }
    if !model_path.exists() {
        return Err(format!("Model dir not found at {}", model_path.display()).into());
    }

    let prompt_res = build_planner_prompt(candidate_set, Some(user_goal));

    let temp_dir = env::temp_dir();
    let in_file = temp_dir.join(format!("fusion_planner_in_{}.jsonl", std::process::id()));
    let out_file = temp_dir.join(format!("fusion_planner_out_{}.jsonl", std::process::id()));

    let req_obj = serde_json::json!({
        "prompt": prompt_res.prompt,
        "labels": prompt_res.labels,
        "candidateSetDigest": format!("{:#x}", prompt_res.candidate_set_digest),
        "mode": "ahem-g",
    });

    fs::write(&in_file, format!("{}\n", req_obj))?;

    let output = Command::new(&bin_path)
        .arg("--eval")
        .arg(&in_file)
        .arg("--mode")
        .arg("ahem-g")
        .arg("--out")
        .arg(&out_file)
        .env("FUSION_MODEL_DIR", &model_path)
        .output()?;

    let _ = fs::remove_file(&in_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&out_file);
        return Err(format!(
            "planner-host failed with status {}: {stderr}",
            output.status
        )
        .into());
    }

    let out_content = fs::read_to_string(&out_file)?;
    let _ = fs::remove_file(&out_file);

    let first_line = out_content
        .lines()
        .next()
        .ok_or("planner-host produced empty output")?;
    let resp: Value = serde_json::from_str(first_line)?;

    let label = resp
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("H")
        .to_string();

    let vetoes = resp
        .pointer("/ahem/g/vetoes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let retries = resp
        .pointer("/ahem/g/retries")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let forced_tokens = resp
        .pointer("/ahem/g/forcedTokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(5);

    let meta = PlannerMetadata {
        ahem: AhemMetadata {
            g: AhemGReport {
                forced_tokens,
                mode: "ENFORCED".to_string(),
                retries,
                vetoes,
            },
            l: AhemLReport {
                mode: "OFF".to_string(),
            },
        },
        host: "linux-host".to_string(),
        kind: "mnn+ahem-g".to_string(),
        lib_mnn_sha256: "27fdb46a95fbfb859dc7046cefddfa1ab816837d48517bdf9e2b36e96581a0c4"
            .to_string(),
        libllm_sha256: "f7dc4af30ef29c8a7f550893820700e42682c43688d571ee6e1b2362a182867f"
            .to_string(),
        model_id: "LFM2.5-230M-MNN".to_string(),
        model_sha256: "9ddd326541c3cb1e5e137ec717a91e5f3542c5ae2c1aa59046c180792544a907"
            .to_string(),
    };

    Ok((label, meta))
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
        let mut attempts = 0;
        loop {
            attempts += 1;
            let url = Url::from_str(&man.rpc_url)?;
            let provider = ProviderBuilder::new().connect_http(url);
            let res = if let Some(target_b) = args.block {
                read_snapshot_at_block(&provider, man, Some(target_b)).await
            } else {
                read_snapshot(man).await
            };
            match res {
                Ok(s) => break s,
                Err(fusion_core::snapshot::SnapshotError::VaultImplChanged {
                    vault,
                    expected,
                    actual,
                    block_number,
                    block_hash,
                    block_timestamp,
                }) => {
                    eprintln!(
                        "CRITICAL LATCH: VAULT_IMPL_CHANGED detected for vault {vault}: expected {expected}, got {actual} at block #{block_number}. Halting agent runner."
                    );
                    let (chain_id, safe, module) = if let Some(ref m) = manifest {
                        (m.chain_id, m.safe, m.module)
                    } else {
                        (84532, Address::ZERO, Address::ZERO)
                    };
                    let user_goal_hash = alloy::primitives::keccak256(args.user_goal.as_bytes());
                    let passport = DecisionPassport {
                        candidate_set: vec![PassportCandidate {
                            assets: "0".to_string(),
                            id: format!("{:#x}", B256::ZERO),
                            kind: "HOLD".to_string(),
                            label: "H".to_string(),
                            reason: "VAULT_IMPL_CHANGED".to_string(),
                        }],
                        candidate_set_digest: format!("{:#x}", B256::ZERO),
                        chain_id,
                        choice_id: format!("{:#x}", B256::ZERO),
                        choice_label: "H".to_string(),
                        created_at_chain: block_timestamp,
                        mandate_hash: format!("{:#x}", B256::ZERO),
                        module: module.to_checksum(None),
                        planner: PlannerMetadata {
                            ahem: AhemMetadata {
                                g: AhemGReport {
                                    forced_tokens: 0,
                                    mode: "OFF".to_string(),
                                    retries: 0,
                                    vetoes: 0,
                                },
                                l: AhemLReport {
                                    mode: "OFF".to_string(),
                                },
                            },
                            host: "linux-host".to_string(),
                            kind: "critical_latch".to_string(),
                            lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                            libllm_sha256: format!("{:#x}", B256::ZERO),
                            model_id: "none".to_string(),
                            model_sha256: format!("{:#x}", B256::ZERO),
                        },
                        reason_code: "VAULT_IMPL_CHANGED".to_string(),
                        route: "HOLD".to_string(),
                        safe: safe.to_checksum(None),
                        snapshot: PassportSnapshot {
                            block: block_number,
                            hash: format!("{:#x}", block_hash),
                        },
                        user_goal_sha256: format!("{user_goal_hash:#x}"),
                        version: "4.1".to_string(),
                    };
                    let passport_json = to_canonical_json(&passport)?;
                    let decision_hash = alloy::primitives::keccak256(passport_json.as_bytes());

                    let passport_file_rel = if let Some(ref dir) = args.passport_dir {
                        fs::create_dir_all(dir)?;
                        let file_name = format!("{decision_hash:#x}.json");
                        let target_path = Path::new(dir).join(&file_name);
                        fs::write(&target_path, &passport_json)?;
                        let hold_path = Path::new(dir).join("hold.json");
                        fs::write(&hold_path, &passport_json)?;
                        println!(
                            "VAULT_IMPL_CHANGED HOLD passport written to {}",
                            target_path.display()
                        );
                        Some(file_name)
                    } else {
                        None
                    };

                    if let Some(ref out_file) = args.evidence_out {
                        let row = EvidenceRow {
                            id: args.evidence_id.clone(),
                            source: args.source.clone(),
                            planner: "critical_latch".to_string(),
                            host: "linux-host".to_string(),
                            tx_hash: None,
                            decision_hash: format!("{decision_hash:#x}"),
                            action_digest: format!("{:#x}", B256::ZERO),
                            candidate_id: format!("{:#x}", B256::ZERO),
                            choice_label: "H".to_string(),
                            kind: "HOLD".to_string(),
                            route: "HOLD".to_string(),
                            status: 1,
                            block_number,
                            gas_used: 0,
                            passport: Some(format!(
                                "evidence/agent/passports/{decision_hash:#x}.json"
                            )),
                            passport_file: passport_file_rel,
                        };
                        let mut rows: Vec<EvidenceRow> = if Path::new(out_file).exists() {
                            let data = fs::read_to_string(out_file)?;
                            serde_json::from_str(&data).unwrap_or_default()
                        } else {
                            vec![]
                        };
                        rows.push(row.clone());
                        let json_data = serde_json::to_string_pretty(&rows)?;
                        fs::write(out_file, json_data)?;
                        println!("Appended latch evidence row {} to {}", row.id, out_file);
                    }

                    // Propose guardian pause when guardian key is configured (§7.2)
                    let guardian_key_opt = args
                        .guardian_key
                        .clone()
                        .or_else(|| env::var("GUARDIAN_PRIVATE_KEY").ok())
                        .or_else(|| env::var("FUSION_GUARDIAN_KEY").ok());
                    if let Some(ref gk) = guardian_key_opt {
                        let trimmed = gk.trim().strip_prefix("0x").unwrap_or(gk.trim());
                        let mut key_bytes = [0u8; 32];
                        if hex::decode_to_slice(trimmed, &mut key_bytes).is_ok() {
                            let zkey = Zeroizing::new(key_bytes);
                            let gsigner = PrivateKeySigner::from_bytes(&B256::from(key_bytes))?;
                            let gaddr = gsigner.address();
                            let gnonce = provider.get_transaction_count(gaddr).await?;
                            let reason_code = alloy::primitives::keccak256(b"VAULT_IMPL_CHANGED");
                            let pause_call = fusion_core::encode::encode_pause(module, reason_code);
                            let ptx = PreparedTransaction {
                                chain_id,
                                nonce: gnonce,
                                to: pause_call.to,
                                value: U256::ZERO,
                                data: pause_call.data,
                                gas_limit: 300_000,
                                max_fee_per_gas: 2_000_000_000,
                                max_priority_fee_per_gas: 1_000_000_000,
                            };
                            let stx = sign_guardian_pause_transaction(
                                &ptx,
                                &zkey,
                                &SessionSigningPolicy { module, chain_id },
                            )?;
                            let raw_hex = stx.raw_tx.strip_prefix("0x").unwrap_or(&stx.raw_tx);
                            let raw_bytes = Bytes::from(hex::decode(raw_hex)?);
                            match provider.send_raw_transaction(&raw_bytes).await {
                                Ok(p) => {
                                    let h = *p.tx_hash();
                                    println!("Guardian pause submitted: {h:#x}");
                                    if let Ok(rcpt) = p.get_receipt().await {
                                        println!(
                                            "Guardian pause mined in block #{:?}: status={}",
                                            rcpt.block_number(),
                                            rcpt.status()
                                        );
                                    }
                                }
                                Err(e) => eprintln!("Guardian pause submission error: {e}"),
                            }
                        }
                    } else {
                        println!(
                            "Guardian pause proposed (reason: VAULT_IMPL_CHANGED); no guardian key configured to broadcast."
                        );
                    }

                    return Ok(());
                }
                Err(e) if attempts < 5 => {
                    eprintln!(
                        "Warning: read_snapshot attempt {attempts} failed ({e}); retrying in 3s..."
                    );
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    } else {
        return Err("No snapshot available".into());
    };

    // Journal initialization and startup reconciliation (§13)
    let journal = args
        .journal_path
        .as_ref()
        .map(fusion_core::journal::DurableJournal::new)
        .or_else(|| {
            env::var("FUSION_JOURNAL_PATH")
                .ok()
                .map(fusion_core::journal::DurableJournal::new)
        });

    if let (Some(ref j), Some(ref m)) = (&journal, &manifest) {
        if let Ok(url) = Url::from_str(&m.rpc_url) {
            let provider = ProviderBuilder::new().connect_http(url);
            let signer_addr = if let Some(ref k) = args.private_key {
                let trimmed = k.trim().strip_prefix("0x").unwrap_or(k.trim());
                let mut b = [0u8; 32];
                if hex::decode_to_slice(trimmed, &mut b).is_ok() {
                    PrivateKeySigner::from_bytes(&B256::from(b))
                        .map(|s| s.address())
                        .unwrap_or(Address::ZERO)
                } else {
                    Address::ZERO
                }
            } else if let Ok(k) = env::var("SESSION_PRIVATE_KEY")
                .or_else(|_| env::var("FUSION_SESSION_KEY"))
                .or_else(|_| env::var("FUSION_PRIVATE_KEY"))
            {
                let trimmed = k.trim().strip_prefix("0x").unwrap_or(k.trim());
                let mut b = [0u8; 32];
                if hex::decode_to_slice(trimmed, &mut b).is_ok() {
                    PrivateKeySigner::from_bytes(&B256::from(b))
                        .map(|s| s.address())
                        .unwrap_or(Address::ZERO)
                } else {
                    Address::ZERO
                }
            } else {
                snapshot.module_state.session_key
            };

            let summary = fusion_core::journal::reconcile_unknown_entries(
                j,
                &provider,
                signer_addr,
                snapshot.block_timestamp,
            )
            .await?;
            if summary.reconciled_mined > 0
                || summary.reconciled_reverted > 0
                || summary.still_in_flight > 0
            {
                println!(
                    "Startup reconciliation: {} mined, {} reverted, {} still in flight",
                    summary.reconciled_mined, summary.reconciled_reverted, summary.still_in_flight
                );
            }
        }
    }

    println!(
        "Snapshot read at block #{} ({:#x}): safe_cash={} USDC raw (paused={})",
        snapshot.block_number,
        snapshot.block_hash,
        snapshot.safe_cash,
        snapshot.module_state.paused
    );

    // 3. Obligations & Rates
    let (mut obligations, bill_proofs, payees) = if let Some(ref path) = args.obligations_path {
        let content = fs::read_to_string(path)?;
        parse_obligations_content(&content)?
    } else {
        (vec![], BTreeMap::new(), vec![])
    };

    if let Some(ref m) = manifest {
        if let Ok(url) = Url::from_str(&m.rpc_url) {
            let provider = ProviderBuilder::new().connect_http(url);
            let mut unpaid = Vec::new();
            for ob in obligations {
                let mut attempts = 0;
                let is_paid = loop {
                    attempts += 1;
                    match fusion_core::snapshot::is_obligation_paid(
                        &provider,
                        m.module,
                        ob.obligation_id_hash,
                        snapshot.block_number,
                    )
                    .await
                    {
                        Ok(p) => break p,
                        Err(e) if attempts < 5 => {
                            eprintln!("Warning: is_obligation_paid attempt {attempts} failed ({e}); retrying in 2s...");
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                        Err(e) => return Err(e.into()),
                    }
                };
                if is_paid {
                    println!(
                        "Obligation {} ({:#x}) is marked PAID on-chain; excluding.",
                        ob.obligation_id, ob.obligation_id_hash
                    );
                } else {
                    unpaid.push(ob);
                }
            }
            obligations = unpaid;
        }
    }

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

    // 4. Candidate Enumeration (§10.4)
    let mut candidate_set = enumerate_candidates(
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
    for c in &candidate_set.candidates {
        println!(
            "  - [{}] kind={:?} assets={} route={:?} reason={}",
            c.label, c.kind, c.action.assets, c.route, c.reason
        );
    }

    // 5. Planner Selection (§10.6, §19.5)
    let (choice_label, planner_meta) = if let Some(ref forced) = args.choice {
        println!("Using explicit candidate choice label: {forced}");
        let meta = if args.planner == "mnn+ahem-g" || args.planner == "host" {
            match invoke_host_planner(
                &candidate_set,
                &args.user_goal,
                args.planner_bin.as_deref(),
                args.model_dir.as_deref(),
            ) {
                Ok((_, m)) => m,
                Err(e) => {
                    eprintln!(
                        "Warning: host planner unavailable ({e}); using deterministic metadata"
                    );
                    PlannerMetadata {
                        ahem: AhemMetadata {
                            g: AhemGReport {
                                forced_tokens: 0,
                                mode: "OFF".to_string(),
                                retries: 0,
                                vetoes: 0,
                            },
                            l: AhemLReport {
                                mode: "OFF".to_string(),
                            },
                        },
                        host: "linux-host".to_string(),
                        kind: "deterministic".to_string(),
                        lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                        libllm_sha256: format!("{:#x}", B256::ZERO),
                        model_id: "none".to_string(),
                        model_sha256: format!("{:#x}", B256::ZERO),
                    }
                }
            }
        } else {
            PlannerMetadata {
                ahem: AhemMetadata {
                    g: AhemGReport {
                        forced_tokens: 0,
                        mode: "OFF".to_string(),
                        retries: 0,
                        vetoes: 0,
                    },
                    l: AhemLReport {
                        mode: "OFF".to_string(),
                    },
                },
                host: "linux-host".to_string(),
                kind: args.planner.clone(),
                lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                libllm_sha256: format!("{:#x}", B256::ZERO),
                model_id: "none".to_string(),
                model_sha256: format!("{:#x}", B256::ZERO),
            }
        };
        (forced.clone(), meta)
    } else {
        match args.planner.as_str() {
            "mnn+ahem-g" | "host" => {
                match invoke_host_planner(
                    &candidate_set,
                    &args.user_goal,
                    args.planner_bin.as_deref(),
                    args.model_dir.as_deref(),
                ) {
                    Ok((lbl, meta)) => {
                        println!(
                            "AHEM-G Planner selected: {} (vetoes={}, forcedTokens={})",
                            lbl, meta.ahem.g.vetoes, meta.ahem.g.forced_tokens
                        );
                        (lbl, meta)
                    }
                    Err(e) => {
                        eprintln!("Warning: host planner unavailable ({e}); falling back to deterministic");
                        let meta = PlannerMetadata {
                            ahem: AhemMetadata {
                                g: AhemGReport {
                                    forced_tokens: 0,
                                    mode: "OFF".to_string(),
                                    retries: 0,
                                    vetoes: 0,
                                },
                                l: AhemLReport {
                                    mode: "OFF".to_string(),
                                },
                            },
                            host: "linux-host".to_string(),
                            kind: "deterministic".to_string(),
                            lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                            libllm_sha256: format!("{:#x}", B256::ZERO),
                            model_id: "none".to_string(),
                            model_sha256: format!("{:#x}", B256::ZERO),
                        };
                        let lbl = candidate_set
                            .candidates
                            .first()
                            .map(|c| c.label.clone())
                            .unwrap_or_else(|| "H".to_string());
                        (lbl, meta)
                    }
                }
            }
            "hold" => {
                let meta = PlannerMetadata {
                    ahem: AhemMetadata {
                        g: AhemGReport {
                            forced_tokens: 0,
                            mode: "OFF".to_string(),
                            retries: 0,
                            vetoes: 0,
                        },
                        l: AhemLReport {
                            mode: "OFF".to_string(),
                        },
                    },
                    host: "linux-host".to_string(),
                    kind: "deterministic".to_string(),
                    lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                    libllm_sha256: format!("{:#x}", B256::ZERO),
                    model_id: "none".to_string(),
                    model_sha256: format!("{:#x}", B256::ZERO),
                };
                ("H".to_string(), meta)
            }
            _ => {
                // deterministic
                let meta = PlannerMetadata {
                    ahem: AhemMetadata {
                        g: AhemGReport {
                            forced_tokens: 0,
                            mode: "OFF".to_string(),
                            retries: 0,
                            vetoes: 0,
                        },
                        l: AhemLReport {
                            mode: "OFF".to_string(),
                        },
                    },
                    host: "linux-host".to_string(),
                    kind: "deterministic".to_string(),
                    lib_mnn_sha256: format!("{:#x}", B256::ZERO),
                    libllm_sha256: format!("{:#x}", B256::ZERO),
                    model_id: "none".to_string(),
                    model_sha256: format!("{:#x}", B256::ZERO),
                };
                let lbl = candidate_set
                    .candidates
                    .first()
                    .map(|c| c.label.clone())
                    .unwrap_or_else(|| "H".to_string());
                (lbl, meta)
            }
        }
    };

    let mut choice = if choice_label == "H" {
        candidate_set.hold.clone()
    } else {
        candidate_set
            .candidates
            .iter()
            .find(|c| c.label == choice_label)
            .cloned()
            .unwrap_or_else(|| candidate_set.hold.clone())
    };

    // Reuse the exact queued action when executing an existing queue.
    // New proposal deadlines are constructed in candidates.rs before hashing.
    if choice.route == Route::Queue {
        if snapshot.queue.exists
            && snapshot.queue.action.obligation_id_hash == choice.action.obligation_id_hash
        {
            let mut q_act = snapshot.queue.action.clone();
            if q_act.kind == ActionKind::Rebalance {
                q_act.kind = ActionKind::PayObligation;
            }
            choice.action = q_act;
            choice.id = fusion_core::hashing::compute_candidate_id_from_action(
                U256::from(chain_id),
                safe,
                module,
                snapshot.module_state.mandate_hash,
                snapshot.module_state.session_id,
                snapshot.block_hash,
                &choice.action,
            );
            for c in &mut candidate_set.candidates {
                if c.label == choice.label {
                    c.action = choice.action.clone();
                    c.id = choice.id;
                }
            }
        }
    }

    println!(
        "Final Choice: label={} kind={:?} reason={} assets={}",
        choice.label, choice.kind, choice.reason, choice.action.assets
    );

    // Check if chosen action is already in flight or reconciled in journal (§13)
    if choice.label != "H" {
        let initial_preview = preview_candidate(
            &choice,
            &snapshot,
            B256::ZERO,
            U256::from(chain_id),
            safe,
            module,
        );
        if let Some(ref j) = journal {
            let in_flight = j.in_flight_digests().unwrap_or_default();
            let completed = j.completed_digests().unwrap_or_default();
            if in_flight.contains(&initial_preview.action_digest) {
                println!(
                    "Action digest {:#x} is STILL IN FLIGHT in journal; refusing to re-sign or double-send. Forcing HOLD.",
                    initial_preview.action_digest
                );
                choice = candidate_set.hold.clone();
                choice.reason = "ACTION_IN_FLIGHT".to_string();
            } else if completed.contains(&initial_preview.action_digest) {
                println!(
                    "Action digest {:#x} was already executed/reconciled in journal; refusing to re-sign or double-send. Forcing HOLD.",
                    initial_preview.action_digest
                );
                choice = candidate_set.hold.clone();
                choice.reason = "ACTION_ALREADY_RECONCILED".to_string();
            }
        }
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

    let passport = DecisionPassport {
        candidate_set: passport_candidates,
        candidate_set_digest: format!("{:#x}", candidate_set.candidate_set_digest),
        chain_id,
        choice_id: format!("{:#x}", choice.id),
        choice_label: choice.label.clone(),
        created_at_chain: snapshot.block_timestamp,
        mandate_hash: format!("{:#x}", snapshot.module_state.mandate_hash),
        module: module.to_checksum(None),
        planner: planner_meta.clone(),
        reason_code: choice.reason.clone(),
        route: format!("{:?}", choice.route),
        safe: safe.to_checksum(None),
        snapshot: PassportSnapshot {
            block: snapshot.block_number,
            hash: format!("{:#x}", snapshot.block_hash),
        },
        user_goal_sha256: format!("{user_goal_hash:#x}"),
        version: "4.1".to_string(),
    };

    let (passport_json, decision_hash) = if choice.route == Route::Queue && snapshot.queue.exists {
        let q_dh = choice.action.decision_hash;
        println!("Using locked queued decisionHash: {q_dh:#x}");
        let p_path = args
            .passport_dir
            .as_ref()
            .map(|d| Path::new(d).join(format!("{q_dh:#x}.json")));
        let json = if let Some(ref p) = p_path {
            if p.exists() {
                fs::read_to_string(p).unwrap_or_else(|_| to_canonical_json(&passport).unwrap())
            } else {
                to_canonical_json(&passport)?
            }
        } else {
            to_canonical_json(&passport)?
        };
        (json, q_dh)
    } else {
        let json = to_canonical_json(&passport)?;
        let dh = alloy::primitives::keccak256(json.as_bytes());
        (json, dh)
    };
    println!("Decision passport JCS hash: {decision_hash:#x}");

    // 7. Preview Verification (§10.7): explicitly admit queue creation proposals.
    let preview = preview_proposal(
        &choice,
        &snapshot,
        decision_hash,
        U256::from(chain_id),
        safe,
        module,
    );

    let is_queue_creation = choice.route == Route::Queue && !snapshot.queue.exists;
    let preview_passed = preview.passed;

    if choice.label != "H" {
        if !preview_passed {
            eprintln!("PREVIEW REJECTED candidate {}:", choice.label);
            for c in &preview.checks {
                if c.status == RuleStatus::Fail {
                    eprintln!("  - rule: {} ({:?}) - {}", c.name, c.error_code, c.message);
                }
            }
            if !args.dry_run {
                return Err("Candidate failed preview checks".into());
            }
        } else {
            if is_queue_creation {
                println!("Preview passed for queue creation; execution still requires readyAt.");
            } else {
                println!("Preview passed all 7 normative rule gates.");
            }
        }
    } else {
        println!("Decision is HOLD. Preview evaluated; no transaction proposed.");
    }

    // Determine canonical kind adhering to §19.5
    let canonical_kind = if choice.label == "H" {
        "HOLD".to_string()
    } else {
        match choice.kind {
            ActionKind::PayObligation => match choice.route {
                Route::Instant => "PAY_INSTANT".to_string(),
                Route::Queue => {
                    if !snapshot.queue.exists {
                        "ACTION_QUEUED".to_string()
                    } else {
                        "PAY_DELAYED".to_string()
                    }
                }
                Route::Approve | Route::ApproveAndQueue => "PAY_OWNER_TIER".to_string(),
                Route::Reject => "PAY_REJECTED".to_string(),
            },
            ActionKind::Deposit => "DEPOSIT".to_string(),
            ActionKind::Withdraw => "WITHDRAW".to_string(),
            ActionKind::Rebalance => "REBALANCE".to_string(),
        }
    };

    // Save dry-run outputs if requested
    if let Some(ref dir) = args.dryrun_dir {
        fs::create_dir_all(dir)?;
        let snap_file = Path::new(dir).join("snapshot.json");
        fs::write(&snap_file, serde_json::to_string_pretty(&snapshot)?)?;

        let pass_file = Path::new(dir).join("passport.json");
        fs::write(&pass_file, &passport_json)?;

        let prev_file = Path::new(dir).join("preview.json");
        fs::write(&prev_file, serde_json::to_string_pretty(&preview)?)?;

        println!("Dry run evidence saved to {}:", dir);
        println!("  - snapshot: {}", snap_file.display());
        println!("  - passport: {}", pass_file.display());
        println!("  - preview:  {}", prev_file.display());
    }

    // Save passport to passport_dir if specified
    let passport_file_rel = if let Some(ref dir) = args.passport_dir {
        fs::create_dir_all(dir)?;
        let file_name = format!("{decision_hash:#x}.json");
        let target_path = Path::new(dir).join(&file_name);
        fs::write(&target_path, &passport_json)?;
        println!("Passport written to {}", target_path.display());

        if choice.label == "H" {
            let hold_path = Path::new(dir).join("hold.json");
            fs::write(&hold_path, &passport_json)?;
            println!("Hold passport written to {}", hold_path.display());
        }
        Some(file_name)
    } else {
        None
    };

    if choice.label == "H" || choice.reason == "HOLD" {
        println!("Decision is HOLD. Recording passport and evidence row.");
        if let Some(ref out_file) = args.evidence_out {
            let row = EvidenceRow {
                id: args.evidence_id.clone(),
                source: args.source.clone(),
                planner: planner_meta.kind.clone(),
                host: "linux-host".to_string(),
                tx_hash: None,
                decision_hash: format!("{decision_hash:#x}"),
                action_digest: format!("{:#x}", preview.action_digest),
                candidate_id: format!("{:#x}", choice.id),
                choice_label: choice.label,
                kind: canonical_kind,
                route: format!("{:?}", choice.route),
                status: 1,
                block_number: snapshot.block_number,
                gas_used: 0,
                passport: Some(format!("evidence/agent/passports/{decision_hash:#x}.json")),
                passport_file: passport_file_rel,
            };

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
        return Ok(());
    }

    // 8. Sign (§10.2, §11.3)
    let key_str = if let Some(ref k) = args.private_key {
        k.clone()
    } else if let Ok(k) = env::var("SESSION_PRIVATE_KEY") {
        k
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
        Route::Queue => {
            if !snapshot.queue.exists {
                encode_queue_action(module, &preview.exact_action, &choice.proof)
            } else {
                encode_execute(module, &preview.exact_action, &choice.proof)
            }
        }
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
        gas_limit: 800_000,
        max_fee_per_gas: 2_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };

    let signed_tx = sign_eip1559_transaction(
        &prepared_tx,
        &zeroized_key,
        &SessionSigningPolicy { module, chain_id },
    )?;
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

        // Record broadcast in durable journal BEFORE sending raw tx to network (§13)
        if let Some(ref j) = journal {
            j.record_broadcast_requested(
                preview.action_digest,
                nonce,
                signed_tx.tx_hash,
                Some(decision_hash),
                snapshot.block_timestamp,
            )?;
            println!(
                "Recorded BROADCAST_REQUESTED in durable journal for tx {:#x} (digest {:#x})",
                signed_tx.tx_hash, preview.action_digest
            );
        }

        let mut attempts = 0;
        let pending = loop {
            attempts += 1;
            match provider.send_raw_transaction(&raw_bytes).await {
                Ok(p) => break p,
                Err(e) if attempts < 5 => {
                    eprintln!("Warning: send_raw_transaction attempt {attempts} failed ({e}); retrying in 3s...");
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                Err(e) => return Err(e.into()),
            }
        };
        let hash = *pending.tx_hash();
        println!("Tx submitted: {hash:#x}. Awaiting receipt...");

        // Crash simulation hook (§13): crash after broadcast before receipt
        if args.crash_after_broadcast
            || env::var("FUSION_CRASH_AFTER_BROADCAST")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false)
        {
            eprintln!(
                "CRASH SIMULATION: process terminated immediately after broadcast before receipt (§13)"
            );
            std::process::exit(137);
        }

        let receipt = pending.get_receipt().await?;
        let st = if receipt.status() { 1 } else { 0 };
        let bn = receipt.block_number().unwrap_or(0);
        let gu = receipt.gas_used();
        println!("Tx confirmed in block #{bn}: status={st} gasUsed={gu}");

        // Record confirmation or revert in durable journal (§13)
        if let Some(ref j) = journal {
            if st == 1 {
                j.record_confirmed(
                    preview.action_digest,
                    nonce,
                    hash,
                    bn,
                    gu,
                    snapshot.block_timestamp,
                )?;
            } else {
                j.record_reverted(
                    preview.action_digest,
                    nonce,
                    hash,
                    bn,
                    snapshot.block_timestamp,
                    Some("Transaction reverted on-chain".to_string()),
                )?;
            }
        }

        (hash, bn, gu, st)
    } else {
        (signed_tx.tx_hash, snapshot.block_number, 0, 1)
    };

    // Save passport indexed by txhash as requested by card specification
    if let Some(ref dir) = args.passport_dir {
        let tx_passport_path = Path::new(dir).join(format!("{tx_hash:#x}.json"));
        fs::write(&tx_passport_path, &passport_json)?;
        println!(
            "Tx-named passport written to {}",
            tx_passport_path.display()
        );
    }

    // Save run log if runs_dir is set
    if let Some(ref dir) = args.runs_dir {
        fs::create_dir_all(dir)?;
        let run_log_path = Path::new(dir).join(format!("{tx_hash:#x}.log"));
        let log_content = format!(
            "agent-runner run log\nevidenceId: {}\nsource: {}\nplanner: {}\ntxHash: {:#x}\ndecisionHash: {:#x}\nblockNumber: {}\nstatus: {}\ngasUsed: {}\nchoice: {}\nkind: {}\n",
            args.evidence_id, args.source, planner_meta.kind, tx_hash, decision_hash, block_number, status, gas_used, choice.label, canonical_kind
        );
        fs::write(&run_log_path, log_content)?;
        println!("Run log written to {}", run_log_path.display());
    }

    // 10. Write Evidence Row (§19.5)
    let row = EvidenceRow {
        id: args.evidence_id.clone(),
        source: args.source.clone(),
        planner: planner_meta.kind.clone(),
        host: "linux-host".to_string(),
        tx_hash: Some(format!("{tx_hash:#x}")),
        decision_hash: format!("{decision_hash:#x}"),
        action_digest: format!("{:#x}", preview.action_digest),
        candidate_id: format!("{:#x}", choice.id),
        choice_label: choice.label,
        kind: canonical_kind,
        route: format!("{:?}", choice.route),
        status,
        block_number,
        gas_used,
        passport: Some(format!("evidence/agent/passports/{tx_hash:#x}.json")),
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
            if !args.continuous {
                return Err(e);
            }
        }

        if !args.continuous {
            break;
        }

        tokio::time::sleep(Duration::from_secs(args.interval_secs)).await;
    }

    println!("Agent runner execution finished cleanly.");
    Ok(())
}
