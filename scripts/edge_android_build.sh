#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Source environment
source "$SCRIPT_DIR/edge_android_env.sh"

APP_REPO="$REPO_ROOT/src/Fintech_Wallet_Agent"
if [ ! -d "$APP_REPO" ] || [ ! -f "$APP_REPO/fusion-core-android/gradlew" ]; then
    echo "Initializing Fintech_Wallet_Agent submodule..."
    git -C "$REPO_ROOT" submodule update --init src/Fintech_Wallet_Agent
fi

# Build Rust fusion-core library if missing or outdated
RUST_CORE_DIR="$APP_REPO/fusion-core"
JNI_LIBS_DIR="$APP_REPO/fusion-core-android/app/src/main/jniLibs/arm64-v8a"
SO_TARGET="$JNI_LIBS_DIR/libfusion_core.so"

if [ ! -f "$SO_TARGET" ]; then
    echo "Building fusion-core cdylib for aarch64-linux-android..."
    mkdir -p "$JNI_LIBS_DIR"
    (
        cd "$RUST_CORE_DIR"
        cargo build --target aarch64-linux-android --release
        cp target/aarch64-linux-android/release/libfusion_core.so "$SO_TARGET"
    )
fi

echo "Building Android application with Gradle..."
ANDROID_DIR="$APP_REPO/fusion-core-android"
(
    cd "$ANDROID_DIR"
    ./gradlew assembleDebug --no-configuration-cache
)

APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
if [ ! -f "$APK_PATH" ]; then
    echo "Error: APK not found at $APK_PATH" >&2
    exit 1
fi

echo "=========================================================="
echo "Android debug build successful!"
echo "APK Path: $APK_PATH"
echo "APK Size: $(ls -lh "$APK_PATH" | awk '{print $5}')"
echo "=========================================================="
