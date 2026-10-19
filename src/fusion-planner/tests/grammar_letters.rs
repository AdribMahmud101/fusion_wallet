#[test]
fn grammar_contract() {
    use fusion_planner::candidate_grammar::*;
    let labels = vec!["A".into(), "H".into()];
    assert!(legal(&labels));
    assert_eq!(validate("{\"choice\":\"A\"}", &labels), Some("A".into()));
    for bad in [
        "{\"choice\":\"C\"}",
        "{\"choice\":\"A\",\"x\":0}",
        "{\"choice\":\"C\",\"choice\":\"A\"}",
    ] {
        assert!(validate(bad, &labels).is_none());
    }
}
#[test]
fn every_candidate_subset_accepts_all_its_letters() {
    use fusion_planner::candidate_grammar::*;
    for mask in 0..128 {
        let mut labels: Vec<String> = (0..7)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| ((b'A' + bit) as char).to_string())
            .collect();
        labels.push("H".into());
        assert!(legal(&labels));
        for letter in 'A'..='H' {
            let raw = format!("{PREFIX}{letter}{SUFFIX}");
            assert_eq!(
                validate(&raw, &labels).is_some(),
                labels.contains(&letter.to_string())
            );
        }
    }
}
#[test]
#[cfg_attr(
    not(planner_model_available),
    ignore = "NOT_RUN: FUSION_MODEL_DIR unset"
)]
fn tokenizer_letters() {
    let Ok(dir) = std::env::var("FUSION_MODEL_DIR") else {
        eprintln!("NOT_RUN tokenizer_letters: FUSION_MODEL_DIR unset");
        return;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/tokenizer-probe");
    let libs = fusion_planner::lib_dir();
    let status = std::process::Command::new("c++")
        .args(["-std=c++17"])
        .arg(root.join("tests/tokenizer_probe.cpp"))
        .arg(root.join("src/tokenizer_bridge.cpp"))
        .arg("-ldl")
        .arg(format!("-L{}", libs.display()))
        .arg(format!("-Wl,-rpath,{}", libs.display()))
        .args(["-lllm", "-o"])
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(
        std::process::Command::new(out)
            .arg(format!("{dir}/config.json"))
            .status()
            .unwrap()
            .success()
    );
}
