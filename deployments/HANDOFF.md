# K2 integration handoff

Base Sepolia FULL / Aave profile. Snapshot: block 48319954, 2026-10-19T11:09:56+00:00.

Safe: `0x029869910714D9e09b854a2688311AFd42fBb44E` (OWNER + APPROVER, threshold 2).
Module: `0xA560ec10259657E9bADd297E68286d2838Cd5a8f`.

All six K2-created contracts have exact Sourcify source matches. The manifest includes canonical external contracts, verified-source links, runtime/implementation pins, compiler settings and creation receipts.

`evidence/base-sepolia.json`: E1–E16 complete, 30 rows / 26 unique transactions, all SCRIPT / deterministic / linux-host. Operational passports are published alongside the rows. K9 retains ownership of Golden*.s.sol; K2 did not create those scripts.

## Current chain state

- Cash: 9467.001235 Aave test USDC; reserved: 50 USDC.
- Both vault share balances and both Safe-to-vault allowances are zero after successful emergency unwind.
- Module paused; session cleared; no queued action. Policy epoch 6; action nonce 6.
- Daily gross used: 2093 / 4000 USDC; remaining today: 1907 USDC. A new session does not reset daily usage.
- First four published bill IDs are permanently paid. The fifth bill (50 USDC for the new-bank payee) remains unpaid and reserved.

## Continue the demo

Use the existing cooled Acme and Payroll payee IDs from `evidence/obligations-84532.json`. For another full payment run, commit fresh bill IDs; paid markers cannot be reset by recommitting. Re-arm through a 2-of-2 Safe batch: commit fresh obligations if needed, startSession (expiry at most 24 hours), then resume the current policy epoch. Re-read state after resume because it increments the epoch.

For same-day repeats, keep total new agent gross within the remaining daily allowance (or wait for UTC-day rollover). A smaller investment can preserve budget for fresh payments; owner allocations do not count as agent usage.

The new-bank payee has its own fresh 3,600-second cooling period; its activeAt is recorded in the PAYEE_NEW_BANK row. It was intentionally rejected while cooling.

Keys remain in the original external environment file. Run signing commands only after loading that environment. The verifier itself needs no keys:

```bash
bash scripts/audit_secrets.sh
python3 scripts/verify_evidence.py evidence/base-sepolia.json
```

Payee cooling is a 3,600-second demo setting (production default 24 hours). Sanctions oracle is a TEST LIST; demo-vault yield is SIMULATED.
