use fusion_planner::*;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| {
        args.iter()
            .position(|x| x == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let dir = std::env::var("FUSION_MODEL_DIR")?;
    let config = std::path::Path::new(&dir).join("config.json");
    let hashes = measured_hashes(&config)?;
    let loaded: Value = serde_json::from_str(&planner_load(
        config.to_string_lossy().into(),
        hashes.to_string(),
    ))?;
    eprintln!("{}", json!({"load":loaded}));
    if loaded["state"] != "READY" {
        return Err(loaded.to_string().into());
    }
    if args.iter().any(|x| x == "--selftest") {
        let v: Value = serde_json::from_str(&planner_selftest())?;
        println!("{v}");
        if v["pass"] != true {
            return Err("selftest failed".into());
        }
        return Ok(());
    }
    let input = get("--eval").ok_or("--eval required")?;
    let mode = get("--mode").unwrap_or("ahem-g".into());
    let out = get("--out").ok_or("--out required")?;
    let mut f = std::fs::File::create(out)?;
    for line in std::io::BufReader::new(std::fs::File::open(input)?).lines() {
        let mut r: Value = serde_json::from_str(&line?)?;
        r["mode"] = json!(mode);
        let result = planner_choose(r.to_string());
        let v: Value = serde_json::from_str(&result)?;
        if v.get("error").is_some() {
            return Err(result.into());
        }
        writeln!(f, "{result}")?;
        f.flush()?;
    }
    Ok(())
}
