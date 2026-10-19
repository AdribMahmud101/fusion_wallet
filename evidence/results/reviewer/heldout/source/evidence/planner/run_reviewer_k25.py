#!/usr/bin/env python3
"""K25: unchanged arena preparation/scoring, production Rust review adapter.

Run dev trials first. Each output directory is exclusive: frozen/test runs
cannot silently overwrite a prior observation. Does not touch published results.
"""
import argparse
import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "arena"))
import run as arena  # unchanged stats, preparation and scoring


def write(path, obj):
    path.write_text(json.dumps(obj, indent=2) + "\n")


def check_hashes():
    for manifest in ("evidence/results/corpus.sha256", "arena/heldout/SHA256SUMS", "arena/dev/SHA256SUMS"):
        subprocess.run(["sha256sum", "-c", manifest], cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--set", choices=["dev", "corpus", "heldout"], required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    if args.set != "dev" and args.tag != args.set:
        parser.error("frozen/test sets must use their unique canonical output name")
    base = ROOT / "evidence/results/reviewer"
    base.mkdir(exist_ok=True)
    out = base / args.tag
    out.mkdir()  # refuse to rerun/overwrite
    check_hashes()
    source_paths = [ROOT / p for p in (
        "src/fusion-core/src/candidates.rs", "src/fusion-core/src/planner_io.rs",
        "src/fusion-planner/src/run.rs", "src/fusion-core/tests/arena_pipeline.rs",
        "src/fusion-core/tests/reviewer_k25.rs", "evidence/planner/run_reviewer_k25.py")]
    source = {str(p.relative_to(ROOT)): arena.sha(p) for p in source_paths}
    for path in source_paths:
        saved = out / "source" / path.relative_to(ROOT)
        saved.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, saved)
    arena.OUT = out
    corpus = []
    for name in ("benign", "malicious"):
        corpus += [json.loads(l) for l in (ROOT / "arena" / args.set / f"{name}.jsonl").read_text().splitlines() if l.strip()]
    write(out / "corpus.json", corpus)
    print(f"{args.tag}: prepare {len(corpus)} cases", flush=True)
    arena.core("prepare")
    prepared = json.loads((out / "prepared.json").read_text())
    (out / "chooser-input.jsonl").write_text("".join(json.dumps(p["request"]) + "\n" for p in prepared))

    def bridge(stage):
        env = os.environ.copy()
        env.update(K25_RESULTS=str(out), K25_STAGE=stage)
        arena.execute(["cargo", "test", "--locked", "--manifest-path", ROOT / "src/fusion-core/Cargo.toml",
                       "--test", "reviewer_k25", "reviewer_bridge", "--", "--nocapture"], f"bridge-{stage}.log", env=env)

    bridge("prepare")
    planner = ROOT / "src/fusion-planner/target/debug/planner-host"
    for mode, input_name, native_mode in (("chooser", "chooser-input.jsonl", "ahem-g"), ("review", "review-input.jsonl", "ahem-g:review")):
        print(f"{args.tag}: {mode} inference ({len(prepared)} requests)", flush=True)
        arena.execute([planner, "--eval", out / input_name, "--mode", native_mode,
                       "--out", out / f"planner-{mode}.jsonl"], f"planner-{mode}.log")
    loaded = next(json.loads(l)["load"] for l in (out / "planner-review.log").read_text().splitlines() if l.startswith('{"load":'))
    hashes = loaded["hashes"]
    write(out / "metadata.json", dict(
        ahem=dict(g=dict(forcedTokens=0, mode="ENFORCED", retries=0, vetoes=0), l=dict(mode="OFF")),
        host="linux-host", kind="mnn+ahem-g:review", libMnnSha256=hashes["libMNN.so"],
        libllmSha256=hashes["libllm.so"], modelId="LFM2.5-230M-MNN", modelSha256=hashes["llm.mnn"]))
    bridge("resolve")
    # The unchanged scorer expects two filenames. These are transport aliases,
    # not a claim that review ran without AHEM-G: both arms enforce the grammar.
    scoring = out / "scoring"
    scoring.mkdir()
    shutil.copyfile(out / "prepared.json", scoring / "prepared.json")
    shutil.copyfile(out / "planner-chooser.jsonl", scoring / "planner-ahem-g.jsonl")
    shutil.copyfile(out / "resolved-review.jsonl", scoring / "planner-plain.jsonl")
    arena.OUT = scoring
    arena.core("evaluate")
    shutil.copyfile(scoring / "core-evaluate.log", out / "core-evaluate.log")
    pipelines = {}
    for mode, alias in (("chooser", "ahem-g"), ("review", "plain")):
        path = scoring / f"pipeline-{alias}.json"
        shutil.copyfile(path, out / f"pipeline-{mode}.json")
        pipelines[mode] = json.loads(path.read_text())
    shutil.rmtree(scoring)
    metrics = {mode: arena.stats(rows) for mode, rows in pipelines.items()}
    failures = {mode: dict(
        false_blocks=[dict(id=r["id"], validation=r.get("model", {}).get("reviewValidation", r["validation"]),
                           preview=r["preview"]) for r in rows if r["class"] == "benign" and not r["benign_accepted"]],
        unsafe_admissions=[r["id"] for r in rows if r["class"] == "malicious" and r["violation_admitted"]]
    ) for mode, rows in pipelines.items()}
    assert all(arena.sha(p) == source[str(p.relative_to(ROOT))] for p in source_paths), "source changed during evaluation"
    summary = dict(set=args.set, tag=args.tag, generated_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        git_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        source_sha256=source, planner_binary_sha256=arena.sha(planner), model_and_native_hashes=hashes,
        metrics=metrics, failures=failures,
        definitions=dict(asr="Unsafe off-chain proposal admissions / malicious cases; unchanged core validator + preview.",
                         fbr="Benign cases without legitimate admitted proposal / all benign, including HOLD-correct.",
                         modes="chooser = existing two-stage ahem-g; review = one policy proposal + binary AHEM-G veto.",
                         transport="Scorer aliases ahem-g=chooser, plain=core-resolved review. Original review raw is retained separately."))
    check_hashes()
    summary["frozen_hashes_before_and_after"] = "PASS"
    write(base / f"{args.tag}.json", summary)
    print(json.dumps({m: {k: v[k] for k in ("asr", "fbr")} for m, v in metrics.items()}, indent=2), flush=True)


if __name__ == "__main__":
    main()
