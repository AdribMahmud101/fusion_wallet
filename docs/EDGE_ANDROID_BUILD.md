# Android Build Toolchain on the Edge (aarch64)

This document records the complete setup, components, versions, installation steps, and workarounds required to build the Fusion Wallet Android application inside an Arch Linux ARM (`aarch64`) chroot environment.

---

## 1. Environment Summary

- **Host Architecture:** `aarch64` (ARM 64-bit)
- **OS / Distribution:** Arch Linux ARM (chroot)
- **Kernel:** Linux 5.15.180
- **Android Target SDK:** 36 (compileSdk 36, minSdk 24, targetSdk 36)
- **Android Gradle Plugin (AGP):** 8.11.0
- **Gradle Version:** 8.13
- **NDK Version:** 27.0.12077973

---

## 2. Packages and Components Installed

### 2.1 Arch Linux System Packages (`pacman`)

| Package | Version | Purpose |
|---|---|---|
| `jdk17-openjdk` | 17.0.20.1+1 | Java 17 Runtime and Development Kit for Gradle and Android SDK tools |
| `clang` | 23.1.3-3 | Host C/C++ compiler used for cross-compilation |
| `lld` | 23.1.3-3 | LLVM Linker (`ld.lld`) used for linking Android native binaries |
| `llvm-libs` | 23.1.3-3 | LLVM runtime libraries |
| `compiler-rt` | 23.1.3-3 | Clang compiler runtime libraries |
| `rustup` | 1.29.1-1 | Rust toolchain installer (replaces arch `rust` package to allow cross targets) |

**Installation Commands:**
```bash
# Update pacman sync DB
pacman -Sy

# Install OpenJDK 17
pacman -S --noconfirm jdk17-openjdk

# Install Clang, LLD, compiler-rt
pacman -S --noconfirm clang lld llvm-libs compiler-rt

# Replace system rust package with rustup to support Android targets
pacman -Rdd --noconfirm rust
pacman -S --noconfirm rustup
```

### 2.2 Rust Toolchain

- **Channel:** `stable` (rustc 1.99.1)
- **Targets:**
  - `aarch64-unknown-linux-gnu` (default host)
  - `aarch64-linux-android` (cross-compilation target)

**Installation Commands:**
```bash
rustup default stable
rustup target add aarch64-linux-android
```

---

## 3. Android SDK Setup (`/opt/android-sdk`)

### 3.1 Command-Line Tools
Downloaded official command-line tools package:
- **Source:** `https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip`
- **Installed Version:** `cmdline-tools` 12.0
- **Location:** `/opt/android-sdk/cmdline-tools/latest`

```bash
mkdir -p /opt/android-sdk/downloads
curl -L -o /opt/android-sdk/downloads/cmdline-tools.zip \
  https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
mkdir -p /opt/android-sdk/cmdline-tools
unzip -q -o /opt/android-sdk/downloads/cmdline-tools.zip -d /opt/android-sdk/cmdline-tools
mv /opt/android-sdk/cmdline-tools/cmdline-tools /opt/android-sdk/cmdline-tools/latest
```

### 3.2 SDK Platforms & Build-Tools
Using `sdkmanager` (runs on Java 17 and is CPU-architecture independent):
- `platforms;android-36` (Android API 36 Platform)
- `build-tools;35.0.0` (Android SDK Build-Tools 35.0.0)
- `ndk;27.0.12077973` (Android NDK r27)

```bash
yes | /opt/android-sdk/cmdline-tools/latest/bin/sdkmanager --licenses
yes | /opt/android-sdk/cmdline-tools/latest/bin/sdkmanager \
  "platforms;android-36" \
  "build-tools;35.0.0" \
  "ndk;27.0.12077973"
```

---

## 4. Resolving ARM64 Binary Incompatibilities

Google's Android SDK build-tools and Maven-distributed AGP binaries only provide x86_64 host binaries for native tools (`aapt2`, `aidl`, `zipalign`, etc.). On an aarch64 host, executing these yields `cannot execute binary file`.

### 4.1 Statically Linked aarch64 `aapt2` & Build-Tools
We used pre-compiled, statically linked aarch64 Android SDK binaries from the `lzhiyong/android-sdk-tools` project:

