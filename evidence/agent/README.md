# Autonomous Agent Golden Path (Card K9)

## Overview
This directory contains the autonomous execution evidence for Card K9 on Base Sepolia (`chainId: 84532`).
The agent runtime loop executes:
$$\text{Snapshot} \longrightarrow \text{Candidates} \longrightarrow \text{Planner (AHEM-G)} \longrightarrow \text{Passport} \longrightarrow \text{Preview} \longrightarrow \text{Sign (Session Key)} \longrightarrow \text{Broadcast} \longrightarrow \text{Evidence}$$

Every transaction is signed by the authorized session key (`roles.session`: `0xB52Cb48eC1cc5Ad1171C83F162228e31752F87fA`), verified by the on-chain `KavachiModule` (`0xA560ec10259657E9bADd297E68286d2838Cd5a8f`), and anchors the decision's canonical `decisionHash` directly in on-chain module events (`ActionExecuted` or `ActionQueued`).

## Planner Configuration
Per Card K9 Revision 2, the primary real planner is used:
- **Planner Kind**: `mnn+ahem-g`
- **Host Binary**: `src/fusion-planner/target/release/planner-host`
- **Model**: `LFM2.5-230M-MNN` at `/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m`
- **AHEM-G Mode**: `ENFORCED` with JSON grammar constraint `{"choice":"<LABEL>"}` (5 forced tokens, 0 vetoes)
- **Host**: `linux-host` (aarch64)

## Execution Sequences
1. **Instant PAY**: Bill 0 (8.00 USDC to Acme `0x61dd06cE2fb7c6d691CBE3E2614b184C01C01399`) — instant execution via `KavachiModule.execute()`.
2. **Delayed Queue**: Bill 1 (50.00 USDC to Payroll `0x208Bfee9F2F7A85d03aAcb953b169542a2A0054a`) — queued via `KavachiModule.queueAction()`, enforces 180s timelock.
3. **Delayed Execution**: Once timelock elapses (`now >= readyAt`), executed via `KavachiModule.execute()`.
4. **Autonomous HOLD**: Evaluated when all obligations are met; records DecisionPassport with `HOLD` reason code and emits no transaction.

## Verification
Run:
```bash
python3 scripts/agent/check_golden.py
```
This mechanically checks that:
- $\ge 3$ new `source = "AGENT"` transactions are present in `evidence/agent/golden.json`.
- None of the tx hashes reuse existing K2 script transactions.
- All receipts have status `1` on `https://sepolia.base.org` with `from == roles.session`.
- The on-chain `decisionHash` emitted in module events matches `keccak256(open(passport_file).read())`.
- Execution stdout logs are archived at `evidence/agent/runs/<txhash>.log`.
