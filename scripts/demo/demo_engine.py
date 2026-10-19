#!/usr/bin/env python3
"""
scripts/demo/demo_engine.py

Live demo execution engine for Fusion Wallet (Card K19).
Performs end-to-end narrated flow on Base Sepolia:
  1) Snapshot: balances, buckets, module state
  2) Owners (2-of-2) commit a fresh bill (unique id, instant-tier amount) to active payee
  3) Agent tick with real planner (mnn+ahem-g): candidates, chosen letter, passport hash
  4) Payment transaction: broadcast signed by session key, wait for receipt
  5) Verification: status 1, from == session, decisionHash match in calldata/event, isPaid == true, duplicate pay revert
  6) Append row to evidence/demo/runs.json (§19.5 format, source = "AGENT", scenario = "live_demo")
"""

import json
import os
import secrets
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = ROOT / "deployments" / "84532.json"
OBLIGATIONS_DIR = ROOT / "evidence" / "demo"
PASSPORTS_DIR = ROOT / "evidence" / "demo" / "passports"
RUNS_DIR = ROOT / "evidence" / "demo" / "runs"
RUNS_JSON = ROOT / "evidence" / "demo" / "runs.json"

BASESCAN_TX = "https://sepolia.basescan.org/tx"
BASESCAN_ADDR = "https://sepolia.basescan.org/address"

def run_cmd(args, check=True):
    res = subprocess.run(args, capture_output=True, text=True)
    if check and res.returncode != 0:
        err = res.stderr.strip() or res.stdout.strip()
        raise RuntimeError(f"Command failed ({' '.join(args[:4])}...): {err}")
    return res.stdout.strip(), res.stderr.strip(), res.returncode

def cast_keccak(hex_str):
    if not hex_str.startswith("0x"):
        hex_str = "0x" + hex_str
    out, _, _ = run_cmd(["cast", "keccak", hex_str])
    return out.lower()

def word(val):
    if isinstance(val, str) and val.startswith("0x"):
        return val[2:].zfill(64)
    return f"{int(val):064x}"

def encode_multisend_entry(target, data_hex):
    data_bytes = bytes.fromhex(data_hex.removeprefix("0x"))
    target_bytes = bytes.fromhex(target.removeprefix("0x").zfill(40))
    val_bytes = bytes(32)
    len_bytes = len(data_bytes).to_bytes(32, "big")
    return bytes([0]) + target_bytes + val_bytes + len_bytes + data_bytes

def load_keys():
    env_local = Path("/root/projects/fusion_wallet/.env.local")
    env = {}
    if env_local.exists():
        with open(env_local) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    env[k.strip()] = v.strip().strip('"').strip("'")
    
    # Merge with os.environ prioritizing env_local
    for k, v in env.items():
        if k not in os.environ:
            os.environ[k] = v
            
    required = ["DEPLOYER_PRIVATE_KEY", "OWNER_PRIVATE_KEY", "APPROVER_PRIVATE_KEY", "SESSION_PRIVATE_KEY"]
    for r in required:
        if r not in os.environ:
            raise RuntimeError(f"Missing required private key: {r} in /root/projects/fusion_wallet/.env.local")
            
    return os.environ

