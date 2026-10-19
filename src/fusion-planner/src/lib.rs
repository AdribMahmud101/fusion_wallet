#[cfg(feature = "bindings")]
uniffi::setup_scaffolding!();
pub mod candidate_grammar;
pub mod run;
pub mod session;
use serde_json::{Value, json};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
static SESSION: OnceLock<Mutex<Option<session::Session>>> = OnceLock::new();
fn slot() -> &'static Mutex<Option<session::Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}
pub fn lib_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MNN_LIB_DIR") {
        return p.into();
    }
    let supplied = PathBuf::from(env!("PLANNER_MNN_DIR"));
    // This checkout supplied x86-64 binaries to an aarch64 host. Use only
    // an explicitly built local native fork when the default is incompatible.
    let rebuilt = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/mnn-host");
    if cfg!(target_arch = "aarch64")
        && std::fs::read(supplied.join("libllm.so"))
            .ok()
            .is_some_and(|b| b.get(18..20) == Some(&[62, 0]))
        && rebuilt.join("libllm.so").exists()
    {
        rebuilt
    } else {
        supplied
    }
}
pub fn sha(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut b = [0u8; 65536];
    loop {
        let n = f.read(&mut b).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn measured_hashes(config: &Path) -> Result<Value, String> {
    let dir = config.parent().ok_or("LOAD_FAILED")?;
    let mut m = serde_json::Map::new();
    for name in [
        "llm.mnn",
        "llm.mnn.weight",
        "tokenizer.mtok",
        "llm_config.json",
        "config.json",
    ] {
        m.insert(name.into(), json!(sha(&dir.join(name))?));
    }
    for name in ["libllm.so", "libMNN.so"] {
        m.insert(name.into(), json!(sha(&lib_dir().join(name))?));
    }
    Ok(Value::Object(m))
}
fn guard(f: impl FnOnce() -> Result<Value, String>) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(v)) => v.to_string(),
        Ok(Err(e)) => json!({"ok":false,"error":{"code":e}}).to_string(),
        Err(_) => json!({"ok":false,"error":{"code":"PANIC"}}).to_string(),
    }
}
#[cfg_attr(feature = "bindings", uniffi::export)]
pub fn planner_version() -> String {
    guard(|| {
        Ok(
            json!({"api":"1.0","ahemAbi":"candidate-adapter-1","mnnBuild":"12356f7","libllmSha256":sha(&lib_dir().join("libllm.so"))?,"libMnnSha256":sha(&lib_dir().join("libMNN.so"))?}),
        )
    })
}
#[cfg_attr(feature = "bindings", uniffi::export)]
pub fn planner_load(config_path: String, expected_hashes_json: String) -> String {
    guard(|| {
        let start = std::time::Instant::now();
        let expected: Value =
            serde_json::from_str(&expected_hashes_json).map_err(|_| "HASH_MISMATCH")?;
        if Path::new(&config_path).file_name().and_then(|n| n.to_str()) != Some("config.json") {
            return Err("HASH_MISMATCH".into());
        }
        let actual = measured_hashes(Path::new(&config_path)).map_err(|_| "HASH_MISMATCH")?;
        if expected != actual {
            return Err("HASH_MISMATCH".into());
        }
        let mut slot = slot().lock().map_err(|_| "PANIC")?;
        // Drop the previous session before creating its replacement.
        *slot = None;
        let api = session::Api::open(&lib_dir().join("libllm.so"))?;
        api.clear();
        let mut s = session::Session {
            api,
            config: config_path,
            state: "READY",
            handle: std::ptr::null_mut(),
        };
        let h = s.create()?;
        s.handle = h;
        *slot = Some(s);
        Ok(
            json!({"state":"READY","coldLoadMs":start.elapsed().as_secs_f64()*1000.,"hashes":actual}),
        )
    })
}
#[cfg_attr(feature = "bindings", uniffi::export)]
pub fn planner_choose(request_json: String) -> String {
    guard(|| {
        let r = serde_json::from_str(&request_json).map_err(|_| "INVALID_REQUEST")?;
        let mut lock = slot().lock().map_err(|_| "PANIC")?;
        run::choose(lock.as_mut().ok_or("NOT_READY")?, r)
    })
}
#[cfg_attr(feature = "bindings", uniffi::export)]
pub fn planner_unload() -> String {
    guard(|| {
        let mut s = slot().lock().map_err(|_| "PANIC")?;
        if let Some(x) = s.take() {
            x.api.clear();
        }
        Ok(json!({"state":"UNPROVISIONED"}))
    })
}
#[cfg_attr(feature = "bindings", uniffi::export)]
pub fn planner_selftest() -> String {
    guard(|| {
        let missing = session::Api::open(&lib_dir().join("libMNN.so"))
            .err()
            .as_deref()
            == Some("AHEM_UNAVAILABLE");
        if slot().lock().map_err(|_| "PANIC")?.is_none() {
            return Ok(
                json!({"pass":false,"details":{"AHEM-G-03":missing,"AHEM-G-04":"NOT_RUN: model not loaded","AHEM-G-05":"NOT_RUN: model not loaded"}}),
            );
        }
        let mut values = vec![];
        let mut cleanup = true;
        for labels in [json!(["A", "H"]), json!(["B", "H"])] {
            let v:Value=serde_json::from_str(&planner_choose(json!({"prompt":format!("Select {} from the available options. H is hold.",labels[0].as_str().unwrap()),"labels":labels,"candidateSetDigest":"selftest","mode":"ahem-g"}).to_string())).map_err(|_|"PANIC")?;
            values.push(v);
            let lock = slot().lock().map_err(|_| "PANIC")?;
            let api = &lock.as_ref().ok_or("NOT_READY")?.api;
            cleanup &= unsafe {
                (api.ahem_is_gate_callback_registered)() == 0
                    && (api.ahem_is_grammar_complete)() == 0
            };
        }
        let lifecycle = cleanup
            && values
                .iter()
                .zip([vec!["A", "H"], vec!["B", "H"]])
                .all(|(v, l)| {
                    v["label"].as_str().is_some_and(|s| l.contains(&s))
                        && v["rawValid"] == true
                        && v["ahem"]["g"]["aborted"] == false
                });
        let ban:Value=serde_json::from_str(&planner_choose(json!({"prompt":"Choose A","labels":["A","H"],"candidateSetDigest":"ban","mode":"ahem-g","banAll":true,"maxRetries":8}).to_string())).map_err(|_|"PANIC")?;
        let exhausted = ban["label"] == "H"
            && ban["ahem"]["g"]["aborted"] == true
            && ban["ahem"]["g"]["vetoes"].as_u64().is_some_and(|x| x > 0)
            && ban["timings"]["totalMs"]
                .as_f64()
                .is_some_and(|x| x < 25000.);
        Ok(
            json!({"pass":missing&&lifecycle&&exhausted,"details":{"AHEM-G-03":missing,"AHEM-G-04":lifecycle,"AHEM-G-05":exhausted},"runs":values,"banAll":ban}),
        )
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn panic_is_json_error() {
        let response = super::guard(|| panic!("test panic boundary"));
        let value: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"ok":false,"error":{"code":"PANIC"}})
        );
    }
}
