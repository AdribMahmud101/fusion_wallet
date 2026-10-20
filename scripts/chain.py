"""Small read-only JSON-RPC / Foundry helpers. No third-party Python packages."""
import functools
import json
import os
from pathlib import Path
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / 'src/contracts'
PUBLIC_RPC = 'https://sepolia.base.org'
IMPLEMENTATION_SLOT = '0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc'


def redact(text):
    for k, v in os.environ.items():
        if k.endswith('PRIVATE_KEY') and v:
            text = text.replace(v, '[REDACTED]').replace(v.removeprefix('0x'), '[REDACTED]')
            try: text = text.replace(str(int(v,16)), '[REDACTED]')
            except ValueError: pass
    return text


def run(args, cwd=CONTRACTS):
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(redact(result.stdout + result.stderr))
    return result.stdout.strip()


@functools.lru_cache(maxsize=2048)
def keccak(data):
    # Same input rules as `cast keccak`: 0x-hex is hashed as bytes, anything
    # else as UTF-8 text. Pure Python so verification needs no Foundry.
    from keccak256 import keccak256_hex
    if isinstance(data, str) and data.startswith('0x'):
        return keccak256_hex(bytes.fromhex(data[2:]))
    return keccak256_hex(data.encode() if isinstance(data, str) else data)


class Rpc:
    def __init__(self, url=PUBLIC_RPC):
        self.url = url

    def call(self, method, params):
        request = urllib.request.Request(self.url, json.dumps({
            'jsonrpc': '2.0', 'id': 1, 'method': method, 'params': params
        }).encode(), {'Content-Type': 'application/json', 'User-Agent': 'curl/8.0'})
        for attempt in range(4):
            try:
                with urllib.request.urlopen(request, timeout=30) as response:
                    body = json.load(response)
                if 'error' in body:
                    raise RuntimeError(json.dumps(body['error']))
                return body['result']
            except urllib.error.HTTPError as error:
                if error.code in [400,401,403,404,405] or attempt==3:raise
                time.sleep(1+attempt)
            except (urllib.error.URLError, TimeoutError):
                if attempt == 3: raise
                time.sleep(1 + attempt)

    def receipt(self, tx):
        # Base can expose a Flashblocks preconfirmation with blockHash == 0 before inclusion.
        # Never persist that placeholder as an on-chain block hash.
        for attempt in range(30):
            result = self.call('eth_getTransactionReceipt', [tx])
            if result is not None and int(result.get('blockHash') or '0x0',16):
                block=self.call('eth_getBlockByNumber',[result['blockNumber'],False])
                if block and block['hash'].lower()==result['blockHash'].lower():return result
            if attempt < 29:time.sleep(1)
        raise ValueError('Receipt not yet included in a canonical block: ' + tx)


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix(path.suffix + '.tmp')
    temp.write_text(json.dumps(value, indent=2) + '\n')
    temp.replace(path)


def canonical_type(item):
    if item['type'].startswith('tuple'):
        return '(' + ','.join(canonical_type(i) for i in item['components']) + ')' + item['type'][5:]
    return item['type']


def signature(item):
    return item['name'] + '(' + ','.join(canonical_type(i) for i in item['inputs']) + ')'


def artifact(name):
    files = list((CONTRACTS / 'out').glob(f'{name}.sol/{name}.json'))
    if len(files) != 1: raise ValueError('Build contract artifact first: ' + name)
    return json.loads(files[0].read_text())


def event_abi(name):
    # Source ABI snapshot ships with the verifier; no compiler/build is needed by judges.
    return json.loads((ROOT / 'scripts/event_abi.json').read_text())[name]


def decode_event(log, abi):
    topics = iter(log['topics'][1:])
    data = log['data'][2:]
    offset = 0
    fields = {}
    for item in abi['inputs']:
        if item['indexed']:
            value = next(topics)
        else:
            value = '0x' + data[offset:offset + 64]
            offset += 64
        kind = item['type']
        if kind == 'address[]' and not item['indexed']:
            start = int(value, 16) * 2
            count = int(data[start:start + 64], 16)
            value = ['0x' + data[start + 64 + i*64:start + 128 + i*64][-40:] for i in range(count)]
        elif kind.startswith('uint') or kind.startswith('int'): value = str(int(value, 16))
        elif kind == 'bool': value = bool(int(value, 16))
        elif kind == 'address': value = '0x' + value[-40:]
        fields[item['name']] = value
    return fields
