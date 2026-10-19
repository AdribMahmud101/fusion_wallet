# Card K0 — Contracts freeze (Phase 1)

You are the Phase-1 spec architect for Fusion Wallet (FinTechathon 2026, Topic F, deadline 2026-10-20 15:59 UTC).
You WRITE frozen interface artifacts. You do not invent fields.

## Where you work
- Your git worktree: `/root/projects/fusion_wallet-wt/K0` (branch `card/K0`). Work ONLY here. `cd` there first.
- Source of truth (read-only): `system architecture/fusion_wallet_architecture.md` in that worktree.
  - §24 Appendix A — Solidity interface (verbatim source of the ABI)
  - §9 — Hashing contract V4.1
  - §7.4–7.12 — policy, actions, checks, events/errors (semantics)
- Toolchain: `forge`/`cast` 1.8.5 on PATH; solc 0.8.24.

## Mission
Freeze the on-chain interface and V4.1 hashing as machine-checkable files, so K1 (Solidity), K3 (Rust) and K4 (Kotlin)
code against ONE ABI and ONE set of digests. K1's `GenVectors.s.sol` must later reproduce your vectors byte-for-byte (D18).

## Owned paths (exclusive)
`spec/**` — write nothing anywhere else. Do not edit the architecture doc.

## Deliverables
1. `spec/reference/` — a tiny Foundry project (foundry.toml pins solc 0.8.24; forge-std vendored under `spec/reference/lib` or installed with `forge install --no-git`).
   - `src/IKavachiModule.sol` — Appendix A copied verbatim (all interfaces, structs, events, errors).
   - `src/HashV41.sol` — reference library implementing EVERY §9 formula literally: TAG, PolicyHash, vaultLimitsHash (sorted), SessionId, MandateHash, ActionDigest, CandidateId (ActionDigest with decisionHash = 0), ObligationLeaf (double hash via bytes.concat), sorted-pair Merkle root and proof verification.
   - `script/GenVectors.s.sol` — emits the vector JSON.
   - `test/HashV41.t.sol` — re-checks every vector and the Merkle proofs.
2. `spec/abi/IKavachiModule.json` (plus one JSON per other interface in Appendix A), from `forge inspect … abi`.
3. `spec/vectors/v4_1.json` — inputs AND expected outputs for at least:
   one Action per S kind (§7.6), a REBALANCE-shaped Action, empty commitment, single-leaf proof, 3-leaf proof,
   PolicyHash, SessionId (with and without session), MandateHash, CandidateId, and one passport JSON → DecisionHash
   (RFC 8785 JCS; include the exact canonical string). Encoding: uint256 as base-10 strings, hex lowercase 0x-prefixed, addresses EIP-55.
4. `spec/check_freeze.sh` — rebuilds ABI + vectors from scratch into a temp dir, runs `forge test` in spec/reference, diffs against committed files, exits non-zero on any drift. Deterministic.
5. `spec/DECISIONS.md` — every ambiguity you resolved and why (cite § numbers).
6. `spec/README.md` — what is frozen, how to verify, and the rule: changing any vector requires a version bump (V4.1 → V4.2).

## Definition of done
`cd /root/projects/fusion_wallet-wt/K0 && bash spec/check_freeze.sh` exits 0.

## Acceptance checks
- ABI names, argument order and types match Appendix A exactly.
- TAG strings exactly: KAVACHI_POLICY_V4_1, KAVACHI_SESSION_V4_1, KAVACHI_MANDATE_V4_1, KAVACHI_ACTION_V4_1, KAVACHI_CANDIDATE_V4_1.
- `abi.encode` everywhere (never encodePacked) except the ObligationLeaf `bytes.concat` double hash as written.
- No secrets, no private keys (use well-known test addresses like 0x…dEaD or vm.addr(1..n) only).
- Commit on `card/K0` with a clear message when done. Do not merge, do not push.

## Unblocks
K1 (contracts) and K3 (fusion-core) start the moment this lands.
