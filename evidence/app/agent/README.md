# K17 in-app agent evidence

Highest reached milestone: **M1** (2026-10-19).

`m1.log` and `m1.json` record a real bound-service run in Android process
`app.fusion.wallet:inference` on **OnePlus 11 (CPH2447), Android 16**.
This is the Android host of the edge chroot, not a Linux inference run.
Tecno CL6 adb access was unauthorized at this milestone; no Tecno claim is made.
The fixed list A/H produced A with AHEM-G enabled, no abort, and five forced tokens.
Cold load: 988.37 ms; decode: 423.24 ms; vetoes: 0.

`build-m1.txt`: `./gradlew testDebugUnitTest assembleDebug` passed.
APK inspection verified all five shipped fork hashes against the provisioned
manifest and confirmed no model weights/tokenizer in the APK.
Model files live only in app-private storage and the external provision source.
Native libraries are reproducible ignored build artifacts, not committed binaries.

The Android dry-run runtime and one Run agent now button are implemented but
M2 device validation is still pending. No transaction was broadcast for M1.
No session or guardian key was imported on the OnePlus for this milestone.

Build: `scripts/android_planner/build.sh`, then `build_core.sh`, then Gradle.
Provision to the Tecno after authorization using `provision_model.sh`.
The app button always requests a dry run; no automatic background-loop claim.
