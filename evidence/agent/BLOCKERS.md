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
- Calling `queueAction` with `deadline = now + delaySeconds + 300` succeeds and emits `ActionQueued`.