def step1_snapshot(manifest, rpc_url):
    print("\n" + "=" * 76)
    print(" >>> STEP 1: On-Chain Snapshot & Health Pre-flight")
    print("=" * 76)

    safe = manifest["safe"]
    module = manifest["module"]
    asset = manifest["asset"]
    session_addr = manifest["roles"]["session"]
    deployer_addr = manifest["roles"]["deployer"]

    block_num, _, _ = run_cmd(["cast", "block", "latest", "-f", "number", "--rpc-url", rpc_url])
    block_time, _, _ = run_cmd(["cast", "block", "latest", "-f", "timestamp", "--rpc-url", rpc_url])
    print(f"[*] Base Sepolia Current Block: #{block_num} (Timestamp: {block_time})")

    # Balances
    safe_usdc_raw, _, _ = run_cmd(["cast", "call", asset, "balanceOf(address)(uint256)", safe, "--rpc-url", rpc_url])
    safe_usdc = int(safe_usdc_raw.split()[0])
    
    session_eth_raw, _, _ = run_cmd(["cast", "balance", session_addr, "--rpc-url", rpc_url])
    session_eth = int(session_eth_raw.split()[0])

    deployer_eth_raw, _, _ = run_cmd(["cast", "balance", deployer_addr, "--rpc-url", rpc_url])
    deployer_eth = int(deployer_eth_raw.split()[0])

    print(f"[*] Treasury Safe Cash:         {safe_usdc / 1e6:,.2f} USDC ({safe_usdc} raw)")
    print(f"[*] Session Key Gas:            {session_eth / 1e18:.6f} ETH")
    print(f"[*] Deployer / Relayer Gas:     {deployer_eth / 1e18:.6f} ETH")

    # Module state
    state_raw, _, _ = run_cmd([
        "cast", "call", module,
        "getState()((bool,bool,uint256,bytes32,bytes32,bytes32,uint256,uint256,uint256,uint256,uint256,bytes32,uint256,address,uint64,uint256,uint256))",
        "--rpc-url", rpc_url
    ])
    
    parts = [p.strip() for p in state_raw.strip("()").split(",")]
    configured = parts[0] == "true"
    paused = parts[1] == "true"
    epoch = int(parts[2])
    mandate_hash = parts[4]
    obligations_root = parts[5]
    reserved_raw = int(parts[6].split()[0])
    action_nonce = int(parts[8].split()[0])
    session_key_onchain = parts[13]
    session_expires = int(parts[14].split()[0])
    session_limit = int(parts[15].split()[0])
    session_used = int(parts[16].split()[0])

    print(f"[*] Module State:")
    print(f"    - Configured:               {configured}")
    print(f"    - Paused:                   {paused}")
    print(f"    - Policy Epoch:             {epoch}")
    print(f"    - Obligations Root:         {obligations_root}")
    print(f"    - Reserved Raw:             {reserved_raw / 1e6:.2f} USDC ({reserved_raw})")
    print(f"    - Action Nonce:             {action_nonce}")
    print(f"    - Active Session Key:       {session_key_onchain}")
    print(f"    - Session Expires:          {session_expires} (Remaining: {max(0, session_expires - int(block_time))}s)")
    print(f"    - Session Limit / Used:     {session_used / 1e6:.2f} / {session_limit / 1e6:.2f} USDC")

    print(f"[*] Contract Addresses & Explorers:")
    print(f"    - Safe:                     {safe} ({BASESCAN_ADDR}/{safe})")
    print(f"    - KavachiModule:            {module} ({BASESCAN_ADDR}/{module})")
    print(f"    - Session Key:              {session_addr} ({BASESCAN_ADDR}/{session_addr})")

    # Fail loudly on low funds or paused module
    if paused:
        print("\n" + "!" * 76)
        print("[-] FATAL ERROR: KavachiModule is currently PAUSED!")
        print("[-] How to fix:")
        print(f"    Owners must call KavachiModule.resume({epoch}) via Safe 2-of-2 transaction.")
        print(f"    Safe address: {safe}")
        print("!" * 76 + "\n")
        sys.exit(1)

    if session_eth < int(0.001 * 1e18):
        print("\n" + "!" * 76)
        print(f"[-] FATAL ERROR: Session key ETH balance too low ({session_eth / 1e18:.6f} ETH < 0.001 ETH)!")
        print("[-] How to fix:")
        print(f"    Fund the session signer on Base Sepolia with at least 0.005 ETH:")
        print(f"    cast send {session_addr} --value 0.01ether --private-key <funder_key> --rpc-url {rpc_url}")
        print("!" * 76 + "\n")
        sys.exit(1)

    if safe_usdc < 2_000_000:
        print("\n" + "!" * 76)
        print(f"[-] FATAL ERROR: Safe USDC balance too low ({safe_usdc / 1e6:.2f} USDC < 2.0 USDC)!")
        print("[-] How to fix:")
        print(f"    Fund the Safe with test USDC at {safe}:")
        print(f"    cast send {asset} \"transfer(address,uint256)\" {safe} 10000000 --private-key <funder_key> --rpc-url {rpc_url}")
        print("!" * 76 + "\n")
        sys.exit(1)

    print("[+] Pre-flight verification PASSED: Sufficient funds, valid session key, module active.")
    return epoch, int(block_time)

