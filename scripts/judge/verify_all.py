#!/usr/bin/env python3
"""
scripts/judge/verify_all.py

Keyless, read-only judge verification engine for Fusion Wallet (FinTechathon 2026, Topic F).
Checks:
  1. Base Sepolia Evidence: runs scripts/verify_evidence.py (30 rows)
  2. Autonomous Agent Golden Path: runs scripts/agent/check_golden.py (3 rows)
  3. App Proofs: checks every evidence/app/*.json tx (status, from, decisionHash)
  4. Demo Runs: checks evidence/demo/runs.json (status, from == session, decisionHash anchoring)
  5. Sourcify Verification Links: verifies public contract verification URLs from deployments/84532.json

Prints a one-screen PASS/FAIL summary table and exits 0 only if all pass.
Requires no private keys or secrets.
"""

import glob
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RPC_URL = os.environ.get("BASE_SEPOLIA_RPC") or os.environ.get("RPC_URL") or "https://sepolia.base.org"

def rpc_call(method: str, params: list, retries: int = 4) -> dict:
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(
        RPC_URL,
        data=payload,
        headers={"Content-Type": "application/json", "User-Agent": "curl/8.0"}
    )
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"RPC error {method}: {res['error']}")
            return res.get("result")
        except Exception as e:
            if attempt == retries - 1:
                raise RuntimeError(f"RPC call failed after {retries} attempts: {e}")
            time.sleep(1 + attempt)

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from keccak256 import keccak256_hex  # noqa: E402  (pure Python: no Foundry needed)


def keccak256_bytes(data: bytes) -> str:
    return keccak256_hex(data)

def check_http_url(url: str, retries: int = 3) -> bool:
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"}
    req = urllib.request.Request(url, headers=headers)
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                if resp.status == 200:
                    return True
        except urllib.error.HTTPError as e:
            if e.code == 200:
                return True
            if attempt == retries - 1:
                return False
        except Exception:
            if attempt == retries - 1:
                return False
        time.sleep(1 + attempt)
    return False

def run_step1_verify_evidence() -> tuple[int, int, list[str]]:
    """Runs scripts/verify_evidence.py with retry logic on transient RPC disconnects."""
    evidence_path = ROOT / "evidence" / "base-sepolia.json"
    if not evidence_path.exists():
        return 0, 1, ["evidence/base-sepolia.json missing"]
    
    with open(evidence_path) as f:
        doc = json.load(f)
    total_rows = len(doc.get("rows", []))
    
    # Try running verify_evidence.py up to 3 times to handle public RPC keepalive drops
    last_err = ""
    for attempt in range(3):
        res = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "verify_evidence.py"), str(evidence_path), "--rpc-url", RPC_URL],
            cwd=ROOT,
            capture_output=True,
            text=True
        )
        if res.returncode == 0:
            return total_rows, 0, []
        last_err = res.stdout.strip() + "\n" + res.stderr.strip()
        time.sleep(2)
        
    return total_rows - 1, 1, [f"verify_evidence.py failed: {last_err.splitlines()[-1] if last_err else 'Unknown error'}"]

def run_step2_check_golden() -> tuple[int, int, list[str]]:
    """Runs scripts/agent/check_golden.py."""
    script_path = ROOT / "scripts" / "agent" / "check_golden.py"
    if not script_path.exists():
        return 0, 1, ["scripts/agent/check_golden.py missing"]
    
    env = os.environ.copy()
    env["BASE_SEPOLIA_RPC"] = RPC_URL
    res = subprocess.run(
        [sys.executable, str(script_path)],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True
    )
    if res.returncode == 0:
        # Count AGENT rows in evidence/agent/golden.json
        golden_file = ROOT / "evidence" / "agent" / "golden.json"
        count = 3
        if golden_file.exists():
            with open(golden_file) as f:
                rows = json.load(f)
            count = len([r for r in rows if r.get("source") == "AGENT" and r.get("txHash")])
        return count, 0, []
    else:
        err_msg = res.stdout.strip().splitlines()[-1] if res.stdout.strip() else res.stderr.strip()
        return 0, 1, [f"check_golden.py failed: {err_msg}"]

