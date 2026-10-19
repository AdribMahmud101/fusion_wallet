# Card K3 — fusion-core v4 (Rust engine, Phase 2–3)

You are the Rust engineer for Fusion Wallet (FinTechathon 2026, Topic F, hard deadline 2026-10-20 15:59 UTC).

## Where you work
- Worktree: `/root/projects/fusion_wallet-wt/K3` (branch `card/K3`). `cd` there first. Work ONLY there.
- Spec (read-only): `system architecture/fusion_wallet_architecture.md` — §9 (hashing V4.1), §10 (all of it: migration table, crate layout, manifest, candidates/economics, planner I/O, preview, passport, gates), §7.7 (checks that `preview` mirrors), §7.9 buckets, §8.2 Merkle, §19.5 evidence rows, Appendix B (UniFFI surface) if present.
- Frozen inputs (read-only): `spec/abi/*.json`, `spec/vectors/v4_1.json`, `spec/DECISIONS.md`. Read DECISIONS.md first.
- Existing crate: `src/fusion-core` (v2). Follow the §10.1 migration table exactly: delete float money, the raw-key FFI and the JSON ledger; keep strict serde, panic guard, zeroize, UniFFI setup, `scripts/audit_no_mocks.py`, `scripts/validate.sh`.

## Owned paths (exclusive)
`src/fusion-core/**` only. `src/fusion-planner` belongs to another card (K8, done) — do not edit it; `agent-runner` may call it as an external binary/process or use the `deterministic` planner.

## Priority order (ship in this order, commit after each)
1. `amount.rs`, `types.rs`, `hashing.rs`, `merkle.rs` + `tests/vectors.rs` matching EVERY vector in `spec/vectors/v4_1.json` and the Merkle proofs. (This is what K1/K4 depend on.)
2. `manifest.rs`, `buckets.rs`, `candidates.rs` (§10.4 enumerator + economics, letter labels A–G + H=HOLD per D14), `planner_io.rs` (strict one-letter validator), `passport.rs` (JCS canonical JSON + DecisionHash; must reproduce the passport vector byte-for-byte).
3. `preview.rs` (pure mirror of §7.7), `encode.rs` (module calldata only: execute/queueAction/cancel/pause/unwind + Safe tx), `lib.rs` UniFFI exports.
4. `snapshot.rs` (alloy, one block), `sign.rs` (zeroized key, EIP-1559), `src/bin/agent-runner.rs` (CLI per §10.2; deterministic planner is fine at first).

## Definition of done
```
cd /root/projects/fusion_wallet-wt/K3/src/fusion-core && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked && python3 scripts/audit_no_mocks.py && ! grep -rnE '\bf(32|64)\b' src/amount.rs src/types.rs src/candidates.rs src/buckets.rs src/preview.rs src/encode.rs
```
exits 0, and `src/fusion-core/STATUS.md` states which of priority steps 1–4 are complete, with test names.

## Acceptance checks
- Vector tests read `spec/vectors/v4_1.json` at test time (path relative to the repo root) — never copy expected values into Rust source.
- No f32/f64 money anywhere. No private key crosses FFI. No mocks in non-test code.
- No test weakened or deleted to go green. No secrets committed.
- Commit on `card/K3` after each priority step. Do not merge, do not push.

## Unblocks
K4 (Android wiring, needs UniFFI bindings), K2/K9 (agent-runner produces evidence rows).
