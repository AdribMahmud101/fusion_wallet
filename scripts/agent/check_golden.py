#!/usr/bin/env python3
"""
scripts/agent/check_golden.py

Verification script for Card K9: Autonomous agent golden path on Base Sepolia.
Read-only: checks on-chain receipts, session key authorization, decisionHash event anchoring,
and passport keccak consistency without using private keys.
"""

import sys
import json
import os
import urllib.request
from pathlib import Path

RPC_URL = os.environ.get("BASE_SEPOLIA_RPC", "https://sepolia.base.org")

ACTION_EXECUTED_TOPIC0 = "0xec3c5fb2a9f98228404a340d65a7da26d4bb846738b0d1f39b0de18f0a0154e8"
ACTION_QUEUED_TOPIC0 = "0x97c854f499aef695661e30437a03bb608a96556ee775d57648e7217d96c449cd"

def keccak256(data: bytes) -> str:
    import subprocess
    proc = subprocess.Popen(
        ["cast", "keccak", "--"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    stdout, stderr = proc.communicate(input=data)
    if proc.returncode != 0:
        raise RuntimeError(f"cast keccak failed: {stderr.decode()}")
    return stdout.decode().strip()

def rpc_call(method: str, params: list):
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(
        RPC_URL,
        data=payload,
        headers={"Content-Type": "application/json", "User-Agent": "Mozilla/5.0"}
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        res = json.loads(resp.read().decode())
    if "error" in res:
        raise RuntimeError(f"RPC error {method}: {res['error']}")
    return res.get("result")

def extract_decision_hash_from_logs(logs, module_address):
    for log in logs:
        if log.get("address", "").lower() != module_address.lower():
            continue
        topics = log.get("topics", [])
        if not topics:
            continue
        data = log.get("data", "")
        if data.startswith("0x"):
            raw_data = data[2:]
        else:
            raw_data = data

        if topics[0].lower() == ACTION_EXECUTED_TOPIC0.lower():
            # ActionExecuted(bytes32 digest, uint256 nonce, bytes32 decisionHash, ...)
            # decisionHash is 3rd word (bytes 64..96 -> chars 128..192)
            if len(raw_data) >= 192:
                dh = "0x" + raw_data[128:192]
                return dh, "ActionExecuted"
        elif topics[0].lower() == ACTION_QUEUED_TOPIC0.lower():
            # ActionQueued(bytes32 digest, bytes32 sessionId, uint256 nonce, uint64 ready, uint64 expires, bytes32 decisionHash)
            # decisionHash is 6th word (bytes 160..192 -> chars 320..384)
            if len(raw_data) >= 384:
                dh = "0x" + raw_data[320:384]
                return dh, "ActionQueued"
    return None, None

def main():
    root = Path(__file__).resolve().parent.parent.parent

    # 1. Load deployment manifest
    man_path = root / "deployments" / "84532.json"
    if not man_path.exists():
        print(f"FAIL: deployment manifest missing at {man_path}")
        sys.exit(1)
    with open(man_path) as f:
        manifest = json.load(f)
    session_address = manifest.get("roles", {}).get("session", "").lower()
    module_address = manifest.get("contracts", {}).get("module", {}).get("address", "").lower()
    if not module_address:
        module_address = manifest.get("module", "").lower()

    print(f"[check_golden] Expected Session Signer: {session_address}")
    print(f"[check_golden] Module Address:          {module_address}")

    # 2. Load K2 base-sepolia.json to ensure no reuse of K2 txs
    k2_path = root / "evidence" / "base-sepolia.json"
    k2_hashes = set()
    if k2_path.exists():
        with open(k2_path) as f:
            k2_data = json.load(f)
        for r in k2_data.get("rows", []):
            th = r.get("txHash")
            if th:
                k2_hashes.add(th.lower())
    print(f"[check_golden] Loaded {len(k2_hashes)} existing K2 tx hashes from {k2_path}")

    # 3. Load evidence/agent/golden.json
    golden_path = root / "evidence" / "agent" / "golden.json"
    if not golden_path.exists():
        print(f"FAIL: {golden_path} does not exist")
        sys.exit(1)
    with open(golden_path) as f:
        rows = json.load(f)

    if not isinstance(rows, list):
        print("FAIL: golden.json must contain a JSON array of rows")
        sys.exit(1)

    agent_tx_rows = [
        r for r in rows
        if r.get("source") == "AGENT" and r.get("txHash")
    ]
    print(f"[check_golden] Found {len(agent_tx_rows)} AGENT transaction rows in golden.json")

    # Requirement: >= 3 AGENT tx rows
    if len(agent_tx_rows) < 3:
        print(f"FAIL: expected at least 3 AGENT tx rows, found {len(agent_tx_rows)}")
        sys.exit(1)

    # 4. Verify each row
    for idx, row in enumerate(agent_tx_rows, 1):
        tx_hash = row.get("txHash")
        row_id = row.get("id", f"Row-{idx}")
        kind = row.get("kind", "UNKNOWN")
        print(f"\n--- Checking Row #{idx} [{row_id}] ({kind}): {tx_hash} ---")

        # 4a. Check absence from K2 base-sepolia.json
        if tx_hash.lower() in k2_hashes:
            print(f"FAIL: {tx_hash} appears in evidence/base-sepolia.json (reuse of K2 script tx is prohibited)")
            sys.exit(1)
        print(f"  [OK] Hash absent from evidence/base-sepolia.json")

        # 4b. Check run log exists
        run_log = root / "evidence" / "agent" / "runs" / f"{tx_hash.lower()}.log"
        run_log_orig = root / "evidence" / "agent" / "runs" / f"{tx_hash}.log"
        if not run_log.exists() and not run_log_orig.exists():
            print(f"FAIL: Run log missing at {run_log}")
            sys.exit(1)
        log_path = run_log if run_log.exists() else run_log_orig
        if log_path.stat().st_size == 0:
            print(f"FAIL: Run log {log_path} is empty")
            sys.exit(1)
        print(f"  [OK] Run log exists ({log_path.name}, {log_path.stat().st_size} bytes)")

        # 4c. Verify passport file and decisionHash keccak
        passport_rel = row.get("passport") or f"evidence/agent/passports/{tx_hash.lower()}.json"
        passport_path = root / passport_rel
        if not passport_path.exists():
            # Try decisionHash.json
            dh = row.get("decisionHash", "")
            alt_path = root / "evidence" / "agent" / "passports" / f"{dh.lower()}.json"
            if alt_path.exists():
                passport_path = alt_path
            else:
                print(f"FAIL: Passport file missing at {passport_path}")
                sys.exit(1)

        with open(passport_path, "rb") as pf:
            passport_bytes = pf.read()
        computed_dh = keccak256(passport_bytes)
        row_dh = row.get("decisionHash", "")
        if computed_dh.lower() != row_dh.lower():
            print(f"FAIL: Passport keccak256 ({computed_dh}) != row.decisionHash ({row_dh})")
            sys.exit(1)
        print(f"  [OK] Passport keccak matches row decisionHash: {computed_dh}")

        # 4d. RPC checks: receipt status == 1 and from == session_address
        receipt = rpc_call("eth_getTransactionReceipt", [tx_hash])
        if not receipt:
            print(f"FAIL: No transaction receipt returned for {tx_hash} from {RPC_URL}")
            sys.exit(1)

        st = receipt.get("status")
        if st != "0x1" and st != 1:
            print(f"FAIL: Transaction status is {st} (expected 0x1) for {tx_hash}")
            sys.exit(1)
        print(f"  [OK] Receipt status is 1 on {RPC_URL} (Block #{int(receipt['blockNumber'], 16)})")

        tx_obj = rpc_call("eth_getTransactionByHash", [tx_hash])
        if not tx_obj:
            print(f"FAIL: No transaction object returned for {tx_hash}")
            sys.exit(1)

        from_addr = tx_obj.get("from", "").lower()
        if from_addr != session_address:
            print(f"FAIL: Tx from ({from_addr}) != session address ({session_address})")
            sys.exit(1)
        print(f"  [OK] Signer is session address: {from_addr}")

        # 4e. Verify module event emitted with matching decisionHash
        event_dh, event_name = extract_decision_hash_from_logs(receipt.get("logs", []), module_address)
        if not event_dh:
            print(f"FAIL: No ActionExecuted or ActionQueued event found for module in receipt logs")
            sys.exit(1)
        if event_dh.lower() != computed_dh.lower():
            print(f"FAIL: On-chain event decisionHash ({event_dh}) != computed passport hash ({computed_dh})")
            sys.exit(1)
        print(f"  [OK] Module event '{event_name}' anchored decisionHash: {event_dh}")

    print("\n" + "=" * 60)
    print(f"SUCCESS: All {len(agent_tx_rows)} AGENT golden path transactions verified cleanly on Base Sepolia!")
    print("=" * 60)
    sys.exit(0)

if __name__ == "__main__":
    main()
