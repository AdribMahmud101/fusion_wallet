#!/usr/bin/env python3
"""Append receipt-backed SCRIPT evidence. Refuses missing/mismatched receipts."""
import argparse
import json
import re
from pathlib import Path
from chain import ROOT, Rpc, decode_event, event_abi, keccak, signature, write_json


def collect(path, rpc, *, tx, evidence_id, kind, scenario, event=None, emitter=None,
            fields=None, status=1, decision_hash=None, passport=None, revert_selector=None):
    if not re.fullmatch(r'0x[0-9a-fA-F]{64}', tx): raise ValueError('Invalid transaction hash')
    receipt = rpc.receipt(tx)
    transaction = rpc.call('eth_getTransactionByHash', [tx])
    if int(receipt['status'], 16) != status: raise ValueError('Unexpected receipt status')
    chain_id = int(rpc.call('eth_chainId', []), 16)
    path = Path(path)
    doc = json.loads(path.read_text()) if path.exists() else {'chainId': chain_id, 'rows': []}
    if doc['chainId'] != chain_id: raise ValueError('Evidence chain mismatch')
    row = {'id': evidence_id, 'txHash': tx.lower(), 'block': int(receipt['blockNumber'], 16),
           'blockHash': receipt['blockHash'], 'kind': kind, 'scenario': scenario,
           'status': status, 'decisionHash': decision_hash, 'source': 'SCRIPT',
           'planner': 'deterministic', 'host': 'linux-host',
           'to': transaction['to'], 'selector': transaction['input'][:10],
           'expectedEvent': None}
    if event:
        if not emitter: raise ValueError('Expected event requires a pinned emitter')
        abi = event_abi(event)
        topic = keccak(signature(abi))
        matches = [log for log in receipt['logs'] if log['address'].lower() == emitter.lower()
                   and log['topics'] and log['topics'][0].lower() == topic]
        expected = fields or {}
        matches = [log for log in matches if all(str(decode_event(log, abi).get(k)).lower() == str(v).lower()
                                                 for k, v in expected.items())]
        if len(matches) != 1: raise ValueError(f'Expected exactly one {event} with fields {expected}; found {len(matches)}')
        log = matches[0]
        decoded = decode_event(log, abi)
        row['expectedEvent'] = {'name': event, 'signature': signature(abi), 'topic0': topic,
                                'address': emitter, 'logIndex': int(log['logIndex'], 16), 'fields': decoded}
        if 'decisionHash' in decoded:
            if decision_hash and decision_hash.lower() != decoded['decisionHash'].lower():
                raise ValueError('Decision hash does not match the emitted event')
            row['decisionHash'] = decoded['decisionHash']
        if 'actionDigest' in decoded: row['actionDigest'] = decoded['actionDigest']
        elif event == 'ActionQueued': row['actionDigest'] = decoded['digest']
    if passport: row['passport'] = str(passport)
    if revert_selector: row['expectedRevertSelector'] = revert_selector
    if status == 0 and not revert_selector: raise ValueError('Revert row needs an expected custom error selector')
    if row['decisionHash'] and not passport: raise ValueError('Agent decision needs a published passport')
    def identity(r):
        return (r['id'],r['txHash'],r['kind'],(r.get('expectedEvent') or {}).get('logIndex'))
    if any(identity(r) == identity(row) for r in doc['rows']): return row
    doc['rows'].append(row)
    write_json(path, doc)
    return row


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('tx_hash'); p.add_argument('--id', required=True); p.add_argument('--kind', required=True)
    p.add_argument('--scenario', required=True); p.add_argument('--event'); p.add_argument('--emitter')
    p.add_argument('--field', action='append', default=[]); p.add_argument('--decision-hash'); p.add_argument('--passport')
    p.add_argument('--status', type=int, choices=[0,1], default=1); p.add_argument('--revert-selector')
    p.add_argument('--file', default=str(ROOT / 'evidence/base-sepolia.json'))
    p.add_argument('--rpc-url', default='https://sepolia.base.org')
    args = p.parse_args()
    row = collect(args.file, Rpc(args.rpc_url), tx=args.tx_hash, evidence_id=args.id, kind=args.kind,
        scenario=args.scenario, event=args.event, emitter=args.emitter, fields=dict(x.split('=',1) for x in args.field),
        status=args.status, decision_hash=args.decision_hash, passport=args.passport, revert_selector=args.revert_selector)
    print(row['id'], row['txHash'], 'receipt confirmed')


if __name__ == '__main__': main()
