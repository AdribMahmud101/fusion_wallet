#!/usr/bin/env python3
"""Generate docs/deliverables/TX_HASHES.md from the recorded evidence.

Reads evidence/tx_index.json (the same index the app's Proof screen uses) and enriches each row from the
evidence files it names (block, scenario, device). Groups every transaction by Topic F operation, using the
same kind -> operation mapping as the app (ProofModels.kt). Offline, deterministic, python3 standard library only.
No provenance is inferred: the source badge is exactly what the index recorded.

    python3 scripts/gen_tx_hashes_md.py            # rewrite docs/deliverables/TX_HASHES.md
    python3 scripts/gen_tx_hashes_md.py --check    # exit 1 if the file is out of date
"""
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INDEX = ROOT / "evidence/tx_index.json"
OUT = ROOT / "docs/deliverables/TX_HASHES.md"
EXPLORER = "https://sepolia.basescan.org/tx/"
EVIDENCE_FILES = (
    "evidence/base-sepolia.json",
    "evidence/agent/golden.json",
    "evidence/demo/runs.json",
    "evidence/app/agent/golden_app.json",
    "evidence/app/guardian_pause.json",
    "evidence/app/agent/owner_setup_receipt.json",
)

ASSET, TRANSACTIONS, RISK, SETUP = "asset", "transactions", "risk", "setup"
GROUPS = (
    (ASSET, "Asset management", "Deposits into and withdrawals from the two pinned vaults, and the owners' funding of the Safe."),
    (TRANSACTIONS, "Transaction execution", "Bill payments (instant, delayed, owner tier) and the queue and approval steps that lead to them."),
    (RISK, "Risk control", "Guardian pause, owner resume, queue cancel, emergency unwind, and attacks the contract rejected on purpose."),
    (SETUP, "Owner and setup", "Safe creation, module enable, policy and bill commitments, payee registration, other owner Safe transactions."),
)
# Same mapping as src/Fintech_Wallet_Agent/.../ui/proof/ProofModels.kt (ProofOperation.operationOf).
ASSET_KINDS = {"DEPOSIT", "WITHDRAW", "REBALANCE", "OWNERALLOCATION"}
TX_KINDS = {"PAYOBLIGATION", "OBLIGATIONPAID", "PAYINSTANT", "PAYDELAYED", "PAYOWNERTIER", "OWNERPAY", "ACTIONQUEUED", "ACTIONAPPROVED"}
RISK_KINDS = {"GUARDIANPAUSE", "PAUSED", "EMERGENCYUNWIND", "OWNERRESUME", "RESUMED", "QUEUECANCELLED", "INTENTIONALREVERT"}
PAYMENT_KINDS = {"PAY_INSTANT", "PAY_DELAYED", "PAY_OBLIGATION"}

KIND_TEXT = {
    "SAFE_CREATION": "Safe v1.4.1 created: two owners, threshold 2",
    "MODULE_ENABLE": "KavachiModule enabled on the Safe",
    "POLICY_CONFIGURED+OBLIGATIONS_COMMITTED+SESSION_STARTED+RESUMED": "Policy configured, bills committed, session started, module resumed (one Safe batch)",
    "OBLIGATIONS_COMMITTED": "Bills committed on chain (Merkle root), reviewed owner setup",
    "SAFE_MULTISIG": "Safe 2-of-2 transaction signed by both owners",
    "PAYEE_ADDED": "Payee registered (cooling period starts)",
    "PAYEE_NEW_BANK": "Changed bank details registered as a new payee (cooling restarts)",
    "DEPOSIT": "Deposit into a vault",
    "WITHDRAW": "Withdraw from a vault",
    "OWNER_ALLOCATION": "Owners fund the Safe (explicit 2-of-2 allocation, not the agent)",
    "PAY_INSTANT": "Bill paid, instant tier",
    "PAY_DELAYED": "Bill paid, delayed tier (after the queue delay)",
    "PAY_OWNER_TIER": "Bill paid, owner-approved tier",
    "OWNER_PAY": "Bill paid directly by the owners (ownerPayObligation)",
    "ACTION_QUEUED": "Delayed action queued",
    "ACTION_APPROVED": "Owners approved an action",
    "QUEUE_CANCELLED": "Queued action cancelled",
    "GUARDIAN_PAUSE": "Guardian pause",
    "OWNER_RESUME": "Owners resumed the module (Safe 2-of-2)",
    "EMERGENCY_UNWIND": "Emergency unwind: vault shares redeemed into the Safe",
    "INTENTIONAL_REVERT": "Attack rejected on purpose by the contract",
}
SOURCE_TEXT = {
    "AGENT": "Broadcast by `agent-runner` with the on-device model (AHEM-G), signed by the session key",
    "APP": "Triggered inside the Android app and signed by the `:signer` process with a Keystore key",
    "OWNER": "Safe 2-of-2 transaction signed by both owners",
    "SCRIPT": "Operator or test script (deterministic planner). Never counted as autonomous",
}


