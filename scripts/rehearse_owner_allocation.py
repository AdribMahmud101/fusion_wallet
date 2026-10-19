#!/usr/bin/env python3
"""Test the owner seed allocation against a Base Sepolia fork, exclusively on Anvil 31337."""
import argparse
import json
import os
from chain import ROOT, Rpc, decode_event, event_abi, keccak, run, signature
from deploy import read_manifest, script, set_environment


def balance(rpc,token,owner):
    return int(run(['cast','call',token,'balanceOf(address)(uint256)',owner,'--rpc-url',rpc.url]).split()[0])


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--rpc-url',default='http://127.0.0.1:18546')
    a=p.parse_args();rpc=Rpc(a.rpc_url)
    if int(rpc.call('eth_chainId',[]),16)!=31337:raise ValueError('Owner allocation rehearsal requires Anvil chain 31337')
    # This method does not exist on public RPCs. Refuse to broadcast without a local node proof.
    node=rpc.call('anvil_nodeInfo',[])
    m=read_manifest(84532);set_environment(m,rpc)
    safe=m['safe'];asset=m['asset'];vault_a=m['vaultA'];vault_b=m['vaultB']
    cash=balance(rpc,asset,safe);shares_a=balance(rpc,vault_a,safe);shares_b=balance(rpc,vault_b,safe)
    nonce=int(rpc.call('eth_call',[{'to':safe,'data':keccak('nonce()')[:10]},'latest']),16)
    os.environ['EVIDENCE_OPERATION']='seedVaults'
    _,receipts=script('EvidenceActions',rpc,m['contracts']['policyLibrary']['address'])
    receipt=receipts[-1]
    assert cash-balance(rpc,asset,safe)==7000*10**6,'wrong observed cash delta'
    assert balance(rpc,vault_a,safe)>shares_a,'no Aave shares minted'
    assert balance(rpc,vault_b,safe)>shares_b,'no demo-vault shares minted'
    for vault in [vault_a,vault_b]:
        allowance=run(['cast','call',asset,'allowance(address,address)(uint256)',safe,vault,'--rpc-url',rpc.url]).split()[0]
        assert int(allowance)==0,'residual allowance'
    after_nonce=int(rpc.call('eth_call',[{'to':safe,'data':keccak('nonce()')[:10]},'latest']),16)
    assert after_nonce==nonce+1,'owner batch must be exactly one Safe transaction'
    abi=event_abi('Deposit');topic=keccak(signature(abi))
    for vault,amount in [(vault_a,3000),(vault_b,4000)]:
        logs=[l for l in receipt['logs'] if l['address'].lower()==vault.lower() and l['topics'][0].lower()==topic]
        assert len(logs)==1,'missing deposit event'
        fields=decode_event(logs[0],abi)
        assert fields['assets']==str(amount*10**6) and fields['owner'].lower()==safe.lower()
    report=['PASS: owner seed allocation rehearsed on a Base Sepolia fork, chain 31337.',
            'Anvil block before allocation: '+str(node.get('currentBlockNumber','unknown')),
            'Fork-only transaction (NOT Base Sepolia): '+receipt['transactionHash'],
            'PASS: exactly one threshold-two Safe batch; nonce increments once.',
            'PASS: cash debit exactly 7,000 USDC; both vault share balances increase.',
            'PASS: Aave Deposit 3,000 USDC and demo-vault Deposit 4,000 USDC, owner=Safe.',
            'PASS: both token allowances return to zero.']
    (ROOT/'deployments/owner-allocation-rehearsal.txt').write_text('\n'.join(report)+'\n')
    print('\n'.join(report))


if __name__=='__main__':main()
