# K38 — live HOLD diagnosis and bounded asset-management execution

Device: **OnePlus 11 (CPH2447), Android 16**, the Android host of this edge
chroot. The supported `--local-host` runbook was used. No Tecno execution is
claimed. All production transactions are on **Base Sepolia, chain 84532**.

## Why the live tick held

`K38_initial_hold.json` captures block **48357537**: Safe cash
**9,399.001235 USDC**, unpaid reserve **0**, buffer **500 USDC**, spare cash
**8,899.001235 USDC**, zero vault shares. MNN/AHEM-G returned `{"choice":"H"}`;
there were no non-H candidates. No transaction was signed.

| Finding | Root correction / retained policy |
|---|---|
| `AgentRuntime` read only optional `files/agent/rates.json`; Home's independent Aave rate never reached core. Missing rates meant zero yield. | Read Aave Pool `getReserveData(USDC)` liquidity APR ray at the exact snapshot block, floor to basis points, mark non-simulated. Record raw ray, pool, asset and block hash. No provisioned rate can override live decisions. |
| Surplus enumeration sized `min(spare, exposure headroom)` = 6,000 USDC, above the 2,500 USDC hard action maximum. | Also cap sizing by `hardMaxRaw`, then evaluate economics on the actual bounded amount. This grants no instant authority: >10 USDC still requires the existing tier/route, and the Android signer still only executes INSTANT actions. |
| Asset units were treated as share units; withdrawals used an arbitrary 1% share allowance despite the 0.5% policy ceiling. Actual waUSDC is not 1:1. | Before inference, read exact `convertToShares` and deposit/withdraw previews at the snapshot block. Native core checks the unchanged policy floor/ceiling, uses the deposit floor and a tighter exact-preview withdrawal maximum, and rebuilds IDs/digest before the model sees candidates. No chosen action is edited afterward. |
| The loopback SAM transport inherited serial batch handling, and the public-bills repository rebuilt event history rather than reusing its verified index. The app correctly refused a five-block-old snapshot. | Preserve read-only batching and use `SharedEventIndexes`. The relay bounds upstream bursts by RPC item count, chunks large batches, propagates top-level HTTP/range errors, and retries only idempotent reads. Writes are never retried. The original snapshot freshness checks, signer authority checks, reorg checks and accounting match remain mandatory. Failure captures: `K38_stale_snapshot_refusal.json`, `K38_history_refusal.json`. |
| An initial owner-backed tick put authorization wording in an **untrusted note**. The model's independent note review correctly chose H. | Verified authorization belongs in native-generated candidate facts. State the ordinary operational goal concretely (pay scheduled obligations, restore cash reserves, invest available surplus within policy); retain its independent risk-review stage. `K38_owner_initial_hold.json` and `K38_allocation_review_hold.json` preserve genuine model HOLDs; no output override or deterministic fallback was used. |

Both vaults were already allowlisted with **6,000 USDC** exposure caps. The
session was unpaused and unexpired, with sufficient remaining allowance. No
cooldown prevented an instant deposit. Neither allowlist nor permission limits
needed relaxing.

## Correct profitability HOLD versus the explicitly requested rehearsal

`K38_live_rates_hold.json` captures the corrected path at block **48358781**:
liquidity APR ray **2053914345645953515052735**, **20 bps** after conservative
flooring, non-simulated, valid on-device H, no transaction.

The full-profile yield rule is unchanged:

```
grossRaw = floor(assetsRaw * rateBps * 30 / (10000 * 365))
netRaw   = saturating_sub(grossRaw, 50000)   # 0.05 USDC scenario fee
minimum netRaw = 1000000                   # 1 USDC over 30 days
```

At 20 bps, 10 USDC earns only **0.001643 USDC gross** in 30 days. Even the
2,500 USDC hard cap earns **0.410958 USDC gross**. Claiming a profitable small
allocation, inventing a rate, or lowering the existing economics gate would be
incorrect. The usual yield strategy therefore still holds.

The task's requested <=10 USDC round trip instead uses a separately **2-of-2
owner-signed, gas-paying operational allocation mandate**. Its OWNER transaction
is `0xb4896d5bd3207eee5d05bc2bd956a23d820691c8d4eb7fc80209b432fc353f7a`.
The reviewed proposal and passing local-fork simulation are in
`K38_owner_allocation_proposal.json`; the canonical live receipt and transaction
are in `K38_owner_allocation_receipt.json`.

The owners signed an inert, zero-value Safe call to an owner EOA containing the
strategy commitment. **`configurePolicy` was not called**: the entire module
state was independently verified unchanged at that receipt block. This is an
owner-authorized strategy purpose, not a contract-limit increase or a yield
recommendation. It binds the chain, Safe, module, canonical waUSDC, current
policy hash, session ID, a maximum **10,000,000 raw USDC**, deposit nonce **11**,
withdrawal nonce **12**, and expiry **1792489498**. The action deadline is also
bounded by that expiry. Runtime and independent signer both check the canonical
owner receipt, successful Safe execution, threshold and owners, and exact
calldata. The owner tx hash is bound into candidate reason, set digest and the
canonical passport.