def run_step3_check_app_evidence(manifest: dict) -> tuple[int, int, list[str]]:
    """Checks every transaction in evidence/app/*.json (status, from, decisionHash)."""
    app_files = sorted(glob.glob(str(ROOT / "evidence" / "app" / "*.json")))
    # The in-app agent transaction (model decided inside the Android app, signed by
    # the Keystore session key) lives with its device evidence under app/agent/.
    app_files += sorted(glob.glob(str(ROOT / "evidence" / "app" / "agent" / "golden_app.json")))
    if not app_files:
        return 0, 0, []
    
    module_addr = manifest.get("contracts", {}).get("module", {}).get("address", "").lower()
    total = 0
    passed = 0
    errors = []
    allocation_rows = []

    for fpath in app_files:
        try:
            with open(fpath) as f:
                data = json.load(f)
        except Exception as e:
            errors.append(f"Cannot parse JSON in {Path(fpath).name}: {e}")
            continue
            
        items = data if isinstance(data, list) else [data]
        for idx, item in enumerate(items):
            tx_hash = item.get("txHash") or item.get("tx_hash")
            if not tx_hash:
                continue
            total += 1
            item_id = item.get("id", f"{Path(fpath).stem}#{idx}")
            
            # Check receipt
            try:
                receipt = rpc_call("eth_getTransactionReceipt", [tx_hash])
                if not receipt:
                    errors.append(f"[{item_id}] No receipt for tx {tx_hash}")
                    continue
                status = int(receipt.get("status", "0"), 16)
                if status != 1:
                    errors.append(f"[{item_id}] Tx {tx_hash} status is {status} (expected 1)")
                    continue
                
                tx = rpc_call("eth_getTransactionByHash", [tx_hash])
                if not tx:
                    errors.append(f"[{item_id}] Tx {tx_hash} not found")
                    continue
                
                # Check from if present
                exp_from = item.get("from")
                if exp_from:
                    actual_from = tx.get("from", "").lower()
                    if actual_from != exp_from.lower():
                        errors.append(f"[{item_id}] Tx from ({actual_from}) != expected ({exp_from.lower()})")
                        continue
                
                # Check decisionHash if present
                exp_dh = item.get("decisionHash") or item.get("decision_hash")
                if exp_dh:
                    passport_path = item.get("passport")
                    if passport_path:
                        p_file = ROOT / passport_path
                        if not p_file.exists():
                            errors.append(f"[{item_id}] Passport missing: {passport_path}")
                            continue
                        p_bytes = p_file.read_bytes()
                        computed_dh = keccak256_bytes(p_bytes)
                        if computed_dh != exp_dh.lower():
                            errors.append(f"[{item_id}] Passport hash ({computed_dh}) != decisionHash ({exp_dh})")
                            continue
                    
                    # Check event in logs
                    found_event = False
                    for log in receipt.get("logs", []):
                        if log.get("address", "").lower() == module_addr:
                            for topic in log.get("topics", []):
                                if topic.lower() == exp_dh.lower():
                                    found_event = True
                                    break
                            if exp_dh.lower().removeprefix("0x") in log.get("data", "").lower():
                                found_event = True
                                break
                    if not found_event:
                        errors.append(f"[{item_id}] decisionHash {exp_dh} not anchored in module event logs")
                        continue
                
                if Path(fpath).name == "golden_app.json":
                    # Verify model/signing process provenance and the exact event
                    # signature, action kind, amount and session-bound passport.
                    from importlib.util import spec_from_file_location, module_from_spec
                    android_dir = ROOT / "scripts" / "android_planner"
                    sys.path.insert(0, str(android_dir))
                    spec = spec_from_file_location("fusion_verify_app", android_dir / "verify_app.py")
                    verifier = module_from_spec(spec)
                    spec.loader.exec_module(verifier)
                    verifier.verify(item, manifest)
                    if item.get("id", "").startswith("K38-APP-"):
                        allocation_rows.append(item)
                passed += 1
            except Exception as e:
                errors.append(f"[{item_id}] Verification exception: {e}")
                
    if allocation_rows or (ROOT / "evidence/app/agent/K38_owner_allocation_receipt.json").exists():
        try:
            from verify_allocation import verify_round_trip
            verify_round_trip(allocation_rows, manifest, full=False)
            index = {row["txHash"].lower(): row for row in json.loads((ROOT / "evidence/tx_index.json").read_text())}
            for row in allocation_rows:
                indexed = index.get(row["txHash"].lower())
                if not indexed or indexed.get("source") != "APP" or indexed.get("notes") != "Owner-mandated rehearsal (2-of-2)":
                    raise ValueError(f"K38 transaction lacks its owner-mandated label: {row['txHash']}")
                if indexed.get("notes", "").startswith("AI-initiated"):
                    raise ValueError(f"K38 mandate transaction mislabeled autonomous: {row['txHash']}")

            # Any later APP deposit claimed as self-initiated must have a fresh,
            # actionable economics record and no owner mandate in its report.
            golden = json.loads((ROOT / "evidence/app/agent/golden_app.json").read_text())
            for row in golden:
                if row.get("kind") != "DEPOSIT" or row.get("allocationOwnerTx"):
                    continue
                indexed = index.get(row["txHash"].lower())
                if not indexed or indexed.get("source") != "APP" or not indexed.get("notes", "").startswith("AI-initiated · break-even "):
                    raise ValueError(f"APP deposit lacks an AI-initiated economics label: {row['txHash']}")
                report_path = ROOT / "evidence/app/agent/runs" / f"{row['txHash']}.json"
                report = json.loads(report_path.read_text())
                if "allocationMandate" in report:
                    raise ValueError(f"AI-initiated deposit report contains allocationMandate: {row['txHash']}")
                amount = report["chosenCandidate"]["action"]["assets"]
                if not any(item.get("sizeRaw") == amount and item.get("status") == "ACTIONABLE"
                           for item in report.get("quant", {}).get("candidates", [])):
                    raise ValueError(f"AI-initiated deposit has no matching ACTIONABLE quote: {row['txHash']}")
        except Exception as error:
            errors.append(f"K38 allocation round trip verification failed: {error}")
            return passed, max(1, total - passed), errors
    return passed, total - passed, errors

