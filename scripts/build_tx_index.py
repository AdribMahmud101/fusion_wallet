#!/usr/bin/env python3
"""Derive the public Activity index from recorded evidence and Base Sepolia logs.

No provenance is inferred from action names, signers or deployment defaults.
Missing payment details are read once per transaction during this build; clients
only need the resulting JSON, not a receipt request for every Activity row.
"""
import concurrent.futures
import json
import os
import re
from collections import defaultdict

from chain import ROOT, Rpc, decode_event, event_abi, keccak, signature, write_json

INPUTS = (
    "evidence/base-sepolia.json",
    "evidence/agent/golden.json",
    "evidence/demo/runs.json",
    "evidence/app/agent/golden_app.json",
    "evidence/app/guardian_pause.json",
    "evidence/app/agent/owner_setup_receipt.json",
)
HASH = re.compile(r"0x[0-9a-fA-F]{64}\Z")
ADDRESS = re.compile(r"0x[0-9a-fA-F]{40}\Z")


def read(path):
    return json.loads((ROOT / path).read_text())


def payee_catalog(base_rows):
    # Resolve the label hashes actually recorded by PayeeAdded against the
    # literal preimages in the owner setup scripts. Never hand-maintain names.
    preimages = {}
    for path in sorted((ROOT / "src/contracts/script").glob("*.sol")):
        for literal in re.findall(r'keccak256\("([^"\\]*)"\)', path.read_text()):
            preimages[keccak(literal)] = literal
    payees = {}
    for row in base_rows:
        event = row.get("expectedEvent") or {}
        fields = event.get("fields", {})
        if event.get("name") == "PayeeAdded":
            payees[fields["payeeIdHash"].lower()] = {
                "recipient": fields["account"].lower(),
                "payeeLabel": preimages.get(fields["labelHash"].lower()),
            }
    # Bill sets can also carry explicit human-readable payee metadata.
    bills = defaultdict(list)
    for path in sorted((ROOT / "evidence/obligations/by-root").glob("*.json")):
        for bill in json.loads(path.read_text()).get("bills", []):
            bills[bill["obligationIdHash"].lower()].append(bill)
            payee = payees.setdefault(bill["payeeIdHash"].lower(), {})
            for key in ("recipient", "payeeLabel"):
                if bill.get(key):
                    payee.setdefault(key, bill[key])
    return payees, bills


def receipt_fields(tx, rows, module, rpc_urls):
    failure = None
    for url in rpc_urls:
        try:
            receipt = Rpc(url).call("eth_getTransactionReceipt", [tx])
            if not receipt or receipt.get("transactionHash", "").lower() != tx:
                raise ValueError(f"Missing/mismatched receipt for {tx}")
            if not int(receipt.get("blockHash") or "0x0", 16):
                raise ValueError(f"Receipt is not included: {tx}")
            blocks = {int(r.get("blockNumber", r.get("block", 0))) for r in rows} - {0}
            if blocks and blocks != {int(receipt["blockNumber"], 16)}:
                raise ValueError(f"Receipt block disagrees with evidence: {tx}")
            events = []
            abis = {keccak(signature(event_abi(name))): event_abi(name)
                    for name in ("ObligationPaid", "ActionExecuted")}
            for log in receipt["logs"]:
                if log["address"].lower() != module or not log.get("topics"):
                    continue
                abi = abis.get(log["topics"][0].lower())
                if abi:
                    events.append((abi["name"], decode_event(log, abi)))
            return events
        except Exception as error:
            failure = error
    raise RuntimeError(f"Cannot enrich {tx} from public RPC: {failure}") from failure


