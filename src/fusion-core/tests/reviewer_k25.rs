//! K25 I/O bridge only. Scoring remains in the unchanged arena_pipeline test.
use alloy::primitives::{keccak256, Address};
use fusion_core::{candidates::CandidateSet, passport::*, planner_io::*, types::Snapshot};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

#[test]
fn reviewer_bridge() {
    let Ok(dir) = std::env::var("K25_RESULTS") else {
        return;
    };
    let dir = PathBuf::from(dir);
    let prepared: Vec<Value> =
        serde_json::from_str(&fs::read_to_string(dir.join("prepared.json")).unwrap()).unwrap();
    let sets: Vec<CandidateSet> = prepared
        .iter()
        .map(|p| serde_json::from_value(p["candidateSet"].clone()).unwrap())
        .collect();
    match std::env::var("K25_STAGE").unwrap().as_str() {
        "prepare" => {
            let requests = prepared
                .iter()
                .zip(&sets)
                .map(|(p, set)| {
                    let facts = build_review_prompt(set, p["case"]["prompt"].as_str());
                    json!({"prompt": facts.prompt, "labels": facts.labels,
                    "candidateSetDigest": facts.candidate_set_digest, "mode": "ahem-g:review"})
                    .to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(dir.join("review-input.jsonl"), requests + "\n").unwrap();
        }
        "resolve" => {
            let models: Vec<Value> = fs::read_to_string(dir.join("planner-review.jsonl"))
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            assert_eq!(models.len(), prepared.len());
            let metadata: PlannerMetadata =
                serde_json::from_str(&fs::read_to_string(dir.join("metadata.json")).unwrap())
                    .unwrap();
            let mut resolved = vec![];
            let mut passports = vec![];
            for ((p, set), model) in prepared.iter().zip(&sets).zip(models) {
                assert_eq!(model["candidateSetDigest"], json!(set.candidate_set_digest));
                assert_eq!(model["plannerKind"], REVIEW_PLANNER_KIND);
                let snapshot: Snapshot = serde_json::from_value(p["snapshot"].clone()).unwrap();
                let mut passport = DecisionPassport {
                    candidate_set: set
                        .candidates
                        .iter()
                        .map(|c| PassportCandidate {
                            assets: c.action.assets.to_string(),
                            id: c.id.to_string(),
                            kind: format!("{:?}", c.kind),
                            label: c.label.clone(),
                            reason: c.reason.clone(),
                        })
                        .collect(),
                    candidate_set_digest: set.candidate_set_digest.to_string(),
                    chain_id: 31337,
                    choice_id: String::new(),
                    choice_label: String::new(),
                    created_at_chain: snapshot.block_timestamp,
                    mandate_hash: snapshot.module_state.mandate_hash.to_string(),
                    module: Address::repeat_byte(0x22).to_string(),
                    planner: metadata.clone(),
                    reason_code: String::new(),
                    route: String::new(),
                    safe: Address::repeat_byte(0x11).to_string(),
                    snapshot: PassportSnapshot {
                        block: snapshot.block_number,
                        hash: snapshot.block_hash.to_string(),
                    },
                    // Existing passport field name; runner uses keccak256 too.
                    user_goal_sha256: keccak256(
                        p["case"]["prompt"].as_str().unwrap_or("").as_bytes(),
                    )
                    .to_string(),
                    version: "4.1".into(),
                };
                passport.planner.ahem.g.forced_tokens =
                    model["ahem"]["g"]["forcedTokens"].as_u64().unwrap();
                passport.planner.ahem.g.retries = model["ahem"]["g"]["retries"].as_u64().unwrap();
                passport.planner.ahem.g.vetoes = model["ahem"]["g"]["vetoes"].as_u64().unwrap();
                let raw = if model["ahem"]["g"]["aborted"] == true {
                    ""
                } else {
                    model["raw"].as_str().unwrap()
                };
                let validation = apply_review_to_passport(raw, set, &mut passport);
                assert_eq!(passport.planner.kind, REVIEW_PLANNER_KIND);
                assert_eq!(
                    passport.candidate_set_digest,
                    set.candidate_set_digest.to_string()
                );
                if validation.choice_label == "H" {
                    assert_eq!(passport.route, "HOLD");
                    assert_eq!(passport.choice_id, set.hold.id.to_string());
                }
                let output = passport.build_output().unwrap();
                let mut adapted = model.clone();
                // A verdict is not an action label. Only the production core
                // adapter's resolved label goes to the original action scorer.
                adapted["reviewVerdict"] = model;
                adapted["reviewValidation"] = json!(validation);
                adapted["label"] = json!(validation.choice_label);
                // Invalid review text must stay invalid to the unchanged
                // scorer, including HOLD-correct cases; do not sanitize it
                // into a non-retryable, apparently valid HOLD.
                adapted["raw"] = if validation.retryable {
                    json!("")
                } else {
                    json!(json!({"choice": validation.choice_label}).to_string())
                };
                resolved.push(adapted.to_string());
                passports
                    .push(json!({"id":p["case"]["id"], "passport": passport, "output":output}));
            }
            fs::write(
                dir.join("resolved-review.jsonl"),
                resolved.join("\n") + "\n",
            )
            .unwrap();
            fs::write(
                dir.join("review-passports.json"),
                serde_json::to_string_pretty(&passports).unwrap(),
            )
            .unwrap();
        }
        _ => panic!("unknown K25 stage"),
    }
}
