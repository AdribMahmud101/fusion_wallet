use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ndk = env::var("ANDROID_NDK_ROOT").expect("source edge_android_env.sh");
    let sysroot = format!("{ndk}/toolchains/llvm/prebuilt/linux-x86_64/sysroot");
    let cxx = env::var("CXX_aarch64_linux_android").unwrap();
    assert!(Command::new(cxx).args(["-std=c++17", "-fPIC", "-O2", "-c", "../src/tokenizer_bridge.cpp"])
        .arg(format!("-I{sysroot}/usr/include/c++/v1"))
        .arg("-o").arg(out.join("bridge.o")).status().unwrap().success());
    assert!(Command::new("ar").arg("crs").arg(out.join("libplanner_bridge.a"))
        .arg(out.join("bridge.o")).status().unwrap().success());
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=planner_bridge");
    println!("cargo:rustc-link-arg={sysroot}/usr/lib/aarch64-linux-android/libc++_static.a");
    println!("cargo:rustc-link-arg={sysroot}/usr/lib/aarch64-linux-android/libc++abi.a");
    println!("cargo:rerun-if-changed=../src/tokenizer_bridge.cpp");
}