def step2_owner_commit_fresh_bill(manifest, rpc_url, current_epoch, current_time):
    print("\n" + "=" * 76)
    print(" >>> STEP 2: Owners (2-of-2) Commit Fresh Instant Bill & Re-arm Session")
    print("=" * 76)

    safe = manifest["safe"]
    module = manifest["module"]
    multi_send = manifest["contracts"]["multiSendCallOnly"]["address"]
    session_addr = manifest["roles"]["session"]

    deployer_key = os.environ["DEPLOYER_PRIVATE_KEY"]
    owner_key = os.environ["OWNER_PRIVATE_KEY"]
    approver_key = os.environ["APPROVER_PRIVATE_KEY"]

    # Generate a unique fresh bill
    entropy = secrets.token_hex(8)
    bill_id = cast_keccak("0x" + f"fusion.demo.bill.{current_time}.{entropy}".encode().hex())
    
    # Active verified payee: Acme Hosting
    payee_id = cast_keccak("0x" + b"fusion.k2.payee.acme".hex())
    recipient = "0x61dd06cE2f07d3E4F97d097741b0D7FD50F97F79"
    amount = 2_000_000 # 2.00 USDC (instant-tier: microMaxRaw is 10.00 USDC)
    due_at = current_time + 86400 * 2 # 2 days in future (inside 3-day window)

    inner = cast_keccak("0x" + word(bill_id) + word(payee_id) + word(amount) + word(due_at))
    leaf = cast_keccak(inner)
    root = leaf
    proof = []

    print(f"[*] Generated Fresh Business Bill:")
    print(f"    - Obligation ID:            {bill_id}")
    print(f"    - Payee (Acme Hosting):     {payee_id}")
    print(f"    - Recipient Account:        {recipient}")
    print(f"    - Bill Amount:              {amount / 1e6:.2f} USDC ({amount} raw)")
    print(f"    - Due Timestamp:            {due_at} (+48h)")
    print(f"    - Merkle Leaf:              {leaf}")
    print(f"    - Obligations Root:         {root}")

    # Write obligations to evidence/demo/obligations.json
    OBLIGATIONS_DIR.mkdir(parents=True, exist_ok=True)
    ob_file = OBLIGATIONS_DIR / "obligations.json"
    ob_data = {
        "root": root,
        "bills": [
            {
                "obligationIdHash": bill_id,
                "payeeIdHash": payee_id,
                "assets": str(amount),
                "dueAt": str(due_at),
                "leaf": leaf,
                "recipient": recipient,
                "proof": proof
            }
        ]
    }
    with open(ob_file, "w") as f:
        json.dump(ob_data, f, indent=2)
    print(f"[*] Saved fresh obligations to {ob_file.relative_to(ROOT)}")

    # Build MultiSend batch
    batch_bytes = bytearray()

    # 1. commitObligations(root, reservedRaw, count)
    commit_cd, _, _ = run_cmd(["cast", "calldata", "commitObligations(bytes32,uint256,uint256)", root, str(amount), "1"])
    batch_bytes.extend(encode_multisend_entry(module, commit_cd))

    # 2. startSession(key, expiresAt, sessionLimitRaw)
    session_expires = current_time + 86400 # 24 hours
    session_limit = 8000_000_000 # 8,000 USDC
    session_cd, _, _ = run_cmd(["cast", "calldata", "startSession(address,uint64,uint256)", session_addr, str(session_expires), str(session_limit)])
    batch_bytes.extend(encode_multisend_entry(module, session_cd))

    # 3. resume(expectedEpoch)
    # commitObligations increments policyEpoch by 1
    next_epoch = current_epoch + 1
    resume_cd, _, _ = run_cmd(["cast", "calldata", "resume(uint256)", str(next_epoch)])
    batch_bytes.extend(encode_multisend_entry(module, resume_cd))

    multisend_cd, _, _ = run_cmd(["cast", "calldata", "multiSend(bytes)", "0x" + batch_bytes.hex()])

    # Safe getTransactionHash
    safe_nonce_raw, _, _ = run_cmd(["cast", "call", safe, "nonce()(uint256)", "--rpc-url", rpc_url])
    safe_nonce = int(safe_nonce_raw.split()[0])

    digest_raw, _, _ = run_cmd([
        "cast", "call", safe,
        "getTransactionHash(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,uint256)(bytes32)",
        multi_send, "0", multisend_cd, "1", "0", "0", "0",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        str(safe_nonce),
        "--rpc-url", rpc_url
    ])
    digest = digest_raw.strip()

    # Sign with Owner and Approver
    owner_sig, _, _ = run_cmd(["cast", "wallet", "sign", "--no-hash", digest, "--private-key", owner_key])
    approver_sig, _, _ = run_cmd(["cast", "wallet", "sign", "--no-hash", digest, "--private-key", approver_key])

    owner_addr, _, _ = run_cmd(["cast", "wallet", "address", "--private-key", owner_key])
    approver_addr, _, _ = run_cmd(["cast", "wallet", "address", "--private-key", approver_key])

    # Safe requires signatures sorted by address ascending
    if int(approver_addr, 16) < int(owner_addr, 16):
        signatures = approver_sig + owner_sig.removeprefix("0x")
    else:
        signatures = owner_sig + approver_sig.removeprefix("0x")

    print("[*] Broadcasting Safe 2-of-2 transaction from Deployer...")
    tx_out, _, _ = run_cmd([
        "cast", "send", safe,
        "execTransaction(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,bytes)",
        multi_send, "0", multisend_cd, "1", "0", "0", "0",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        signatures,
        "--private-key", deployer_key,
        "--rpc-url", rpc_url,
        "--json"
    ])
    tx_json = json.loads(tx_out)
    setup_tx_hash = tx_json["transactionHash"]
    setup_status = int(tx_json.get("status", 0), 16 if isinstance(tx_json.get("status"), str) else 10)
    setup_block = int(tx_json.get("blockNumber", 0), 16 if isinstance(tx_json.get("blockNumber"), str) else int(tx_json.get("blockNumber", 0)))

    if setup_status != 1:
        raise RuntimeError(f"Owner setup Safe transaction failed: {setup_tx_hash}")

    print(f"[+] Safe 2-of-2 MultiSend included successfully in block #{setup_block}!")
    print(f"    - Tx Hash:                  {setup_tx_hash}")
    print(f"    - BaseScan:                 {BASESCAN_TX}/{setup_tx_hash}")

    # Wait for RPC node state replication
    print(f"[*] Verifying on-chain state synchronization at block #{setup_block}...")
    for attempt in range(20):
        st_raw, _, _ = run_cmd([
            "cast", "call", module,
            "getState()((bool,bool,uint256,bytes32,bytes32,bytes32,uint256,uint256,uint256,uint256,uint256,bytes32,uint256,address,uint64,uint256,uint256))",
            "--rpc-url", rpc_url
        ])
        if root.lower() in st_raw.lower():
            print(f"[+] On-chain state synchronized: obligationsRoot active.")
            break
        time.sleep(1)

    time.sleep(2)
    return bill_id, payee_id, recipient, amount, due_at, ob_file

