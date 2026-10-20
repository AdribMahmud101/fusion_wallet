#!/usr/bin/env python3
"""Replay the committed frozen model choices through the current core; no inference."""
import json
import pathlib
import subprocess
import tempfile

import run

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASELINE = json.loads((ROOT / 'arena/baselines/k36.json').read_text())


def main():
    with tempfile.TemporaryDirectory(prefix='k36-frozen-replay-') as temp:
        run.OUT = pathlib.Path(temp)
        corpus = []
        for name in ('benign', 'malicious'):
            corpus += [json.loads(line) for line in (ROOT / f'arena/corpus/{name}.jsonl').read_text().splitlines()]
        run.write('corpus.json', corpus)
        for arm in ('ahem-g', 'plain'):
            data = subprocess.check_output(['git', 'show', BASELINE['source_commit'] +
                                            f':evidence/results/planner-{arm}.jsonl'], cwd=ROOT)
            (run.OUT / f'planner-{arm}.jsonl').write_bytes(data)
        run.core('prepare')
        run.core('evaluate')
        rows = {arm: json.loads((run.OUT / f'pipeline-{arm}.json').read_text())
                for arm in ('ahem-g', 'plain')}
        metrics = {arm: run.stats(rs) for arm, rs in rows.items()}
        result = dict(kind='fixed-raw frozen replay; no inference', source_commit=BASELINE['source_commit'],
                      before=BASELINE['frozen']['metrics'], metrics=metrics,
                      decisions={arm: [dict(id=r['id'], choice=r['validation']['choiceLabel'],
                                            accepted=r['benign_accepted'], admitted=r['admitted'],
                                            failures=[c for c in (r['preview'] or {}).get('checks', [])
                                                      if c['status'] == 'FAIL']) for r in rs]
                                 for arm, rs in rows.items()})
        assert all(m['asr']['numerator'] == 0 for m in metrics.values())
        (ROOT / 'evidence/results/frozen-replay.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(metrics, indent=2))


if __name__ == '__main__':
    main()
