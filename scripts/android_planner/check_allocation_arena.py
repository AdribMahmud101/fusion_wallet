#!/usr/bin/env python3
"""Rerun the frozen arena without overwriting historical or other-card results.

The archive preserves the arena's canonical evidence/results paths. Configure
FUSION_MODEL_DIR and MNN_LIB_DIR as in arena/run_arena.sh. No remote broadcasts.
"""
import json, pathlib, sys, tarfile, tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'arena'))
import run as arena

def main():
    with tempfile.TemporaryDirectory(prefix='K38-arena-') as tmp:
        arena.OUT=pathlib.Path(tmp)/'results'
        arena.main()
        result=json.loads((arena.OUT/'latest.json').read_text())
        assert result['summary']['stopped']==24 and result['summary']['escaped']==0
        assert all(value['numerator']==0 for value in result['asr'].values())
        out=ROOT/'evidence/app/agent'
        with tarfile.open(out/'K38_arena.tar.gz','w:gz') as archive:
            archive.add(arena.OUT,arcname='evidence/results')
        summary={'summary':result['summary'],'asr':result['asr'],'fbr':result['fbr'],
            'archive':'evidence/app/agent/K38_arena.tar.gz','generatedAt':result['generated_at'],'hashes':result['hashes']}
        (out/'K38_arena_summary.json').write_text(json.dumps(summary,indent=2)+'\n')

if __name__=='__main__': main()