def read(path):
    return json.loads((ROOT / path).read_text())


def token(kind):
    return kind.strip().upper().split("+")[0].strip().replace("_", "").replace(" ", "")


def group_of(kind):
    t = token(kind)
    if t in ASSET_KINDS:
        return ASSET
    if t in TX_KINDS:
        return TRANSACTIONS
    if t in RISK_KINDS:
        return RISK
    return SETUP


def enrich():
    """txHash (lower) -> facts from the evidence files, as recorded."""
    facts = {}

    def walk(node, file):
        if isinstance(node, dict):
            h = node.get("txHash") or node.get("transactionHash")
            if isinstance(h, str) and h.startswith("0x") and len(h) == 66:
                f = facts.setdefault(h.lower(), {})
                for src, dst in (("id", "id"), ("scenario", "scenario"), ("device", "device"), ("choiceLabel", "label"),
                                 ("purpose", "purpose"), ("note", "note")):
                    if isinstance(node.get(src), str):
                        f.setdefault(dst, node[src])
                block = node.get("block") if node.get("block") is not None else node.get("blockNumber")
                if isinstance(block, int):
                    f.setdefault("block", block)
                if node.get("status") in (0, 1):
                    f.setdefault("status", node["status"])
                f.setdefault("file", file)
            for v in node.values():
                walk(v, file)
        elif isinstance(node, list):
            for v in node:
                walk(v, file)

    for path in EVIDENCE_FILES:
        if (ROOT / path).exists():
            walk(read(path), path)
    return facts


def usdc(raw):
    n = int(raw)
    whole, frac = divmod(n, 10**6)
    return f"{whole:,}.{frac:06d}".rstrip("0").rstrip(".")


def describe(row, f):
    kind = row["kind"]
    text = KIND_TEXT.get(kind, kind.replace("_", " ").title())
    if kind == "INTENTIONAL_REVERT" and f.get("scenario"):
        text = f"Attack rejected on purpose: {f['scenario']}"
    if row.get("payeeLabel") and kind in PAYMENT_KINDS | {"OWNER_PAY", "PAY_OWNER_TIER", "PAYEE_ADDED"}:
        text += f" ({row['payeeLabel']})"
    if row.get("payeeLabel") and kind == "PAYEE_NEW_BANK":
        text += f" ({row['payeeLabel']})"
    extras = []
    if f.get("label") and row["source"] in ("AGENT", "APP"):
        extras.append(f"model chose `{f['label']}`")
    if row.get("planner") and row["source"] in ("AGENT", "APP"):
        extras.append(f"planner `{row['planner']}`")
    if f.get("device"):
        extras.append(f["device"].split(",")[0])
    if f.get("id"):
        extras.append(f"evidence `{f['id']}`")
    if f.get("status") == 0:
        extras.append("reverted, status 0")
    return text + (" · " + " · ".join(extras) if extras else "")