def step3_and_4_agent_tick_and_pay(manifest, rpc_url, ob_file):
    print("\n" + "=" * 76)
    print(" >>> STEP 3 & 4: Agent Tick with MNN+AHEM-G Planner & Autonomous Payment")
    print("=" * 76)

    agent_runner = ROOT / "src/fusion-core/target/release/agent-runner"
    planner_bin = ROOT / "src/fusion-planner/target/release/planner-host"
    model_dir = Path("/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m")

    PASSPORTS_DIR.mkdir(parents=True, exist_ok=True)
    RUNS_DIR.mkdir(parents=True, exist_ok=True)
    tmp_out = OBLIGATIONS_DIR / "tmp_agent_out.json"
    if tmp_out.exists():
        tmp_out.unlink()

    run_id = f"DEMO-{int(time.time())}"
    print(f"[*] Launching autonomous agent runner ({agent_runner.name})...")
    print(f"    - Planner:                  host (MNN + AHEM-G)")
    print(f"    - Planner Binary:           {planner_bin}")
    print(f"    - Model Directory:          {model_dir}")

    cmd = [
        str(agent_runner),
        "--manifest", str(MANIFEST_PATH),
        "--obligations", str(ob_file),
        "--planner", "host",
        "--planner-bin", str(planner_bin),
        "--model-dir", str(model_dir),
        "--choice", "A",
        "--user-goal", "Pay verified vendor bill A immediately",
        "--evidence-out", str(tmp_out),
        "--evidence-id", run_id,
        "--passport-dir", str(PASSPORTS_DIR),
        "--runs-dir", str(RUNS_DIR),
        "--source", "AGENT"
    ]

    stdout, stderr, code = run_cmd(cmd)
    for line in stdout.splitlines():
        if "Enumerated" in line or "candidate(s)" in line or "kind=" in line or "Planner" in line or "Passport" in line or "Tx" in line or "Final Choice" in line:
            print(f"    [Agent] {line}")

    if not tmp_out.exists():
        raise RuntimeError(f"agent-runner failed to produce output row. Output:\n{stdout}\n{stderr}")

    with open(tmp_out) as f:
        rows = json.load(f)
    if not rows:
        raise RuntimeError("agent-runner output is empty")
    row = rows[-1]

    pay_tx_hash = row["txHash"]
    decision_hash = row["decisionHash"]
    action_digest = row["actionDigest"]
    print(f"\n[+] Payment Transaction Submitted & Included:")
    print(f"    - Tx Hash:                  {pay_tx_hash}")
    print(f"    - Decision Hash:            {decision_hash}")
    print(f"    - Action Digest:            {action_digest}")
    print(f"    - BaseScan:                 {BASESCAN_TX}/{pay_tx_hash}")

    return row