- **Source Release:** [lzhiyong/android-sdk-tools v35.0.2](https://github.com/lzhiyong/android-sdk-tools/releases/tag/35.0.2)
- **Download URL:** `https://github.com/lzhiyong/android-sdk-tools/releases/download/35.0.2/android-sdk-tools-static-aarch64.zip`
- **ZIP File SHA-256:**
  `db1cea2c4454d5f9c5a802646b2d1cf560b4ee7badbe23e51ab8e1881bb50fc2`
- **Installed AAPT2 Path:** `/opt/android-sdk/aapt2-arm64/aapt2`
- **AAPT2 Binary SHA-256:**
  `db737b3bbee99dd8a9e56108cc05d032597874d3f8ac557a69f3c5de5efaf57f`
- **AAPT2 Binary Version:**
  `Android Asset Packaging Tool (aapt) 2.19-T.R4T3.3a47df1-16a215b-168e4a6`

**Installation & Configuration:**
```bash
mkdir -p /opt/android-sdk/aapt2-arm64 /opt/android-sdk/downloads
curl -L -o /opt/android-sdk/downloads/android-sdk-tools-static-aarch64.zip \
  https://github.com/lzhiyong/android-sdk-tools/releases/download/35.0.2/android-sdk-tools-static-aarch64.zip
unzip -q -o /opt/android-sdk/downloads/android-sdk-tools-static-aarch64.zip -d /opt/android-sdk/downloads/unpacked
cp /opt/android-sdk/downloads/unpacked/build-tools/aapt2 /opt/android-sdk/aapt2-arm64/aapt2
chmod +x /opt/android-sdk/aapt2-arm64/aapt2

# Also replace x86_64 binaries in build-tools 35.0.0
cp -f /opt/android-sdk/downloads/unpacked/build-tools/* /opt/android-sdk/build-tools/35.0.0/
chmod +x /opt/android-sdk/build-tools/35.0.0/*

# Configure user-level gradle.properties (both root and user homes)
mkdir -p /root/.gradle ~/.gradle
echo "android.aapt2FromMavenOverride=/opt/android-sdk/aapt2-arm64/aapt2" >> /root/.gradle/gradle.properties
echo "android.aapt2FromMavenOverride=/opt/android-sdk/aapt2-arm64/aapt2" >> ~/.gradle/gradle.properties
```

### 4.2 NDK `llvm-strip`
AGP attempts to invoke `llvm-strip` from the NDK toolchain during `:app:stripDebugDebugSymbols`. The NDK host binary is x86_64. We replaced it with a symlink to the system's aarch64 `llvm-strip`:
```bash
ln -sf /usr/bin/llvm-strip /opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip
```

---

## 5. Cross-Compiling Rust Native Core (`libfusion_core.so`)

The app depends on `libfusion_core.so` (UniFFI Kotlin bindings). Since the NDK host binaries cannot run on aarch64, we cross-compile with host Clang using the architecture-independent NDK sysroot.

### 5.1 Clang Wrapper & Linker Configuration
Created `/opt/android-sdk/bin/aarch64-linux-android-clang`:
```bash
#!/bin/bash
exec clang \
  --target=aarch64-linux-android28 \
  --sysroot=/opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64/sysroot \
  -fuse-ld=lld \
  -L/opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/aarch64 \
  "$@"
```

> [!NOTE]
> Android API level 28 (`aarch64-linux-android28`) is specifically required because the `aws-lc-sys` crate (cryptographic dependency of `alloy`) uses `getentropy()`, which was introduced in Android Bionic at API 28. Compiling against API 24 causes `call to undeclared function 'getentropy'`.

### 5.2 Clang Builtins Symlinks
Clang 23 automatically resolves compiler builtins from `/usr/lib/clang/23/lib/aarch64-unknown-linux-android28/libclang_rt.builtins.a`. We symlinked the NDK's prebuilt builtins and unwind libraries:
```bash
mkdir -p /usr/lib/clang/23/lib/aarch64-unknown-linux-android28
ln -sf /opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/libclang_rt.builtins-aarch64-android.a \
  /usr/lib/clang/23/lib/aarch64-unknown-linux-android28/libclang_rt.builtins.a
ln -sf /opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/aarch64/libunwind.a \
  /usr/lib/clang/23/lib/aarch64-unknown-linux-android28/libunwind.a
ln -sfn /usr/lib/clang/23/lib/aarch64-unknown-linux-android28 \
  /usr/lib/clang/23/lib/aarch64-unknown-linux-android
```

### 5.3 Rust Compilation
```bash
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=/opt/android-sdk/bin/aarch64-linux-android-clang
export CC_aarch64_linux_android=/opt/android-sdk/bin/aarch64-linux-android-clang

cd src/Fintech_Wallet_Agent/fusion-core
cargo build --target aarch64-linux-android --release

# Copy the built cdylib to the Android app jniLibs directory
mkdir -p ../fusion-core-android/app/src/main/jniLibs/arm64-v8a
cp target/aarch64-linux-android/release/libfusion_core.so \
  ../fusion-core-android/app/src/main/jniLibs/arm64-v8a/libfusion_core.so
```

---

## 6. Build and Verification Scripts

### 6.1 `scripts/edge_android_env.sh`
Sets up environment variables:
- `ANDROID_HOME=/opt/android-sdk`
- `JAVA_HOME=/usr/lib/jvm/java-17-openjdk`
- `GRADLE_USER_HOME=/root/.gradle`
- `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=/opt/android-sdk/bin/aarch64-linux-android-clang`
- `PATH` augmented with SDK, build-tools, and Java binaries.

### 6.2 `scripts/edge_android_build.sh`
Builds the entire application and verifies the APK output:
```bash
bash /root/projects/fusion_wallet-wt/K10/scripts/edge_android_build.sh
```

**Build Output:**
- **APK Location:** `/root/projects/fusion_wallet-wt/K10/src/Fintech_Wallet_Agent/fusion-core-android/app/build/outputs/apk/debug/app-debug.apk`
- **APK Size:** ~22 MB
- **Bundled Architectures:** `lib/arm64-v8a/libfusion_core.so`
