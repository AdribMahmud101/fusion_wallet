fn main() {
    println!("cargo:rerun-if-env-changed=FUSION_MODEL_DIR");
    println!("cargo:rustc-check-cfg=cfg(planner_model_available)");
    if std::env::var_os("FUSION_MODEL_DIR").is_some() {
        println!("cargo:rustc-cfg=planner_model_available");
    }
    println!("cargo:rerun-if-env-changed=MNN_LIB_DIR");
    let dir = std::env::var("MNN_LIB_DIR")
        .unwrap_or("/root/projects/victor_fusion/ahem/third_party/MNN/build".into());
    println!("cargo:rustc-env=PLANNER_MNN_DIR={dir}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let status = std::process::Command::new(std::env::var("CXX").unwrap_or("c++".into()))
        .args([
            "-std=c++17",
            "-fPIC",
            "-c",
            "src/tokenizer_bridge.cpp",
            "-o",
        ])
        .arg(out.join("tokenizer_bridge.o"))
        .status()
        .expect("C++ tokenizer bridge compiler");
    assert!(status.success());
    assert!(
        std::process::Command::new("ar")
            .arg("crs")
            .arg(out.join("libplanner_tokenizer.a"))
            .arg(out.join("tokenizer_bridge.o"))
            .status()
            .unwrap()
            .success()
    );
    println!("cargo:rerun-if-changed=src/tokenizer_bridge.cpp");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=planner_tokenizer");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
