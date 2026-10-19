//! Android process boundary. The candidate gate and model decision code are the
//! exact host sources, compiled alongside an Android libc++ session adapter.
#[path = "../../src/candidate_grammar.rs"]
pub mod candidate_grammar;
#[path = "../../src/run.rs"]
pub mod run;
pub mod session;

use jni::{JNIEnv, objects::{JClass, JString}, sys::jstring};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path, sync::{Mutex, OnceLock}, time::Instant};

static SESSION: OnceLock<Mutex<Option<session::Session>>> = OnceLock::new();

fn sha(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|_| "FILE_UNAVAILABLE")?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|_| "FILE_UNAVAILABLE")?;
        if n == 0 { break; }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn dispatch(op: &str, input: &str) -> Result<Value, String> {
    let mut slot = SESSION.get_or_init(|| Mutex::new(None)).lock().map_err(|_| "PANIC")?;
    match op {
        "load" => {
            let start = Instant::now();
            let request: Value = serde_json::from_str(input).map_err(|_| "INVALID_REQUEST")?;
            let config = request["configPath"].as_str().ok_or("INVALID_REQUEST")?;
            let libs = request["nativeLibraryDir"].as_str().ok_or("INVALID_REQUEST")?;
            let model_dir = Path::new(config).parent().ok_or("INVALID_REQUEST")?;
            if Path::new(config).file_name().and_then(|s| s.to_str()) != Some("config.json") {
                return Err("INVALID_REQUEST".into());
            }
            let mut actual = serde_json::Map::new();
            for name in ["config.json", "llm_config.json", "llm.mnn", "llm.mnn.weight", "tokenizer.mtok"] {
                actual.insert(name.into(), json!(sha(&model_dir.join(name))?));
            }
            for name in ["libllm.so", "libMNN.so", "libMNN_Express.so", "libMNN_CL.so", "libMNN_Vulkan.so"] {
                actual.insert(name.into(), json!(sha(&Path::new(libs).join(name))?));
            }
            if request["expectedHashes"] != Value::Object(actual.clone()) {
                return Err("HASH_MISMATCH".into());
            }
            *slot = None;
            // MNN's tmp directory is relative. Each process has its own cwd.
            std::env::set_current_dir(model_dir).map_err(|_| "LOAD_FAILED")?;
            let api = session::Api::open(&Path::new(libs).join("libllm.so"))?;
            api.clear();
            let mut s = session::Session { api, config: config.into(), state: "READY", handle: std::ptr::null_mut() };
            s.handle = s.create()?;
            *slot = Some(s);
            Ok(json!({"state":"READY", "coldLoadMs":start.elapsed().as_secs_f64()*1000., "hashes":actual,
                "plannerKind":"mnn+ahem-g@android", "sampling":{"type":"greedy"}}))
        }
        "choose" => {
            let r: run::Request = serde_json::from_str(input).map_err(|_| "INVALID_REQUEST")?;
            // There is no ungated Android execution surface.
            if r.mode != "ahem-g" { return Err("INVALID_REQUEST".into()); }
            run::choose(slot.as_mut().ok_or("NOT_READY")?, r)
        }
        "unload" => { *slot = None; Ok(json!({"state":"UNPROVISIONED"})) }
        _ => Err("INVALID_REQUEST".into()),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_fusion_wallet_inference_NativePlanner_call(
    mut env: JNIEnv, _: JClass, op: JString, input: JString,
) -> jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let op: String = env.get_string(&op).map_err(|_| "JNI_INPUT")?.into();
        let input: String = env.get_string(&input).map_err(|_| "JNI_INPUT")?.into();
        dispatch(&op, &input)
    }));
    let value = match result {
        Ok(Ok(value)) => value,
        Ok(Err(code)) => json!({"ok":false,"error":{"code":code}}),
        Err(_) => json!({"ok":false,"error":{"code":"PANIC"}}),
    };
    env.new_string(value.to_string()).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}
