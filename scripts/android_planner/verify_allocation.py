#!/usr/bin/env python3
"""Independently verify the OWNER mandate and both APP decisions, keylessly."""
import json
from owner_allocation import ROOT, RPC, rpc, calldata, verify as verify_owner
from verify_app import verify as verify_app

def verify_round_trip(rows, manifest, full=True):
    selected=[r for r in rows if r.get('id','').startswith('K38-APP-')]
    assert len(selected)==2, 'K38 requires exactly two independently captured APP asset-management transactions'
    deposit=next(r for r in selected if r['kind']=='DEPOSIT')
    withdrawal=next(r for r in selected if r['kind']=='WITHDRAW')
    assert deposit['allocationOwnerTx']==withdrawal['allocationOwnerTx']
    owner=verify_owner(deposit['allocationOwnerTx'],manifest)
    assert deposit['blockNumber']<withdrawal['blockNumber']
    runs=[json.loads((ROOT/'evidence/app/agent/runs'/f'{r["txHash"]}.json').read_text()) for r in (deposit,withdrawal)]
    amounts=[int(r['chosenCandidate']['action']['assets']) for r in runs]
    assert amounts[0]==int(owner['assetsRaw'])==10_000_000 and 0<amounts[1]<=amounts[0]
    assert [int(r['chosenCandidate']['action']['expectedNonce']) for r in runs]==[owner['depositNonce'],owner['withdrawNonce']]
    assert runs[0]['model']['process']==runs[1]['model']['process']=='app.fusion.wallet:inference'
    assert runs[0]['requestId']!=runs[1]['requestId']
    tag=hex(withdrawal['blockNumber'])
    def view(to,data): return int(rpc(RPC,'eth_call',[{'to':to,'data':data},tag]),16)
    cash=view(manifest['asset'],calldata('balanceOf(address)',manifest['safe']))
    shares=view(manifest['vaultA'],calldata('balanceOf(address)',manifest['safe']))
    assets=view(manifest['vaultA'],calldata('convertToAssets(uint256)',shares))
    assert cash==int(runs[0]['snapshot']['safeCash'])-amounts[0]+amounts[1]
    index=json.loads((ROOT/'evidence/tx_index.json').read_text())
    indexed={row['txHash']:row for row in index}
    assert indexed[owner['txHash']]['source']=='OWNER' and indexed[owner['txHash']]['kind']=='ALLOCATION_MANDATE'
    for row,amount in zip((deposit,withdrawal),amounts):
        entry=indexed[row['txHash']]
        assert entry['source']=='APP' and entry['planner']=='mnn+ahem-g@android' and entry['amountRaw']==str(amount)
    verified=[verify_app(row,manifest) for row in (deposit,withdrawal)] if full else []
    return {'status':'PASS','network':'Base Sepolia','device':deposit['device'],'ownerTxHash':owner['txHash'],
        'ownerSource':'OWNER','planner':'mnn+ahem-g@android','depositTxHash':deposit['txHash'],'withdrawTxHash':withdrawal['txHash'],
        'depositAssetsRaw':str(amounts[0]),'withdrawAssetsRaw':str(amounts[1]),'safeCashAfterRaw':str(cash),
        'vaultSharesAfterRaw':str(shares),'vaultAssetsAfterRaw':str(assets),'transactions':verified,
        'purpose':'Explicitly owner-authorized gas-paying allocation rehearsal; ordinary yield economics unchanged'}

if __name__=='__main__':
    m=json.loads((ROOT/'deployments/84532.json').read_text())
    rows=json.loads((ROOT/'evidence/app/agent/golden_app.json').read_text())
    result=verify_round_trip(rows,m)
    (ROOT/'evidence/app/agent/K38_verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))
