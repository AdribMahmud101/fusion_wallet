use fusion_planner::*;
#[test]
fn missing_symbol() {
    // libc is a real, loadable library on this host, with no AHEM exports.
    let path = [
        "/usr/lib/libc.so.6",
        "/lib/aarch64-linux-gnu/libc.so.6",
        "/lib/x86_64-linux-gnu/libc.so.6",
    ]
    .into_iter()
    .map(std::path::Path::new)
    .find(|p| p.exists())
    .expect("host libc fixture");
    assert!(unsafe { libloading::Library::new(path) }.is_ok());
    assert_eq!(
        session::Api::open(path).err().as_deref(),
        Some("AHEM_UNAVAILABLE")
    );
}
#[test]
#[cfg_attr(
    not(planner_model_available),
    ignore = "NOT_RUN: FUSION_MODEL_DIR unset"
)]
fn lifecycle_and_ban_all() {
    let Ok(dir) = std::env::var("FUSION_MODEL_DIR") else {
        eprintln!("NOT_RUN AHEM-G-04/05: FUSION_MODEL_DIR unset");
        return;
    };
    let config = std::path::Path::new(&dir).join("config.json");
    let hashes = measured_hashes(&config).unwrap();
    let wrong = planner_load(config.to_string_lossy().into(), "{}".into());
    assert!(wrong.contains("HASH_MISMATCH"));
    let loaded = planner_load(config.to_string_lossy().into(), hashes.to_string());
    assert!(loaded.contains("READY"), "{loaded}");
    let result = planner_selftest();
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["pass"], true, "{result}");
    planner_unload();
}
