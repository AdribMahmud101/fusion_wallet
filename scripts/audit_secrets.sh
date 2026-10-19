#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Grep inventory includes legitimate transaction/code/decision hashes; never print matches.
python3 - <<'PY'
import os, pathlib, re, subprocess, sys
root = pathlib.Path.cwd()
env_file = pathlib.Path('/root/projects/fusion_wallet/.env.local')
secrets = {v.strip().lower().removeprefix('0x') for k,v in os.environ.items()
           if k.endswith('PRIVATE_KEY') and re.fullmatch(r'(?:0x)?[0-9a-fA-F]{64}', v.strip())}
if env_file.exists():
    for line in env_file.read_text().splitlines():
        if 'PRIVATE_KEY' in line and '=' in line:
            value = line.split('=',1)[1].strip().strip('"\'').lower().removeprefix('0x')
            if re.fullmatch('[0-9a-f]{64}', value): secrets.add(value)
if not secrets:
    sys.exit('FAIL: no known keys available to audit against (load the secret environment)')
paths = subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z']).split(b'\0')
literal = re.compile(rb'PRIVATE_KEY\s*=\s*["\']?(?:0x)?[0-9a-fA-F]{64}', re.I)
failed = []
count = 0
def scan(label, data):
    global count
    lower = data.lower()
    count += len(re.findall(rb'0x[0-9a-fA-F]{64}', data))
    if literal.search(data) or any(key.encode() in lower for key in secrets):
        failed.append(label)
for raw in paths:
    if not raw: continue
    p = root / os.fsdecode(raw)
    if p.is_file(): scan(os.fsdecode(raw), p.read_bytes())
# Check every blob reachable from this branch, including previous versions and deleted files.
objects = subprocess.check_output(['git','rev-list','--objects','HEAD']).splitlines()
for entry in objects:
    oid = entry.split(b' ',1)[0].decode()
    if subprocess.check_output(['git','cat-file','-t',oid]).strip() == b'blob':
        scan('history blob '+oid, subprocess.check_output(['git','cat-file','blob',oid]))
if failed:
    print('FAIL: secret detected in: ' + ', '.join(failed))
    sys.exit(1)
print(f'PASS: working tree and reachable branch history contain no configured private keys or literal PRIVATE_KEY assignments; inspected {count} hash-shaped values (public hashes are allowed).')
PY