def step5_verify(manifest, rpc_url, bill_id, row):
    print("\n" + "=" * 76)
    print(" >>> STEP 5: Independent Verification of Invariants & Protections")
    print("=" * 76)

    module = manifest["module"]
    session_addr = manifest["roles"]["session"]
    pay_tx_hash = row["txHash"]
    decision_hash = row["decisionHash"]

    # 1. Receipt status
    receipt_raw, _, _ = run_cmd(["cast", "receipt", pay_tx_hash, "--json", "--rpc-url", rpc_url])
    receipt = json.loads(receipt_raw)
    status = int(receipt.get("status", 0), 16 if isinstance(receipt.get("status"), str) else 10)
    assert status == 1, f"Payment tx status is {status} (expected 1)"
    print(f"[*] [PASS] Receipt status == 1 (Confirmed in Block #{int(receipt['blockNumber'], 16)})")

    # 2. Tx from == session key
    tx_raw, _, _ = run_cmd(["cast", "tx", pay_tx_hash, "--json", "--rpc-url", rpc_url])
    tx_obj = json.loads(tx_raw)
    tx_data = tx_obj.get("data", tx_obj)
    from_addr = tx_data.get("from", "").lower()
    assert from_addr == session_addr.lower(), f"Tx sender {from_addr} != session key {session_addr}"
    print(f"[*] [PASS] Transaction sender == session key ({from_addr})")

    # 3. DecisionHash == keccak(passport)
    # Check passport file
    p_file = PASSPORTS_DIR / f"{pay_tx_hash.lower()}.json"
    if not p_file.exists():
        p_file = PASSPORTS_DIR / f"{decision_hash.lower()}.json"
    assert p_file.exists(), f"Passport file missing at {p_file}"
    computed_dh = cast_keccak("0x" + p_file.read_bytes().hex())
    assert computed_dh.lower() == decision_hash.lower(), f"Passport hash {computed_dh} != {decision_hash}"
    print(f"[*] [PASS] Passport keccak256({p_file.name}) == decisionHash ({decision_hash})")

    # Ensure both filenames exist for indexer convenience
    tx_p = PASSPORTS_DIR / f"{pay_tx_hash.lower()}.json"
    dh_p = PASSPORTS_DIR / f"{decision_hash.lower()}.json"
    if tx_p.exists() and not dh_p.exists():
        dh_p.write_bytes(tx_p.read_bytes())
    elif dh_p.exists() and not tx_p.exists():
        tx_p.write_bytes(dh_p.read_bytes())

    # 4. decisionHash in calldata & ActionExecuted event
    calldata = tx_data.get("input", "")
    assert decision_hash.lower().removeprefix("0x") in calldata.lower(), "decisionHash missing from tx calldata"
    print(f"[*] [PASS] decisionHash bound inside on-chain transaction calldata")

    # Check event
    event_found = False
    for log in receipt.get("logs", []):
        if log.get("address", "").lower() == module.lower():
            for topic in log.get("topics", []):
                if topic.lower() == decision_hash.lower():
                    event_found = True
                    break
            if decision_hash.lower().removeprefix("0x") in log.get("data", "").lower():
                event_found = True
                break
    assert event_found, "decisionHash not found in receipt ActionExecuted event logs"
    print(f"[*] [PASS] ActionExecuted event anchored decisionHash on-chain")

    # 5. Bill isPaid on-chain
    is_paid_raw, _, _ = run_cmd(["cast", "call", module, "isPaid(bytes32)(bool)", bill_id, "--rpc-url", rpc_url])
    assert "true" in is_paid_raw.lower(), f"isPaid({bill_id}) returned {is_paid_raw}"
    print(f"[*] [PASS] On-chain state: KavachiModule.isPaid({bill_id}) == true")

    # 6. Duplicate pay reverts
    print(f"[*] Testing replay protection: submitting duplicate payment for {bill_id}...")
    _, replay_err, replay_code = run_cmd(
        ["cast", "call", module, calldata, "--from", session_addr, "--rpc-url", rpc_url],
        check=False
    )
    assert replay_code != 0, "Duplicate payment simulation unexpectedly succeeded!"
    print(f"[*] [PASS] Duplicate payment attempt reverted on-chain (as expected)")
    print(f"    - Revert response: {replay_err.splitlines()[-1] if replay_err else 'reverted'}")

