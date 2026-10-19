#!/usr/bin/env python3
"""Execute 2-of-2 Safe transaction to establish fresh on-chain state for K9 agent golden path."""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEPLOYMENTS = ROOT / "deployments/84532.json"

def run_cmd(args):
    res = subprocess.run(args, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"Command failed ({args[0]}): {res.stderr.strip() or res.stdout.strip()}")
    return res.stdout.strip()

def keccak(hex_str):
    if not hex_str.startswith("0x"):
        hex_str = "0x" + hex_str
    return run_cmd(["cast", "keccak", hex_str]).lower()

def word(val):
    if isinstance(val, str) and val.startswith("0x"):
        return val[2:].zfill(64)
    return f"{int(val):064x}"

def encode_entry(target, data_hex):
    data_bytes = bytes.fromhex(data_hex.removeprefix("0x"))
    target_bytes = bytes.fromhex(target.removeprefix("0x").zfill(40))
    val_bytes = bytes(32)
    len_bytes = len(data_bytes).to_bytes(32, "big")
    return bytes([0]) + target_bytes + val_bytes + len_bytes + data_bytes

def make_obligations(due):
    amounts = [8 * 10**6, 50 * 10**6] # 8 USDC (instant tier) and 50 USDC (delayed tier)
    payee_acme = keccak("0x" + b"fusion.k2.payee.acme".hex())
    payee_payroll = keccak("0x" + b"fusion.k2.payee.payroll".hex())
    payees = [payee_acme, payee_payroll]
    recipients = [
        "0x61dd06cE2f07d3E4F97d097741b0D7FD50F97F79",
        "0x208Bfee9F222d611b4Db71d5ee53b3dAB1A2188A",
    ]

    bills = []
    for i in range(2):
        bill_id = keccak("0x" + (b"fusion.k9.bill." + bytes([i])).hex())
        payee = payees[i]
        amount = amounts[i]
        inner = keccak("0x" + word(bill_id) + word(payee) + word(amount) + word(due))
        leaf = keccak(inner)
        bills.append({
            "obligationIdHash": bill_id,
            "payeeIdHash": payee,
            "assets": str(amount),
            "dueAt": str(due),
            "leaf": leaf,
            "recipient": recipients[i],
        })

    leaves = sorted(b["leaf"] for b in bills)
    tree = [None] * (len(leaves) * 2 - 1)
    for i, leaf in enumerate(leaves):
        tree[len(tree) - 1 - i] = leaf
    for i in reversed(range(len(leaves) - 1)):
        pair = sorted([tree[2 * i + 1], tree[2 * i + 2]])
        tree[i] = keccak("0x" + pair[0][2:] + pair[1][2:])

    for bill in bills:
        idx = len(tree) - 1 - leaves.index(bill["leaf"])
        proof = []
        while idx > 0:
            sibling = tree[idx - 1 if idx % 2 == 0 else idx + 1]
            proof.append(sibling)
            idx = (idx - 1) // 2
        bill["proof"] = proof

    root = tree[0]
    return root, bills

