#!/usr/bin/env python3
"""Read-only, independent verification of committed K17 APP evidence on Base Sepolia."""
import json
from prepare_demo import ROOT, RPC, calldata, cast, rpc, words


def verify(row, manifest):
    assert row['source'] == 'APP' and row['status'] == 1 and row['chainId'] == 84532
    assert rpc(RPC, 'eth_chainId', []) == '0x14a34'
    passport_bytes = (ROOT / row['passport']).read_bytes()
    passport = json.loads(passport_bytes)
    decision_hash = cast('keccak', '0x' + passport_bytes.hex())
    assert decision_hash == row['decisionHash']
    assert row['planner'] == passport['planner']
    assert passport['planner']['kind'] == 'mnn+ahem-g@android'
    assert passport['planner']['host'] == 'device'
    assert passport['planner']['ahem']['g']['mode'] == 'ENFORCED'
    report = json.loads((ROOT / 'evidence/app/agent/runs' / (row['txHash'] + '.json')).read_text())
    assert report['source'] == 'APP' and report['dryRun'] is False
    assert report['passportJson'].encode() == passport_bytes
    assert report['model']['process'] == 'app.fusion.wallet:inference'
    assert report['execution']['signerProcess'] == 'app.fusion.wallet:signer'
    assert report['model']['pid'] != report['execution']['signerPid']
    assert report['model']['rawValid'] and not report['model']['ahem']['g']['aborted']
    receipt = rpc(RPC, 'eth_getTransactionReceipt', [row['txHash']])
    tx = rpc(RPC, 'eth_getTransactionByHash', [row['txHash']])
    assert int(receipt['status'], 16) == 1 and int(tx['chainId'], 16) == 84532
    for obj in (receipt, tx):
        assert obj['from'].lower() == manifest['roles']['session'].lower() == row['from'].lower()
        assert obj['to'].lower() == manifest['module'].lower() == row['to'].lower()
        assert obj['blockHash'] == receipt['blockHash']
    block = rpc(RPC, 'eth_getBlockByNumber', [receipt['blockNumber'], False])
    assert block['hash'] == receipt['blockHash']
    assert decision_hash[2:] in tx['input'].lower()
    stored_receipt = json.loads((ROOT / row['receipt']).read_text())
    stored_tx = json.loads((ROOT / row['transaction']).read_text())
    for key in ('transactionHash', 'blockHash', 'status', 'from', 'to', 'gasUsed', 'logs'):
        assert receipt[key] == stored_receipt[key], 'Stored receipt mismatch: ' + key
    for key in ('hash', 'input', 'from', 'to', 'chainId', 'blockHash'):
        assert tx[key] == stored_tx[key], 'Stored transaction mismatch: ' + key
    action = report['chosenCandidate']['action']
    amount = int(action['assets'])
    event_topic = cast('keccak', 'ActionExecuted(bytes32,uint256,bytes32,uint8,uint256,uint256,uint256,uint256,uint256,uint256,bytes32)')
    # Use the deployed interface's exact event signature; also pin its known topic.
    assert event_topic == '0xec3c5fb2a9f98228404a340d65a7da26d4bb846738b0d1f39b0de18f0a0154e8'
    events = [log for log in receipt['logs'] if log['address'].lower() == manifest['module'].lower()
              and log['topics'] == [event_topic, row['actionDigest'], '0x' + format(int(action['expectedNonce']), '064x'), decision_hash]]
    assert len(events) == 1
    data = words(events[0]['data'])
    assert int(data[0], 16) == action['kind'] and int(data[1], 16) == amount
    assert action['kind'] in (0, 1, 3), 'Unsupported APP action'
    def view(to, data):
        return rpc(RPC, 'eth_call', [{'to': to, 'data': data}, receipt['blockNumber']])
    transfer_topic = cast('keccak', 'Transfer(address,address,uint256)')
    if action['kind'] == 3:
        recipient = action['recipient']
        sender = manifest['safe']
        assert int(view(manifest['module'], calldata('isPaid(bytes32)', action['obligationIdHash'])), 16) == 1
    else:
        assert 0 < amount <= 10_000_000, 'APP asset-management evidence must stay within 10 USDC'
        from owner_allocation import verify as verify_owner
        owner_hash = row['allocationOwnerTx']
        owner = verify_owner(owner_hash, manifest, report['snapshot'])
        assert report['allocationMandate']['ownerTxHash'] == owner_hash
        assert report['chosenCandidate']['reason'] == 'OWNER_ALLOCATION_REHEARSAL:' + owner_hash
        assert passport['reasonCode'] == report['chosenCandidate']['reason']
        assert int(action['expectedNonce']) == owner['depositNonce' if action['kind'] == 0 else 'withdrawNonce']
        assert amount <= int(owner['assetsRaw']) and (action['kind'] != 0 or amount == int(owner['assetsRaw']))
        assert action['deadline'] <= owner['expiresAt']
        saved_owner = json.loads((ROOT / 'evidence/app/agent/K38_owner_allocation_receipt.json').read_text())
        assert saved_owner['source'] == 'OWNER' and saved_owner['txHash'] == owner_hash
        assert saved_owner['receipt']['logs'] == owner['receipt']['logs']
        vault = action['destinationVault'] if action['kind'] == 0 else action['sourceVault']
        assert vault.lower() == manifest['vaultA'].lower(), 'Expected canonical Aave waUSDC'
        sender, recipient = (manifest['safe'], vault) if action['kind'] == 0 else (vault, manifest['safe'])
        # ERC-4626 Deposit/Withdraw is stronger than assumptions about the pool's
        # underlying transfer route. Both owner and receiver must be our Safe.
        signature = 'Deposit(address,address,uint256,uint256)' if action['kind'] == 0 else 'Withdraw(address,address,address,uint256,uint256)'
        topic = cast('keccak', signature)
        vault_events = [log for log in receipt['logs'] if log['address'].lower() == vault.lower()
                        and log.get('topics', [None])[0] == topic]
        assert len(vault_events) == 1, 'Expected exactly one ERC-4626 asset-management event'
        event = vault_events[0]
        safe_topic = '0x' + manifest['safe'][2:].lower().zfill(64)
        assert event['topics'][1:] == [safe_topic] * (2 if action['kind'] == 0 else 3)
        vault_data = words(event['data'])
        assert int(vault_data[0], 16) == amount
        shares = int(vault_data[1], 16)
        if action['kind'] == 0:
            assert int(data[2], 16) == 0 and int(data[3], 16) == shares
            assert shares >= int(action['minSharesOut'])
        else:
            assert int(data[3], 16) == 0 and int(data[2], 16) == shares
            assert 0 < shares <= int(action['maxSharesBurn'])
    if action['kind'] == 3:
        transfers = [log for log in receipt['logs'] if log['address'].lower() == manifest['asset'].lower()
                     and log['topics'] == [transfer_topic, '0x' + sender[2:].lower().zfill(64),
                                           '0x' + recipient[2:].lower().zfill(64)]
                     and int(log['data'], 16) == amount]
        assert len(transfers) == 1
    state = words(view(manifest['module'], calldata('getState()')))
    assert int(state[8], 16) == int(action['expectedNonce']) + 1
    assert int(data[5], 16) == int(state[6], 16)
    assert int(data[6], 16) == int(action['policyEpoch']) == int(state[2], 16)
    assert '0x' + data[7] == report['snapshot']['moduleState']['sessionId']
    cash_after = int(view(manifest['asset'], calldata('balanceOf(address)', manifest['safe'])), 16)
    assert int(data[4], 16) == cash_after
    reserve_delta = amount if action['kind'] == 3 else 0
    assert int(state[6], 16) == int(report['snapshot']['moduleState']['reservedRaw']) - reserve_delta
    return {'status': 'PASS', 'txHash': row['txHash'], 'decisionHash': decision_hash,
            'blockNumber': int(receipt['blockNumber'], 16), 'assetsRaw': str(amount),
            'recipient': recipient, 'from': tx['from'], 'to': tx['to'],
            'checks': ['APP/device provenance', 'distinct Android inference and signer processes',
                       'AHEM-G enforced, legal model output', 'exact passport keccak',
                       'canonical successful live receipt', 'session sender and module destination',
                       'stored receipt and transaction match RPC', 'decisionHash in calldata and ActionExecuted',
                       'USDC payment or canonical ERC-4626 event',
                       'action nonce increment and expected reserve']}


if __name__ == '__main__':
    out = ROOT / 'evidence/app/agent'
    manifest = json.loads((ROOT / 'deployments/84532.json').read_text())
    rows = json.loads((out / 'golden_app.json').read_text())
    assert rows, 'No APP transactions to verify'
    result = {'status': 'PASS', 'network': 'Base Sepolia', 'transactions': [verify(row, manifest) for row in rows]}
    (out / 'm3_verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