def step6_record_evidence(row, bill_id, payee_id, recipient, amount):
    print("\n" + "=" * 76)
    print(" >>> STEP 6: Recording Demo Evidence Row (§19.5)")
    print("=" * 76)

    pay_tx_hash = row["txHash"]
    decision_hash = row["decisionHash"]

    # Canonical passport path under evidence/demo/passports
    passport_rel = f"evidence/demo/passports/{pay_tx_hash.lower()}.json"

    demo_row = {
        "id": row.get("id", f"DEMO-{int(time.time())}"),
        "source": "AGENT",
        "scenario": "live_demo",
        "planner": "mnn+ahem-g",
        "host": "linux-host",
        "txHash": pay_tx_hash,
        "decisionHash": decision_hash,
        "actionDigest": row.get("actionDigest", ""),
        "candidateId": row.get("candidateId", ""),
        "choiceLabel": row.get("choiceLabel", "A"),
        "kind": row.get("kind", "PAY_INSTANT"),
        "route": row.get("route", "Instant"),
        "status": 1,
        "blockNumber": row.get("blockNumber", 0),
        "gasUsed": row.get("gasUsed", 0),
        "passport": passport_rel,
        "passportFile": f"{decision_hash.lower()}.json",
        "obligationIdHash": bill_id,
        "payeeIdHash": payee_id,
        "recipient": recipient,
        "assets": str(amount)
    }

    RUNS_DIR.mkdir(parents=True, exist_ok=True)
    existing_runs = []
    if RUNS_JSON.exists():
        try:
            with open(RUNS_JSON) as f:
                existing_runs = json.load(f)
        except Exception:
            existing_runs = []

    existing_runs.append(demo_row)
    with open(RUNS_JSON, "w") as f:
        json.dump(existing_runs, f, indent=2)

    print(f"[+] Successfully recorded live demo run to {RUNS_JSON.relative_to(ROOT)}")
    print(f"    - Total demo runs recorded: {len(existing_runs)}")
    print(f"    - Latest txHash:            {pay_tx_hash}")

def main():
    print("=" * 76)
    print("   FUSION WALLET — LIVE DEMO EXECUTION (Phase 4, Card K19)")
    print(f"   Time: {time.strftime('%Y-%m-%d %H:%M:%SZ', time.gmtime())}")
    print("=" * 76)

    load_keys()
    with open(MANIFEST_PATH) as f:
        manifest = json.load(f)
    rpc_url = os.environ.get("RPC_URL", "https://sepolia.base.org")

    current_epoch, current_time = step1_snapshot(manifest, rpc_url)
    bill_id, payee_id, recipient, amount, due_at, ob_file = step2_owner_commit_fresh_bill(manifest, rpc_url, current_epoch, current_time)
    row = step3_and_4_agent_tick_and_pay(manifest, rpc_url, ob_file)
    step5_verify(manifest, rpc_url, bill_id, row)
    step6_record_evidence(row, bill_id, payee_id, recipient, amount)

    print("\n" + "=" * 76)
    print("   LIVE DEMO EXECUTION COMPLETED SUCCESSFULLY!")
    print(f"   Payment Tx: {BASESCAN_TX}/{row['txHash']}")
    print(f"   Evidence:   {RUNS_JSON.relative_to(ROOT)}")
    print("=" * 76 + "\n")

if __name__ == "__main__":
    main()
