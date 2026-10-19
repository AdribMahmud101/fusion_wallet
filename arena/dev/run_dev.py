#!/usr/bin/env python3
"""K20 dev runner — mirrors arena/heldout/run_heldout.py for the tuning dev set.

Evaluates planner performance on arena/dev/ and records results in evidence/results/dev/.
"""
import datetime, json, pathlib, platform, subprocess, sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / "arena"))
import run  # noqa: E402  (arena/run.py — reused unchanged)

OUT = ROOT / "evidence" / "results" / "dev"
run.OUT = OUT  # redirect every helper (execute/core/write) at the dev dir
MANIFEST = "arena/dev/SHA256SUMS"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for p in OUT.iterdir():
        if p.is_file():
            p.unlink()

    checks = (ROOT / MANIFEST).read_text()
    for line in checks.splitlines():
        if not line.strip():
            continue
        digest, name = line.split()
        assert run.sha(ROOT / name) == digest, f"Dev corpus changed: {name}"
    (OUT / "corpus.sha256").write_text(checks)

    corpus = []
    for name in ("benign", "malicious"):
        rows = [json.loads(x) for x in (HERE / f"{name}.jsonl").read_text().splitlines() if x.strip()]
        assert len(rows) == 30 and all(x["class"] == name for x in rows), name
        corpus.extend(rows)
    assert len({x["id"] for x in corpus}) == len(corpus), "duplicate id"
    run.write("corpus.json", corpus)

    print("Building planner-host (release)", flush=True)
    run.execute(["cargo", "build", "--locked", "--release", "--manifest-path",
                 ROOT / "src/fusion-planner/Cargo.toml", "--bin", "planner-host"], "build-planner.log")
    run.core("prepare")
    prepared = json.loads((OUT / "prepared.json").read_text())
    assert len(prepared) == len(corpus)
    with (OUT / "planner-input.jsonl").open("w") as f:
        for p in prepared:
            f.write(json.dumps(p["request"]) + "\n")

    planner = ROOT / "src/fusion-planner/target/release/planner-host"
    for mode in ("ahem-g", "plain"):
        print(f"Running {len(prepared)} planner-host decisions: {mode}", flush=True)
        run.execute([planner, "--eval", OUT / "planner-input.jsonl", "--mode", mode,
                     "--out", OUT / f"planner-{mode}.jsonl"], f"planner-{mode}.log")
    run.core("evaluate")

    modes = {m: json.loads((OUT / f"pipeline-{m}.json").read_text()) for m in ("ahem-g", "plain")}
    metrics = {m: run.stats(rows) for m, rows in modes.items()}

    def per_case(rows):
        out = []
        for r in rows:
            out.append(dict(
                id=r["id"], cls=r["class"], group=None,
                choice=r["validation"]["choiceLabel"],
                reason_code=r["validation"]["reasonCode"],
                retryable=r["validation"]["retryable"],
                raw_valid=r["model"]["rawValid"],
                admitted=r["admitted"],
                violation_admitted=r["violation_admitted"],
                benign_accepted=r["benign_accepted"],
                hold_correct=r["hold_correct"],
                vetoes=r["model"]["ahem"]["g"]["vetoes"],
                aborted=r["model"]["ahem"]["g"]["aborted"]))
        return out

    group_by_id = {r["id"]: r.get("group") for r in corpus}
    decisions = {}
    for m, rows in modes.items():
        pc = per_case(rows)
        for d in pc:
            d["group"] = group_by_id.get(d["id"])
        decisions[m] = pc

    result = dict(
        schema_version=1,
        kind="dev evaluation (tuning set)",
        generated_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        git_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        host=platform.machine(),
        environment="Linux host, local planner-host; no Anvil/forge (off-chain planner ASR/FBR only)",
        corpus=dict(benign=sum(x["class"] == "benign" for x in corpus),
                    malicious=sum(x["class"] == "malicious" for x in corpus),
                    hashes="evidence/results/dev/corpus.sha256",
                    manifest=MANIFEST),
        asr={m: v["asr"] for m, v in metrics.items()},
        fbr={m: v["fbr"] for m, v in metrics.items()},
        metrics=metrics,
        decisions=decisions,
        definitions=dict(
            asr="Malicious cases whose planner final raw -> core validator -> preview admits an "
                "action NOT in the allowed set / all malicious cases. HOLD is safe.",
            fbr="Benign cases with no legitimate admitted proposal / all benign cases (includes HOLD-correct).",
            arms="ahem-g = AHEM grammar/controls on; plain = controls disabled."),
    )
    run.write("latest.json", result)
    markdown(result)
    print(json.dumps({"asr": result["asr"], "fbr": result["fbr"]}, indent=2), flush=True)


def markdown(d):
    def fmt(m, k):
        x = d["metrics"][m][k]
        return f"{x['numerator']}/{x['denominator']} ({x['rate']:.1%})" if x["rate"] is not None else f"{x['numerator']}/{x['denominator']} (n/a)"

    L = ["# K20 — Dev Evaluation", "",
         f"Run: {d['generated_at']} · {d['environment']} · source `{d['git_commit']}`", "",
         "| Arm | Off-chain ASR | FBR (all benign) | FBR (actionable) | Invalid final raw | Vetoes | Aborts |",
         "|---|---|---|---|---|---|---|"]
    for m in ("ahem-g", "plain"):
        mm = d["metrics"][m]
        L.append(f"| {m} | {fmt(m,'asr')} | {fmt(m,'fbr')} | {fmt(m,'actionable_fbr')} | "
                 f"{fmt(m,'invalid_final_raw')} | {mm['vetoes']} | {mm['aborts']} |")
    L += ["", "## Per-arm details"]
    for m in ("ahem-g", "plain"):
        dec = d["decisions"][m]
        blocked = [x for x in dec if x["cls"] == "benign" and not x["benign_accepted"]]
        admits = [x for x in dec if x["cls"] == "malicious" and x["violation_admitted"]]
        bstr = ", ".join(f"{x['id']}({x['group']})" for x in blocked) or "none"
        astr = ", ".join(f"{x['id']}({x['group']})" for x in admits) or "none"
        L.append(f"- **{m}**: {d['asr'][m]['numerator']}/{d['asr'][m]['denominator']} malicious admitted; "
                 f"{d['fbr'][m]['numerator']}/{d['fbr'][m]['denominator']} benign blocked.")
        L.append(f"  - Malicious admitted: {astr}")
        L.append(f"  - Benign blocked (false blocks): {bstr}")
    (OUT / "DEV.md").write_text("\n".join(L) + "\n")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        (OUT / "latest.json").unlink(missing_ok=True)
        run.write("failure.json", {"status": "dev_error", "error": str(e),
                  "time": datetime.datetime.now(datetime.timezone.utc).isoformat()})
        raise
