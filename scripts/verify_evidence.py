#!/usr/bin/env python3
"""Independent read-only receipt, event, code and passport verifier; needs Python 3 + cast, no keys."""
import argparse
import json
import sys
import urllib.error
from pathlib import Path
from chain import ROOT, IMPLEMENTATION_SLOT, Rpc, decode_event, event_abi, keccak, signature


def check(condition, message):
    if not condition: raise ValueError(message)


def verify_row(row, rpc):
    check(row['source'] in ['APP','SCRIPT'], 'source must be explicitly APP or SCRIPT')
    if row['source'] == 'SCRIPT':
        check(row.get('planner') and row.get('host'), 'SCRIPT planner and host labels missing')
    receipt = rpc.receipt(row['txHash'])
    tx = rpc.call('eth_getTransactionByHash', [row['txHash']])
    check(receipt['transactionHash'].lower() == row['txHash'].lower(), 'transaction hash mismatch')
    check(int(receipt['status'],16) == row['status'], 'receipt status mismatch')
    check(int(receipt['blockNumber'],16) == row['block'], 'block mismatch')
    check(receipt['blockHash'].lower() == row['blockHash'].lower(), 'block hash mismatch')
    check((tx['to'] or '').lower() == (row['to'] or '').lower(), 'destination mismatch')
    check(tx['input'][:10].lower() == row['selector'].lower(), 'transaction selector mismatch')
    decoded = {}
    expected = row.get('expectedEvent')
    if row['status'] == 1:
        check(expected is not None, 'successful evidence needs an expected event')
        abi = event_abi(expected['name'])
        topic = keccak(signature(abi))
        check(topic == expected['topic0'].lower(), 'event topic does not match source ABI')
        check(signature(abi) == expected['signature'], 'event signature mismatch')
        logs = [log for log in receipt['logs'] if int(log['logIndex'],16) == expected['logIndex']
                and log['address'].lower() == expected['address'].lower()
                and log['topics'] and log['topics'][0].lower() == topic]
        check(len(logs) == 1, 'expected event missing or ambiguous')
        decoded = decode_event(logs[0], abi)
        for key, value in expected['fields'].items():
            check(str(decoded.get(key)).lower() == str(value).lower(), 'event field mismatch: ' + key)
    else:
        selector = row.get('expectedRevertSelector')
        check(selector is not None, 'revert evidence missing expected custom error')
        error_data = None
        try:
            trace = rpc.call('debug_traceTransaction', [row['txHash'], {'tracer':'callTracer'}])
            error_data = trace.get('output')
        except (RuntimeError, urllib.error.URLError, TimeoutError):
            pass
        if not error_data:
            call = {k:tx[k] for k in ['from','to','value'] if tx.get(k) is not None}
            call['data'] = tx['input']
            try:
                rpc.call('eth_call', [call, hex(row['block']-1)])
            except RuntimeError as error:
                # Providers wrap revert data differently; require the exact selector in their JSON error.
                check(selector.lower() in str(error).lower(), 'revert replay returned a different error')
            else:
                raise ValueError('prior-block replay succeeded; cannot independently prove expected revert')
        else:
            check(error_data[:10].lower() == selector.lower(), 'trace custom error mismatch')
    decision = row.get('decisionHash')
    if expected and expected['name'] in ['ActionExecuted','ActionQueued']:
        check(decision is not None, 'agent event must publish decisionHash')
    if decision:
        check(row.get('passport'), 'decision passport missing')
        passport_path = (ROOT / row['passport']).resolve()
        check(passport_path.is_relative_to(ROOT / 'evidence'), 'passport must be under evidence/')
        raw = passport_path.read_bytes()
        passport = json.loads(raw)
        check(keccak('0x'+raw.hex()) == decision.lower(), 'passport bytes hash mismatch')
        if passport.get('snapshot'):
            snapshot=passport['snapshot']
            block=rpc.call('eth_getBlockByNumber',[hex(int(snapshot['block'])),False])
            check(block is not None and block['hash'].lower()==snapshot['hash'].lower(),'passport snapshot block hash mismatch')
        if decoded: check(decoded.get('decisionHash','').lower() == decision.lower(), 'event decisionHash mismatch')
        digest = decoded.get('actionDigest', decoded.get('digest'))
        if digest: check(row.get('actionDigest','').lower() == digest.lower(), 'event actionDigest mismatch')
        if passport.get('actionDigest'):
            check(passport['actionDigest'].lower() == row['actionDigest'].lower(), 'passport actionDigest mismatch')
        elif passport.get('action'):
            # A final digest includes decisionHash, so embedding it in its own hashed passport is circular.
            # Reconstruct it from the published pre-decision action and domain instead.
            from evidence_actions import action_digest
            action = dict(passport['action'], decisionHash=decision)
            if passport.get('actionDigestWithoutDecision'):
                without_decision=dict(action,decisionHash='0x'+'00'*32)
                unsigned=action_digest(passport['chainId'],passport['safe'],passport['module'],
                                       passport['mandateHash'],passport['sessionId'],without_decision)
                check(unsigned==passport['actionDigestWithoutDecision'].lower(),'pre-decision action digest mismatch')
            reconstructed = action_digest(passport['chainId'], passport['safe'], passport['module'],
                                          passport['mandateHash'], passport['sessionId'], action)
            check(reconstructed.lower() == row['actionDigest'].lower(), 'passport action/domain digest mismatch')
            from evidence_actions import ACTION_TYPE, encode_action
            direct_selectors={keccak(name+'('+ACTION_TYPE+',bytes32[])')[:10] for name in ['execute','queueAction']}
            check(tx['to'].lower()==passport['module'].lower(),'agent transaction targets another module')
            check(tx['input'][:10].lower() in direct_selectors,'agent selector must be execute or queueAction')
            check(tx['input'][10:10+14*64].lower()==encode_action(action)[2:].lower(),'signed action input differs from passport')
        else:
            raise ValueError('passport must bind an actionDigest or a complete reconstructable action')
    return receipt


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('evidence', type=Path)
    p.add_argument('--manifest', type=Path, default=ROOT / 'deployments/84532.json')
    p.add_argument('--rpc-url', default='https://sepolia.base.org')
    args = p.parse_args()
    failures = []
    try:
        doc = json.loads(args.evidence.read_text())
        manifest = json.loads(args.manifest.read_text())
        rpc = Rpc(args.rpc_url)
        chain_id = int(rpc.call('eth_chainId', []),16)
        check(chain_id == doc['chainId'] == manifest['chainId'], 'chain mismatch')
        check(manifest.get('status') in ['DEPLOYED','SETUP_COMPLETE'], 'manifest does not describe an actual deployment')
        check(bool(doc['rows']), 'no receipt evidence exists')
        for name in ['safe','module','asset','vaultA','vaultB','sanctionsList']:
            check(manifest[name].lower()==manifest['contracts'][name]['address'].lower(),'application address alias mismatch: '+name)
        for name,item in manifest['contracts'].items():
            check(manifest['codeHashes'][name].lower()==item['runtimeCodeHash'].lower(),'application code hash alias mismatch: '+name)
        safe=manifest['contracts']['safe']['address']
        threshold=rpc.call('eth_call',[{'to':safe,'data':keccak('getThreshold()')[:10]},'latest'])
        check(int(threshold,16)==manifest['threshold']==2,'Safe must retain threshold two')
        owners=rpc.call('eth_call',[{'to':safe,'data':keccak('getOwners()')[:10]},'latest'])[2:]
        owner_count=int(owners[64:128],16)
        actual_owners={'0x'+owners[128+i*64:192+i*64][-40:] for i in range(owner_count)}
        check(actual_owners=={a.lower() for a in manifest['owners']},'Safe owners differ from manifest')
        singleton=rpc.call('eth_call',[{'to':safe,'data':keccak('masterCopy()')[:10]},'latest'])
        check(('0x'+singleton[-40:]).lower()==manifest['contracts']['safeSingleton']['address'].lower(),'Safe singleton changed')
        enabled_data=keccak('isModuleEnabled(address)')[:10]+manifest['contracts']['module']['address'][2:].zfill(64)
        enabled=rpc.call('eth_call',[{'to':safe,'data':enabled_data},'latest'])
        check(int(enabled,16)==1,'Kavachi module is no longer enabled')
        for name, contract in manifest['contracts'].items():
            code = rpc.call('eth_getCode', [contract['address'], 'latest'])
            check(code != '0x', 'missing runtime code: ' + name)
            check(keccak(code) == contract['runtimeCodeHash'].lower(), 'runtime hash mismatch: ' + name)
            if contract.get('implementation'):
                slot = rpc.call('eth_getStorageAt', [contract['address'], IMPLEMENTATION_SLOT, 'latest'])
                check(('0x'+slot[-40:]).lower() == contract['implementation'].lower(), 'implementation changed: ' + name)
                if contract.get('implementationCodeHash'):
                    logic=rpc.call('eth_getCode',[contract['implementation'],'latest'])
                    check(keccak(logic)==contract['implementationCodeHash'].lower(),'implementation code hash changed: '+name)
            print('PASS code', name, contract['address'])
    except Exception as error:
        print('FAIL', str(error)); return 1
    for row in doc['rows']:
        try:
            verify_row(row, rpc)
            print('PASS', row['id'], row['kind'], row['txHash'])
        except Exception as error:
            failures.append(row['txHash'])
            print('FAIL', row.get('id'), row.get('txHash'), str(error))
    print(f"{len(doc['rows'])-len(failures)}/{len(doc['rows'])} evidence rows passed")
    return 1 if failures else 0


if __name__ == '__main__': sys.exit(main())