def main():
    simulate_only = "--simulate" in sys.argv
    print(f"Setting up fresh state on Base Sepolia (simulate={simulate_only})...")

    # Load environment variables
    env_local = Path("/root/projects/fusion_wallet/.env.local")
    env = {}
    if env_local.exists():
        with open(env_local) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    env[k.strip()] = v.strip().strip('"').strip("'")

    rpc_url = env.get("RPC_URL", "https://sepolia.base.org")
    deployer_key = env["DEPLOYER_PRIVATE_KEY"]
    owner_key = env["OWNER_PRIVATE_KEY"]
    approver_key = env["APPROVER_PRIVATE_KEY"]

    with open(DEPLOYMENTS) as f:
        manifest = json.load(f)

    safe = manifest["safe"]
    module = manifest["module"]
    asset = manifest["asset"]
    multi_send = manifest["contracts"]["multiSendCallOnly"]["address"]
    session_address = manifest["roles"]["session"]

    # 1. Check current timestamp and compute fresh obligations
    now = int(run_cmd(["cast", "block", "latest", "-f", "timestamp", "--rpc-url", rpc_url]))
    due_at = now + 86400 * 2 # 2 days out, inside 3-day window

    root, bills = make_obligations(due_at)
    print(f"Computed new obligations root: {root}")
    for idx, b in enumerate(bills):
        print(f"  - Bill {idx}: id={b['obligationIdHash']} assets={int(b['assets'])/1e6:.1f} USDC leaf={b['leaf']}")

    # Write obligations to evidence/agent/obligations-84532.json
    ob_path = ROOT / "evidence/agent/obligations-84532.json"
    ob_data = {"root": root, "bills": bills}
    with open(ob_path, "w") as f:
        json.dump(ob_data, f, indent=2)
    print(f"Saved fresh obligations to {ob_path.relative_to(ROOT)}")

    # 2. Check current cash balance
    bal_raw = run_cmd(["cast", "call", asset, "balanceOf(address)(uint256)", safe, "--rpc-url", rpc_url])
    current_cash = int(bal_raw.split()[0])
    print(f"Safe current USDC cash: {current_cash / 1e6:.6f} USDC ({current_cash} raw)")

    # 3. Query current module policy epoch
    epoch_raw = run_cmd(["cast", "call", module, "getState()((bool,bool,uint256,bytes32,bytes32,bytes32,uint256,uint256,uint256,uint256,uint256,bytes32,uint256,address,uint64,uint256,uint256))", "--rpc-url", rpc_url])
    epoch_str = epoch_raw.split(",")[2].strip()
    current_epoch = int(epoch_str)
    print(f"Module current policyEpoch: {current_epoch}")

    # Build MultiSend batch
    batch_bytes = bytearray()

    # Step 1: commitObligations(bytes32 root, uint256 reservedRaw, uint256 count)
    reserved_raw = 58_000_000 # 58 USDC (8 USDC + 50 USDC)
    count = 2
    commit_calldata = run_cmd(["cast", "calldata", "commitObligations(bytes32,uint256,uint256)", root, str(reserved_raw), str(count)])
    batch_bytes.extend(encode_entry(module, commit_calldata))

    # Step 2: startSession(address key, uint64 expiresAt, uint256 sessionLimitRaw)
    session_expires = now + 82800 # 23 hours
    session_limit = 8000_000_000 # 8,000 USDC
    session_calldata = run_cmd(["cast", "calldata", "startSession(address,uint64,uint256)", session_address, str(session_expires), str(session_limit)])
    batch_bytes.extend(encode_entry(module, session_calldata))

    # Step 3: resume(uint256 expectedEpoch)
    # Note: commitObligations resets obligations and increments policyEpoch by 1!
    resume_calldata = run_cmd(["cast", "calldata", "resume(uint256)", str(current_epoch + 1)])
    batch_bytes.extend(encode_entry(module, resume_calldata))

    # MultiSend call encoding
    batch_hex = "0x" + batch_bytes.hex()
    multisend_calldata = run_cmd(["cast", "calldata", "multiSend(bytes)", batch_hex])

    # Safe getTransactionHash
    nonce_raw = run_cmd(["cast", "call", safe, "nonce()(uint256)", "--rpc-url", rpc_url])
    safe_nonce = int(nonce_raw.split()[0])
    print(f"Safe nonce: {safe_nonce}")

    digest_raw = run_cmd([
        "cast", "call", safe,
        "getTransactionHash(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,uint256)(bytes32)",
        multi_send, "0", multisend_calldata, "1", "0", "0", "0",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        str(safe_nonce),
        "--rpc-url", rpc_url
    ])
    digest = digest_raw.strip()
    print(f"Safe transaction EIP-712 digest: {digest}")

    # Sign digest with OWNER and APPROVER
    owner_sig = run_cmd(["cast", "wallet", "sign", "--no-hash", digest, "--private-key", owner_key])
    approver_sig = run_cmd(["cast", "wallet", "sign", "--no-hash", digest, "--private-key", approver_key])

    owner_addr = run_cmd(["cast", "wallet", "address", "--private-key", owner_key]).lower()
    approver_addr = run_cmd(["cast", "wallet", "address", "--private-key", approver_key]).lower()

    # Signatures must be sorted by address ascending
    if int(approver_addr, 16) < int(owner_addr, 16):
        signatures = approver_sig + owner_sig.removeprefix("0x")
    else:
        signatures = owner_sig + approver_sig.removeprefix("0x")

    # Simulate via cast call
    sim_out = run_cmd([
        "cast", "call", safe,
        "execTransaction(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,bytes)(bool)",
        multi_send, "0", multisend_calldata, "1", "0", "0", "0",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        signatures,
        "--from", manifest["roles"]["deployer"],
        "--rpc-url", rpc_url
    ])
    print(f"Simulation call result: {sim_out}")
    if "true" not in sim_out.lower():
        raise RuntimeError(f"Simulation failed: {sim_out}")
    print("Simulation PASSED! Safe 2-of-2 transaction will succeed.")

    if simulate_only:
        print("Simulation complete. Exiting without broadcasting.")
        return

    # Broadcast transaction from DEPLOYER
    print("Broadcasting Safe 2-of-2 transaction...")
    tx_out = run_cmd([
        "cast", "send", safe,
        "execTransaction(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,bytes)",
        multi_send, "0", multisend_calldata, "1", "0", "0", "0",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        signatures,
        "--private-key", deployer_key,
        "--rpc-url", rpc_url,
        "--json"
    ])
    tx_json = json.loads(tx_out)
    tx_hash = tx_json["transactionHash"]
    status = int(tx_json.get("status", 0), 16 if isinstance(tx_json.get("status"), str) else 10)
    print(f"Owner setup transaction broadcast: {tx_hash} (status={status})")

    if status != 1:
        raise RuntimeError(f"Safe transaction failed: {tx_hash}")

    # Verify on-chain state
    print("Verifying on-chain state after refresh...")
    epoch_after = run_cmd(["cast", "call", module, "getState()((bool,bool,uint256,bytes32,bytes32,bytes32,uint256,uint256,uint256,uint256,uint256,bytes32,uint256,address,uint64,uint256,uint256))", "--rpc-url", rpc_url])
    print(f"New state: {epoch_after}")

    print(f"Owner setup complete! Transaction hash: {tx_hash}")
    return tx_hash

if __name__ == "__main__":
    main()