def main():
    grouped = defaultdict(list)
    for path in INPUTS:
        data = read(path)
        if isinstance(data, dict) and "receipt" in data and "rows" not in data:
            # A single owner-setup receipt (2-of-2 Safe tx committing bills).
            receipt = data["receipt"]
            data = [{"txHash": receipt.get("transactionHash") or receipt.get("txHash"),
                     "source": data.get("source"), "kind": "OBLIGATIONS_COMMITTED"}]
        for row in data.get("rows", []) if isinstance(data, dict) else data:
            tx = row.get("txHash")
            if not tx:  # Hold decisions have no transaction.
                continue
            if not HASH.fullmatch(tx):
                raise ValueError(f"Invalid txHash in {path}: {tx}")
            if row.get("source") not in {"SCRIPT", "OWNER", "AGENT", "APP"}:
                raise ValueError(f"Missing/invalid recorded source in {path}: {tx}")
            grouped[tx.lower()].append((path, row))

    payees, bills = payee_catalog(read(INPUTS[0])["rows"])
    deployment = read("deployments/84532.json")
    module = deployment["module"].lower()
    rpc_urls = list(dict.fromkeys(filter(None, (
        os.environ.get("BASE_SEPOLIA_RPC"), deployment["rpcUrl"],
        "https://base-sepolia-rpc.publicnode.com",
    ))))
    if int(Rpc(rpc_urls[0]).call("eth_chainId", []), 16) != 84532:
        raise ValueError("Index must be built against Base Sepolia")

    def build(entry):
        tx, records = entry
        sources = {r["source"] for _, r in records}
        if len(sources) != 1:
            raise ValueError(f"Conflicting recorded provenance for {tx}: {sources}")
        kinds = list(dict.fromkeys(r["kind"] for _, r in records))
        result = {"txHash": tx, "source": sources.pop(), "kind": "+".join(kinds),
                  "evidenceFile": records[0][0]}
        planners = set()
        for _, row in records:
            planner = row.get("planner")
            if isinstance(planner, dict):
                planner = planner.get("kind")
            if planner:
                planners.add(planner)
        if len(planners) > 1:
            raise ValueError(f"Conflicting recorded planners for {tx}")
        if planners:
            result["planner"] = planners.pop()

        amounts, recipients, payee_ids, obligation_ids = {}, set(), set(), set()
        for i, (_, row) in enumerate(records):
            event = row.get("expectedEvent") or {}
            fields = event.get("fields", {})
            amount = row.get("amountRaw", row.get("assets", fields.get("amountRaw", fields.get("assets"))))
            if amount is not None:
                amounts[event.get("logIndex", f"row:{i}")] = int(amount)
            recipient = row.get("recipient", fields.get("recipient", fields.get("account")))
            if recipient:
                recipients.add(recipient.lower())
            for key, values in (("payeeIdHash", payee_ids), ("obligationIdHash", obligation_ids)):
                value = row.get(key, fields.get(key))
                if value:
                    values.add(value.lower())

        if not amounts or not recipients:
            events = receipt_fields(tx, [r for _, r in records], module, rpc_urls)
            paid = [f for name, f in events if name == "ObligationPaid"]
            executed = [f for name, f in events if name == "ActionExecuted"]
            if not amounts:
                # Paid and Executed describe the same payment; count it once.
                values = [int(f["amountRaw"]) for f in paid] or [int(f["assets"]) for f in executed]
                if values:
                    amounts["receipt"] = sum(values)
            for fields in paid:
                recipients.add(fields["recipient"].lower())
                payee_ids.add(fields["payeeIdHash"].lower())
                obligation_ids.add(fields["obligationIdHash"].lower())

        for obligation in obligation_ids:
            for bill in bills[obligation]:
                payee_ids.add(bill["payeeIdHash"].lower())
        labels = set()
        for payee_id in payee_ids:
            payee = payees.get(payee_id, {})
            if payee.get("recipient"):
                recipients.add(payee["recipient"].lower())
            if payee.get("payeeLabel"):
                labels.add(payee["payeeLabel"])
        if amounts:
            if any(a < 0 for a in amounts.values()):
                raise ValueError(f"Negative amount in {tx}")
            result["amountRaw"] = str(sum(amounts.values()))
        if len(recipients) == 1:
            recipient = recipients.pop()
            if not ADDRESS.fullmatch(recipient):
                raise ValueError(f"Invalid recipient in {tx}")
            result["recipient"] = recipient
        if len(labels) == 1:
            result["payeeLabel"] = labels.pop()
        return result

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        index = list(pool.map(build, sorted(grouped.items())))
    write_json(ROOT / "evidence/tx_index.json", index)
    print(f"Indexed {len(index)} transactions from {len(INPUTS)} evidence files")


if __name__ == "__main__":
    main()
