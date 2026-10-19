# Card K12 — In-app guardian Stop (signer) on Base Sepolia (Phase 4)

Fusion Wallet (FinTechathon 2026, Topic F; deadline 2026-10-20 15:59 UTC). The app currently shows "Guardian pause: not wired in monitor build". A working kill switch in the app — one tap, real on-chain pause tx — is one of the strongest Security (30%) demo moments.

## Where you work
- Umbrella worktree `/root/projects/fusion_wallet-wt/K12` (branch `card/K12`). `git submodule update --init src/Fintech_Wallet_Agent`; inside it `git fetch /root/projects/fusion_wallet/src/Fintech_Wallet_Agent fusion-v3-integration && git checkout -B card/K12 FETCH_HEAD` (must include merged K4 + K5). Commit inside the submodule.
- Build: `source /root/projects/fusion_wallet/scripts/edge_android_env.sh` then `./gradlew testDebugUnitTest assembleDebug` in `fusion-core-android`.
- Contracts/ABI: `src/contracts` (guardian `pause` function and its event in `KavachiModule`), `spec/abi/IKavachiModule.json`, `deployments/84532.json` (module `0xA560ec10259657E9bADd297E68286d2838Cd5a8f`, roles.guardian `0x46B8E40EB9767BD46E03C5eCF398C10E1372251E`). Spec §6 (guardian role), §7.10, §11 (`:signer` process, Keystore), §17 (Stop UX).
- Guardian testnet key: `/root/projects/fusion_wallet/.env.local` `GUARDIAN_PRIVATE_KEY` (testnet only). NEVER commit it, log it, or put it in the APK, resources, BuildConfig or assets.

## Owned paths (inside the submodule)
`.../app/fusion/wallet/signer/**`, the `:signer` process entry in `AndroidManifest.xml`, and ONE new method on `MainViewModel` (`suspend fun requestGuardianPause(): Result<String /*txHash*/>`) — coordinate by adding only that method; K11 owns the rest of `ui/**`.

## Build
1. `signer/` running in a separate `:signer` Android process (bound service). It holds the guardian key **wrapped by Android Keystore** (AES-GCM key in AndroidKeyStore encrypts the secp256k1 private key; ciphertext in app-private storage). Import path for the demo: a debug-only `adb shell am broadcast` / intent extra or a one-time import screen that takes the hex key from the operator at runtime — the key must never be in source or the APK.
2. Sign an EIP-1559 tx for the module's guardian pause call (nonce, fees, chainId 84532 from the RPC) with web3j or a minimal Kotlin secp256k1 (BouncyCastle) implementation; broadcast via `eth_sendRawTransaction`; poll the receipt; return the tx hash.
3. Wire `requestGuardianPause()`; the K5 Stop dialog shows the tx hash with a BaseScan link on success, the error on failure.
4. Unit tests: tx encoding/signing against a known vector (e.g. sign with a fixed test key and compare to `cast mktx` output you capture into the test), and that no key material is ever logged.
5. Write `signer/README.md`: threat model (what Keystore wrapping protects and what it does not), how to import the testnet guardian key for the demo, and how the owner resumes after a pause (2-of-2 Safe path).

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K12/src/Fintech_Wallet_Agent/fusion-core-android && source /root/projects/fusion_wallet/scripts/edge_android_env.sh && ./gradlew testDebugUnitTest assembleDebug && ! grep -rniE "0x[0-9a-f]{64}" app/src/main | grep -viE "topic|selector|hash|keccak|event"
```
exits 0, and `signer/README.md` exists. (The supervisor will do the live on-device pause; do NOT broadcast a pause on Base Sepolia yourself — other cards are running live agent txs.)

## Acceptance
- No private key in git, APK, logs, or resources. No Float/Double money. No test weakened.
- Commit inside the submodule on `card/K12`. Do not merge, do not push.