def run_step4_check_demo_runs(manifest: dict) -> tuple[int, int, list[str]]:
    """Checks evidence/demo/runs.json (status, from == session, decisionHash anchoring)."""
    runs_file = ROOT / "evidence" / "demo" / "runs.json"
    if not runs_file.exists():
        return 0, 1, ["evidence/demo/runs.json does not exist (run scripts/demo/live_demo.sh first)"]
    
    try:
        with open(runs_file) as f:
            runs = json.load(f)
    except Exception as e:
        return 0, 1, [f"Failed to read evidence/demo/runs.json: {e}"]
    
    if not isinstance(runs, list) or len(runs) == 0:
        return 0, 1, ["evidence/demo/runs.json must contain a non-empty array of runs"]
    
    session_addr = manifest.get("roles", {}).get("session", "").lower()
    module_addr = manifest.get("contracts", {}).get("module", {}).get("address", "").lower()
    
    passed = 0
    errors = []
    
    for idx, run in enumerate(runs):
        run_id = run.get("id", f"Run-{idx}")
        tx_hash = run.get("txHash") or run.get("tx_hash")
        if not tx_hash:
            errors.append(f"[{run_id}] Missing txHash")
            continue
        
        try:
            receipt = rpc_call("eth_getTransactionReceipt", [tx_hash])
            if not receipt:
                errors.append(f"[{run_id}] No receipt found for tx {tx_hash}")
                continue
            status = int(receipt.get("status", "0"), 16)
            if status != 1:
                errors.append(f"[{run_id}] Tx status is {status} (expected 1)")
                continue
            
            tx = rpc_call("eth_getTransactionByHash", [tx_hash])
            if not tx:
                errors.append(f"[{run_id}] Tx {tx_hash} not found")
                continue
            
            from_addr = tx.get("from", "").lower()
            if from_addr != session_addr:
                errors.append(f"[{run_id}] Tx sender ({from_addr}) != session key ({session_addr})")
                continue
            
            decision_hash = run.get("decisionHash") or run.get("decision_hash")
            if decision_hash:
                # Passport file check
                p_rel = run.get("passport") or f"evidence/demo/passports/{tx_hash.lower()}.json"
                p_path = ROOT / p_rel
                if not p_path.exists():
                    p_path = ROOT / "evidence" / "demo" / "passports" / f"{decision_hash.lower()}.json"
                if not p_path.exists():
                    errors.append(f"[{run_id}] Passport file not found: {p_rel}")
                    continue
                
                p_bytes = p_path.read_bytes()
                computed_dh = keccak256_bytes(p_bytes)
                if computed_dh != decision_hash.lower():
                    errors.append(f"[{run_id}] Passport keccak ({computed_dh}) != decisionHash ({decision_hash.lower()})")
                    continue
                
                # Event anchoring check
                anchored = False
                for log in receipt.get("logs", []):
                    if log.get("address", "").lower() == module_addr:
                        for topic in log.get("topics", []):
                            if topic.lower() == decision_hash.lower():
                                anchored = True
                                break
                        if decision_hash.lower().removeprefix("0x") in log.get("data", "").lower():
                            anchored = True
                            break
                if not anchored:
                    errors.append(f"[{run_id}] decisionHash {decision_hash} not found in module receipt event logs")
                    continue
            
            passed += 1
        except Exception as e:
            errors.append(f"[{run_id}] Verification exception: {e}")
            
    return passed, len(runs) - passed, errors

