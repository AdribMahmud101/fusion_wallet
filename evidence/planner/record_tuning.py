#!/usr/bin/env python3
"""Archive a completed, unmodified arena run before the next tuning experiment."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
RESULTS = ROOT / 'evidence/results'
report = json.loads((RESULTS / 'latest.json').read_text())
name = sys.argv[1]
if not name or Path(name).name != name:
    raise SystemExit('Use one experiment directory name')
destination = ROOT / 'evidence/planner/tuning' / name
destination.mkdir()
for source in ('src/fusion-planner/src/run.rs', 'src/fusion-core/src/planner_io.rs',
               'src/fusion-core/src/candidates.rs', 'src/fusion-planner/src/session.rs',
               'src/fusion-planner/src/tokenizer_bridge.cpp', 'src/fusion-planner/src/lib.rs'):
    data = (ROOT / source).read_bytes()
    assert hashlib.sha256(data).hexdigest() == report['hashes'][source], source
    (destination / Path(source).name).write_bytes(data)
for filename in ('latest.json', 'planner-input.jsonl', 'planner-ahem-g.jsonl',
                 'planner-plain.jsonl', 'pipeline-ahem-g.json', 'pipeline-plain.json',
                 'corpus.sha256'):
    shutil.copyfile(RESULTS / filename, destination / filename)
(destination / 'source.diff').write_bytes(subprocess.check_output(
    ['git', 'diff', '--', 'src/fusion-planner', 'src/fusion-core/src/planner_io.rs',
     'src/fusion-core/src/candidates.rs'], cwd=ROOT))
print(json.dumps(report['metrics'], indent=2))
