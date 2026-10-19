# Deployment and evidence operations

Run from the repository root. Requires Foundry 1.8.5 and Python 3; no Python packages are needed.
Never enable shell tracing. Signing scripts take keys only from environment variables.

```bash
set -a
. /root/projects/fusion_wallet/.env.local
set +a
python3 scripts/preflight.py
```

## Anvil rehearsal

Start `anvil --host 127.0.0.1 --port 18545 --chain-id 31337 --silent` in another terminal.
The silent option prevents Anvil from printing its default development keys.
`python3 scripts/deploy.py rehearse --reset-local` resets **only this local Anvil**, deploys official
Safe artifacts plus MockUSDC and two demo vaults, executes threshold-two setup, advances local time,
and exercises payments, investments, queueing, cancellation, deliberate failures, pause and unwind.
It writes `deployments/31337.json`, `evidence/anvil.json`, and local operational passports.

```bash
python3 scripts/verify_evidence.py evidence/anvil.json \
  --manifest deployments/31337.json --rpc-url http://127.0.0.1:18545
```

The committed local manifest/evidence records the rehearsal performed for K2. Local hashes are
not Base Sepolia transactions; restarting Anvil requires a new rehearsal to verify them.

## Base Sepolia

```bash
python3 scripts/deploy.py deploy --chain 84532 --profile FULL
python3 scripts/source_verify.py
python3 scripts/deploy.py setup --chain 84532
python3 scripts/evidence_actions.py before-cooling --chain 84532
# K2's extra FULL-profile owner seed allocation (already executed in the committed deployment):
# python3 scripts/evidence_actions.py owner-allocation --chain 84532
# python3 scripts/evidence_actions.py before-cooling --chain 84532  # add reserve-raid rejection
# Wait until deployments/84532.json payeesActiveAt (chain time).
python3 scripts/evidence_actions.py after-cooling --chain 84532
# Wait for the delayed PAY queue's readyAt; normally 181 seconds.
python3 scripts/evidence_actions.py finish-queue --chain 84532
bash scripts/audit_secrets.sh
python3 scripts/verify_evidence.py evidence/base-sepolia.json
```

Deployment refuses to overwrite an existing chain manifest. Setup refuses to reset an already
completed policy/session. Role gas funding tops up to the authorized targets rather than adding
the same amounts again. If a broadcast fails partway, inspect the manifest, receipts, and ignored
Foundry broadcast files before resuming; do not delete a Base Sepolia manifest to retry blindly.

The FULL profile uses Aave's test USDC and waUSDC plus a KavachiDemoVault. Local rehearsal uses
3,000 MockUSDC, matching §19.6 arithmetic. On Base Sepolia the faucet mints 10,000 Aave USDC
directly to the Safe. To reproduce the runbook's working-cash reserve boundary on Base Sepolia,
K2 allocates 3,000 to waUSDC and 4,000 to the demo vault in an explicit threshold-two owner batch.
These owner allocations are separate from the agent's 1,850-USDC Aave deposit, preserve the
reserve/buffer and exposure caps, and leave 1,150 USDC cash. They are labelled `OWNER_ALLOCATION`
and have no agent decision passport. The owner batch was first rehearsed on an Anvil fork;
see `owner-allocation-rehearsal.txt` and `scripts/rehearse_owner_allocation.py`.
D2/SCALED uses Circle USDC
and two demo vaults; obtain at least 30 Circle USDC for the Safe before setup.

All operational action rows are explicitly `SCRIPT`, `deterministic`, `linux-host`.
The `k2-operational-v1` passports are published by these deployment scripts; they do not claim
to originate from the Rust planner, Android signer, or a model. K9 owns `Golden*.s.sol` and
the integration of full production decision passports. The verifier accepts those passports
when they bind a final `actionDigest`. K2 passports instead publish the full pre-decision action
and domain, allowing reconstruction of the emitted final digest without a circular hash.

The verifier needs no keys, writes nothing, and uses only read-only RPC methods and `cast keccak`.
It validates contract runtime/implementation pins, Safe owners/threshold/module enablement,
receipt status, destination, selector, block, event emitter/topic/fields, passport bytes, and
the action digest. Intentional reverts use `debug_traceTransaction` when supported and otherwise
an `eth_call` replay at the previous block; an unprovable error fails closed.

`collect_evidence.py --help` documents appending individual receipt-backed rows. The bundled
`scripts/event_abi.json` is a source ABI snapshot, so judges need no Solidity build artifacts.
Source verification uses Sourcify without an API key. Project contracts use Solidity 0.8.24,
Cancun, via IR, optimizer 200. The official canonical Safe proxy uses Solidity 0.7.6, Istanbul,
optimizer disabled. The manifest includes both the external canonical contracts and all project
deployments, including the linked policy library and the constructor-created payee registry.

Payee cooling: **3,600 seconds, demo setting; production default 24 hours**.
The sanctions list is a **TEST LIST**. Demo vault yield is **SIMULATED**.