def run_step5_check_sourcify(manifest: dict) -> tuple[int, int, list[str]]:
    """Checks Sourcify contract verification links in deployments/84532.json."""
    contracts = manifest.get("contracts", {})
    sourcify_links = []
    for name, item in contracts.items():
        v = item.get("verification", {})
        url = v.get("url")
        if url and "sourcify.dev" in url:
            sourcify_links.append((name, url))
    
    if not sourcify_links:
        return 0, 0, []
    
    passed = 0
    errors = []
    for name, url in sourcify_links:
        ok = check_http_url(url)
        if ok:
            passed += 1
        else:
            errors.append(f"Sourcify link unreachable or non-200 for contract {name}: {url}")
            
    return passed, len(sourcify_links) - passed, errors

def main():
    print("=" * 76)
    print(" FUSION WALLET — COMPREHENSIVE JUDGE VERIFICATION SUITE")
    print(f" Working Directory: {ROOT}")
    print(f" RPC URL:           {RPC_URL}")
    print(f" Time:              {time.strftime('%Y-%m-%d %H:%M:%SZ', time.gmtime())}")
    print("=" * 76)

    manifest_path = ROOT / "deployments" / "84532.json"
    if not manifest_path.exists():
        print(f"FATAL: {manifest_path} not found")
        sys.exit(1)
    with open(manifest_path) as f:
        manifest = json.load(f)

    results = []

    # 1. Base Sepolia Evidence
    print("\n[1/5] Verifying Base Sepolia evidence rows (verify_evidence.py)...")
    s1_pass, s1_fail, s1_errs = run_step1_verify_evidence()
    s1_total = s1_pass + s1_fail
    results.append(("Base Sepolia Evidence (K2)", s1_total, s1_pass, s1_fail, s1_errs))

    # 2. Agent Golden Path
    print("[2/5] Verifying autonomous agent golden path (check_golden.py)...")
    s2_pass, s2_fail, s2_errs = run_step2_check_golden()
    s2_total = s2_pass + s2_fail
    results.append(("Agent Golden Path (K9)", s2_total, s2_pass, s2_fail, s2_errs))

    # 3. App Proofs
    print("[3/5] Verifying app transactions (evidence/app/*.json)...")
    s3_pass, s3_fail, s3_errs = run_step3_check_app_evidence(manifest)
    s3_total = s3_pass + s3_fail
    results.append(("App Proofs (evidence/app/*.json)", s3_total, s3_pass, s3_fail, s3_errs))

    # 4. Demo Runs
    print("[4/5] Verifying live demo runs (evidence/demo/runs.json)...")
    s4_pass, s4_fail, s4_errs = run_step4_check_demo_runs(manifest)
    s4_total = s4_pass + s4_fail
    results.append(("Demo Runs (evidence/demo/runs.json)", s4_total, s4_pass, s4_fail, s4_errs))

    # 5. Sourcify Verification Links
    print("[5/5] Verifying Sourcify contract source verification links...")
    s5_pass, s5_fail, s5_errs = run_step5_check_sourcify(manifest)
    s5_total = s5_pass + s5_fail
    results.append(("Sourcify Source Links (84532.json)", s5_total, s5_pass, s5_fail, s5_errs))

    # One-screen PASS/FAIL Summary Table
    print("\n" + "=" * 76)
    print("                     VERIFICATION SUMMARY TABLE")
    print("=" * 76)
    print(f"{'Verification Category':<38} {'Total':>7} {'Passed':>8} {'Failed':>8} {'Status':>10}")
    print("-" * 76)

    total_all = 0
    passed_all = 0
    failed_all = 0
    all_errors = []

    for name, total, passed, failed, errs in results:
        status_str = "PASS" if (failed == 0 and total > 0) else ("EMPTY" if total == 0 else "FAIL")
        print(f"{name:<38} {total:>7} {passed:>8} {failed:>8} {status_str:>10}")
        total_all += total
        passed_all += passed
        failed_all += failed
        if errs:
            all_errors.extend(errs)

    print("-" * 76)
    overall_status = "PASS" if failed_all == 0 and total_all > 0 else "FAIL"
    print(f"{'OVERALL VERIFICATION':<38} {total_all:>7} {passed_all:>8} {failed_all:>8} {overall_status:>10}")
    print("=" * 76)

    if all_errors:
        print("\nFailures / Discrepancies Details:")
        for err in all_errors:
            print(f"  [-] {err}")
        print()

    if overall_status == "PASS":
        print(f"All {passed_all} verification checks PASSED cleanly with 0 keys required.")
        sys.exit(0)
    else:
        print(f"Verification FAILED with {failed_all} error(s).")
        sys.exit(1)

if __name__ == "__main__":
    main()
