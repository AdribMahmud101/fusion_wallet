#!/bin/bash
# Edge Android Build Environment Setup for Arch Linux ARM (aarch64)

export ANDROID_HOME="${ANDROID_HOME:-/opt/android-sdk}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_HOME/ndk/27.0.12077973}"
export NDK_HOME="$ANDROID_NDK_ROOT"

# Ensure JDK 17 is in PATH and JAVA_HOME is set
if [ -d "/usr/lib/jvm/java-17-openjdk" ]; then
    export JAVA_HOME="/usr/lib/jvm/java-17-openjdk"
elif [ -d "/usr/lib/jvm/default-runtime" ]; then
    export JAVA_HOME="/usr/lib/jvm/default-runtime"
fi

# Ensure gradle user home has the aapt2 override
# One shared Gradle home for every worker: per-account HOMEs each spawned
# their own daemons and exhausted the phone's RAM (load 155, swap full).
export GRADLE_USER_HOME=/root/.gradle
mkdir -p "$GRADLE_USER_HOME"
if ! grep -q "android.aapt2FromMavenOverride" "$GRADLE_USER_HOME/gradle.properties" 2>/dev/null; then
    echo "android.aapt2FromMavenOverride=$ANDROID_HOME/aapt2-arm64/aapt2" >> "$GRADLE_USER_HOME/gradle.properties"
fi
if [ "$GRADLE_USER_HOME" != "/root/.gradle" ]; then
    mkdir -p /root/.gradle
    if ! grep -q "android.aapt2FromMavenOverride" "/root/.gradle/gradle.properties" 2>/dev/null; then
        echo "android.aapt2FromMavenOverride=$ANDROID_HOME/aapt2-arm64/aapt2" >> "/root/.gradle/gradle.properties"
    fi
fi

# Toolchain wiring for Rust compilation target aarch64-linux-android
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ANDROID_HOME/bin/aarch64-linux-android-clang"
export CC_aarch64_linux_android="$ANDROID_HOME/bin/aarch64-linux-android-clang"
export CXX_aarch64_linux_android="$ANDROID_HOME/bin/aarch64-linux-android-clang++"

# Path configuration
export PATH="$ANDROID_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/build-tools/35.0.0:${JAVA_HOME:+$JAVA_HOME/bin:}$PATH"