def build():
    rows = read("evidence/tx_index.json")
    facts = enrich()
    seen = set()
    for r in rows:
        h = r["txHash"].lower()
        assert h not in seen, f"duplicate hash {h}"
        seen.add(h)

    by_group = {g: [] for g, _, _ in GROUPS}
    for order, r in enumerate(rows):
        f = facts.get(r["txHash"].lower(), {})
        by_group[group_of(r["kind"])].append((f.get("block", 10**12), order, r, f))
    for g in by_group:
        by_group[g].sort(key=lambda x: (x[0], x[1]))

    sources = Counter(r["source"] for r in rows)
    # Sent by the on-device model's session key: recorded source AGENT or APP with an mnn+ahem-g planner.
    ai_exec = [r for r in rows if r["source"] in ("AGENT", "APP") and str(r.get("planner", "")).startswith("mnn+ahem-g")]
    ai_pay = [r for r in ai_exec if r["kind"] in PAYMENT_KINDS]
    have_ai_asset = any(group_of(r["kind"]) == ASSET and r["source"] in ("AGENT", "APP") for r in rows)

    out = []
    w = out.append
    w("# Fusion Wallet: Transaction Hashes")
    w("")
    w("<!-- GENERATED by scripts/gen_tx_hashes_md.py from evidence/tx_index.json. Do not edit by hand. -->")
    w("")
    w("Network: Base Sepolia testnet (chain 84532). Test money only.")
    w(f"Every row is a transaction in `evidence/tx_index.json` ({len(rows)} rows, {len(seen)} unique hashes), "
      "grouped by the Topic F operation it performed. Click a hash to open it on BaseScan.")
    w("To check them yourself with no keys: `bash scripts/judge/verify_all.sh` (see `JUDGES.md`).")
    w("")
    w("## Source badges")
    w("")
    w("The badge is what the index recorded. It is never guessed from the action name or the signer.")
    w("")
    w("| Badge | Meaning | Rows |")
    w("|---|---|---:|")
    for s in ("AGENT", "APP", "OWNER", "SCRIPT"):
        w(f"| **{s}** | {SOURCE_TEXT[s]} | {sources.get(s, 0)} |")
    other = sorted(set(sources) - set(SOURCE_TEXT))
    for s in other:
        w(f"| **{s}** | Recorded in the index | {sources[s]} |")
    w("")
    w("## Summary")
    w("")
    w("| Operation | AGENT | APP | OWNER | SCRIPT | Total |")
    w("|---|---:|---:|---:|---:|---:|")
    for g, title, _ in GROUPS:
        c = Counter(x[2]["source"] for x in by_group[g])
        w(f"| {title} | {c.get('AGENT', 0)} | {c.get('APP', 0)} | {c.get('OWNER', 0)} | {c.get('SCRIPT', 0)} | {len(by_group[g])} |")
    w(f"| **All** | {sources.get('AGENT', 0)} | {sources.get('APP', 0)} | {sources.get('OWNER', 0)} | {sources.get('SCRIPT', 0)} | {len(rows)} |")
    w("")
    n_agent = sources.get("AGENT", 0)
    n_app_ai = sum(1 for r in ai_exec if r["source"] == "APP")
    w(f"- **Sent by the on-device AI's session key: {len(ai_exec)}.** That is {n_agent} `AGENT` and {n_app_ai} `APP` "
      f"(an agent payment signed on the phone). {len(ai_pay)} are bill payments and {len(ai_exec) - len(ai_pay)} "
      "is a queue step. The model chose each from the candidate set.")
    w(f"- The proof screen in the app counts `AGENT` rows only ({n_agent}).")
    w("- `SCRIPT` rows are operator scripts. They show the contract working. They are not claimed as autonomous.")
    w("")

    for g, title, blurb in GROUPS:
        items = by_group[g]
        w(f"## {title} ({len(items)})")
        w("")
        w(blurb)
        w("")
        if g == ASSET and not any(x[2]["source"] in ("AGENT", "APP") for x in items):
            w("**None of these were chosen by the AI.** They are operator and owner actions. See the end of this file for what is pending.")
            w("")
        if not items:
            w("No rows in the index.")
            w("")
            continue
        w("| Source | What | USDC | Block | Transaction |")
        w("|---|---|---:|---:|---|")
        for _, _, r, f in items:
            amount = usdc(r["amountRaw"]) if r.get("amountRaw") else ""
            block = str(f["block"]) if "block" in f else "not recorded"
            h = r["txHash"]
            w(f"| {r['source']} | {describe(r, f)} | {amount} | {block} | [{h}]({EXPLORER}{h}) |")
        w("")

    w("## Not in the index yet")
    w("")
    pending = []
    if not have_ai_asset:
        pending.append("- `[PENDING: K38M]` On-chain deposit and withdraw by the on-device AI's session key. "
                       "They were an owner-mandated rehearsal under a Safe 2-of-2 mandate, not a decision to invest. "
                       "They are not in `evidence/tx_index.json` yet, so they are not listed above.")
        pending.append("- `[PENDING: K47]` A deposit the AI chose on its own because it pays, with no owner mandate.")
    pending.append("- `[PENDING: K41]` An automatic guardian pause triggered by the risk reflex (today's pause rows are manual).")
    for line in pending:
        w(line)
    w("")
    w("## Regenerate")
    w("")
    w("```bash")
    w("python3 scripts/build_tx_index.py         # rebuild evidence/tx_index.json (needs RPC access)")
    w("python3 scripts/gen_tx_hashes_md.py       # rewrite this file (offline)")
    w("python3 scripts/gen_tx_hashes_md.py --check")
    w("```")
    w("")
    return "\n".join(out)


def main():
    text = build()
    if sys.argv[1:] == ["--check"]:
        if not OUT.exists() or OUT.read_text() != text:
            print("TX_HASHES.md is out of date; run scripts/gen_tx_hashes_md.py", file=sys.stderr)
            sys.exit(1)
        print("TX_HASHES.md is up to date")
        return
    if sys.argv[1:]:
        sys.exit(__doc__)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
