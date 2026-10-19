# Card K10 — Android build toolchain on the edge (TIME-BOXED: 60 minutes)

You are the build engineer. Goal: build the Fusion Wallet Android app **inside this Arch Linux ARM (aarch64) chroot** so Android cards can compile and test here. Hard stop after 60 minutes of work: commit what you have and write the blocker.

## Where you work
- Worktree: `/root/projects/fusion_wallet-wt/K10` (branch `card/K10`). The Android app is the submodule `src/Fintech_Wallet_Agent` (its own git repo; branch `fusion-v3-integration`, AGP 8.11.0, compileSdk 36). It may need `git submodule update --init src/Fintech_Wallet_Agent` inside your worktree.
- You may install system packages with `pacman` and SDK components under `/opt/android-sdk`. That is the point of this card.

## Known facts (verified by the supervisor)
- `pacman -S jdk17-openjdk` is available (17.0.20). `clang` 23 is available. 14 GB RAM, 93 GB disk.
- The Android SDK cmdline-tools, platforms, build-tools jars, d8/R8 and Gradle are Java and run on aarch64.
- **aapt2 is the blocker**: AGP downloads an x86_64 aapt2 from Maven. Use a statically linked aarch64 aapt2 (e.g. the releases of `lzhiyong/android-sdk-tools` on GitHub — verify the binary runs: `aapt2 version`) and set `android.aapt2FromMavenOverride=/opt/android-sdk/aapt2-arm64/aapt2` in the user-level `~/.gradle/gradle.properties` (NOT in the repo). Also replace other x86_64 native tools in build-tools (zipalign, aidl) with arm64 builds if the build needs them.
- The NDK host toolchain is x86_64 and will not run. If the app needs the Rust `fusion-core` `.so` (`aarch64-linux-android`), build it with `cargo` + `rustup target add aarch64-linux-android` using the system `clang --target=aarch64-linux-android<api> --sysroot=<ndk>/toolchains/llvm/prebuilt/linux-x86_64/sysroot` as linker (the sysroot is architecture-independent). Put that wiring in `scripts/edge_android_env.sh`.

## Owned paths (exclusive)
`scripts/edge_android_env.sh`, `scripts/edge_android_build.sh`, `docs/EDGE_ANDROID_BUILD.md`. Do NOT modify app source files. Machine-level installs (`/opt/android-sdk`, `~/.gradle`) are allowed.

## Definition of done
```
bash /root/projects/fusion_wallet-wt/K10/scripts/edge_android_build.sh
```
runs `./gradlew assembleDebug` for the app module and exits 0, printing the APK path; `docs/EDGE_ANDROID_BUILD.md` records every install step, versions, and the aapt2 source + SHA-256.

## If blocked at 60 minutes
Commit the scripts and `docs/EDGE_ANDROID_BUILD.md` with the exact failing command and error. Do not keep trying.

## Acceptance
- No secrets. Do not commit binaries. Commit on `card/K10`. Do not merge, do not push.
