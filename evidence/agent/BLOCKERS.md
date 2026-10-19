# Known Blockers & Library Workarounds (Card K9)

## Blocker 1: Hardcoded Action Deadline for Queued Actions in `src/fusion-core/src/candidates.rs`

### Severity
High (blocks on-chain `queueAction` execution if not worked around).

### Details
In `src/fusion-core/src/candidates.rs` (line 228), candidate action generation hardcodes `deadline = now + 120`:
```rust
let action = Action {
    ...
    deadline: now + 120,
    ...
};
```
However, `src/contracts/src/KavachiPolicyLib.sol` (lines 49–56) enforces strict timelock-relative bounds on `action.deadline` during `queueAction`:
```solidity
if (queueing) {
    if (
        a.deadline < block.timestamp + policy.delaySeconds + 30
            || a.deadline > block.timestamp + policy.delaySeconds + 900
    ) revert I.InvalidDeadline();
}
```
When `policy.delaySeconds == 180`, `a.deadline` must be at least `block.timestamp + 210`. The hardcoded value of `now + 120` fails with revert `0x769d11e4` (`InvalidDeadline()`).

### Workaround in `src/fusion-core/src/bin/agent-runner.rs`
Per the Card K9 instructions ("Do not edit other `src/fusion-core/src/**` files; if a library bug blocks you, write `evidence/agent/BLOCKERS.md` with the failing test and work around it in the binary"):
1. When `choice.route == Route::Queue && !snapshot.queue.exists`:
   - Adjust `choice.action.deadline = snapshot.block_timestamp + snapshot.policy.delay_seconds + 300`.
   - Recompute `choice.id` and update the candidate in `candidate_set.candidates` before generating the Decision Passport.
2. When `choice.route == Route::Queue && snapshot.queue.exists`:
   - Synchronize `choice.action` with `snapshot.queue.action` so that the action digest and deadline match the queued on-chain state exactly.

### Test Case
- Calling `queueAction` with `deadline = now + 120` reverts with `0x769d11e4` (`InvalidDeadline()`).
- Calling `queueAction` with `deadline = now + delaySeconds + 600` succeeds and emits `ActionQueued`.

## Blocker 2: ActionKind Enum Inversion in `src/fusion-core/src/snapshot.rs`

### Severity
Critical (blocks queued obligation execution if not worked around).

### Details
In `src/contracts/src/IKavachiModule.sol`, `enum ActionKind` is defined as:
```solidity
enum ActionKind {
    DEPOSIT,        // 0
    WITHDRAW,       // 1
    REBALANCE,      // 2
    PAY_OBLIGATION  // 3
}
```
However, in `src/fusion-core/src/snapshot.rs` (lines 211–216), `convert_action` maps:
```rust
let kind = match a.kind {
    0 => ActionKind::Deposit,
    1 => ActionKind::Withdraw,
    2 => ActionKind::PayObligation,
    3 => ActionKind::Rebalance,
    ...
};
```
When `getQueue()` returns an active queued bill with Solidity enum value `3` (`PAY_OBLIGATION`), `snapshot.rs` erroneously parses it as `ActionKind::Rebalance`. As a result, calculating the action digest or attempting to execute the action results in `UnsupportedAction` or `BadDigest`.

### Workaround in `src/fusion-core/src/bin/agent-runner.rs`
When synchronizing `choice.action` from `snapshot.queue.action` during queue execution (`choice.route == Route::Queue && snapshot.queue.exists`):
1. Check if `q_act.kind == ActionKind::Rebalance` and remap it to `ActionKind::PayObligation`.
2. Recompute candidate ID and preserve the original locked `decisionHash` from the queued action.