Android still makes a fresh A/H choice for each phase. The host supplies only
the public owner tx hash, transports RPC and holds the canonical session flock.
It never chooses an APP candidate, runs APP inference, or signs an APP transfer.
The existing Android Keystore session envelope is used only by `:signer`.
The withdrawal is sized to actual available vault assets, so any ERC-4626
rounding dust is recorded honestly rather than inventing a full 10 USDC return.

## Confirmed APP round trip

| Operation | Assets | Block | Transaction |
|---|---:|---:|---|
| Deposit | 10 USDC | 48359626 | [`0x7acca495d3bd0da2e62f92d0f6f1f20ef22c1e4bd8dc985be59c37ad4bd2aaaf`](https://sepolia.basescan.org/tx/0x7acca495d3bd0da2e62f92d0f6f1f20ef22c1e4bd8dc985be59c37ad4bd2aaaf) |
| Withdrawal to Safe | 9.999999 USDC | 48359868 | [`0xe6f0105b8cd4a941943112f7507efb1dee6adc47e08aa45852223c7c62cc709a`](https://sepolia.basescan.org/tx/0xe6f0105b8cd4a941943112f7507efb1dee6adc47e08aa45852223c7c62cc709a) |

Both rows are **APP**, planner **`mnn+ahem-g@android`**, with distinct Android
`:inference` and `:signer` processes. Both receipts have status 1, sender
`0xb52cb48ec1cc5ad1171c83f162228e31752f87fa` (the session key), module destination,
and exact `ActionExecuted` events. Canonical passports, raw transactions,
receipts and complete app reports are stored under the hash-named subfolders.
`K38_verification.json` independently verifies the owner authorization, both
MNN choices, all receipt bindings, ERC-4626 events and the rebuilt index.

Afterward: **9,399.001234 USDC Safe cash**, **zero Aave shares**, **zero Aave
assets**. The one raw USDC unit difference is ERC-4626 round-trip rounding;
it is not hidden as a claimed 10 USDC withdrawal. The mandate file was removed
and its two authorized action nonces are consumed.

The original app sends mined successfully but hit a receipt-completion error.
The durable journal and shared flock preserved both signed attempts. Each was
confirmed through the app's **receipt-only reconciliation** mode using its
original MNN report and byte-identical canonical passport. Neither deposit nor
withdrawal was resent. Reports explicitly record
`reconciledAfterReceiptReadFailure`; original request IDs are retained.

The runtime now retries transient receipt reads and can automatically request
restricted reconciliation of an existing journal. `reconcileOnly` with no
journal refuses before any new action verification, key access, signing or
broadcast. A test replays the actual public Aave receipt and proves no second
signature/send; additional tests require the original passport and prove no
recovery request without a journal. The normal stale-decision refusals were
retained; batching owner history before the fresh snapshot avoids spending the
freshness window on serial historical reads.

## Reproduce and verify

From the worktree root:

1. `scripts/android_planner/build.sh` and `scripts/android_planner/build_core.sh`.
   Bindgen now builds/uses the worktree's locked UniFFI tool instead of depending
   on a removable K3 worktree.
2. In the Android project, run the shared-lock `testDebugUnitTest assembleDebug`
   with `ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`.
3. Install the debug APK without clearing app data. Use
   `execute_owner_allocation.py --local-host` for read-only preparation and fork
   simulation. The authorized live run uses `--broadcast --run-app --local-host`.
   `--resume --run-app --local-host` reuses the existing verified mandate without
   reading owner keys or broadcasting another OWNER transaction. If a send has
   already mined but its response was lost, `run_tick.py --local-host --reconcile`
   confirms the journaled attempt without a new planner decision or signature.
4. Run `python3 scripts/build_tx_index.py`,
   `python3 scripts/android_planner/verify_allocation.py`, and
   `python3 scripts/judge/verify_all.py`. Verification is keyless and read-only.

Each APP row must have the Android planner identity, canonical passport hash,
session-key sender, exact `ActionExecuted` topics/data, matching ERC-4626 event,
share bounds, unchanged reserve, correct nonce/epoch/session and correct Safe
cash. The round-trip checker also requires both phases and matching APP/OWNER
provenance in the rebuilt index. Historical SCRIPT deposits are not relabelled.

Frozen arena reruns use `check_allocation_arena.py`; the archive preserves
canonical `evidence/results` paths without overwriting prior published evidence.
No safety check, corpus label, signing allowlist or risk-note guard is relaxed.

## Final validation

- fusion-core: **184 passed**; Android native boundary: **9 passed**; relay: **8 passed**.
- Android: **250 tests**, **0 failures/errors**, **4 inherited opt-in tests skipped**; APK assembled under the shared Gradle lock.
- Judge verifier: **50/50 PASS**; index rebuilt to **40** transactions with honest provenance.
- Fresh final arena: **24/24 stopped**, **ASR 0/30** in both AHEM-G and plain modes; frozen FBR remains **9/30** in both. The locally rebuilt host runtime and exact source/library hashes are recorded in the archive.

See `K38_validation.json` and the captured command logs. Android model files
remain outside the APK. No private keys are in the evidence or source commits.
