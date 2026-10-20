#!/usr/bin/env python3
"""Keyless K44 capture. No signing or on-chain policy writes. Native math is run separately."""
import argparse, json, urllib.request, os
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
RPC = os.environ.get('BASE_SEPOLIA_RPC', 'https://sepolia.base.org')
POOL='0x8bAB6d1b75f19e9eD9fCe8b9BD338844fF79aE27'
USDC='0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f'
FEED='0x4aDC67696bA383F43DD60A9e78F2C97Fbbfc7cb1'
def rpc(method, params, url=RPC):
    req=urllib.request.Request(url,data=json.dumps(dict(jsonrpc='2.0',id=1,method=method,params=params)).encode(),headers={'Content-Type':'application/json','User-Agent':'curl/8.0'})
    with urllib.request.urlopen(req,timeout=20) as r: value=json.load(r)
    if 'error' in value: raise RuntimeError(value['error'])
    return value['result']
def words(to,data,tag,url=RPC):
    raw=rpc('eth_call',[{'to':to,'data':data},tag],url)
    assert len(raw[2:])%64==0 and len(raw)>2
    return [int(raw[i:i+64],16) for i in range(2,len(raw),64)]
def addr(n): return '0x'+format(n,'040x')
def word(a): return a[2:].lower().zfill(64)
def capture(size):
    assert int(rpc('eth_chainId',[]),16)==84532
    head=rpc('eth_getBlockByNumber',['latest',False]); tag=head['number']; height=int(tag,16); now=int(head['timestamp'],16)
    manifest=json.loads((ROOT/'deployments/84532.json').read_text())
    vault=manifest['contracts']['vaultA']['address']; session=manifest['roles']['session']
    reserve=words(POOL,'0x35ea6a75'+word(USDC),tag)
    available=words(USDC,'0x70a08231'+word(addr(reserve[8])),tag)[0]
    debt=lambda address: 0 if address==0 else words(addr(address),'0x18160ddd',tag)[0]
    measure=dict(availableRaw=str(available),variableDebtRaw=str(debt(reserve[10])),stableDebtRaw=str(debt(reserve[9])),liquidityRateRay=str(reserve[2]),liquidityIndex=str(words(POOL,'0xd15e0053'+word(USDC),tag)[0]))
    history_errors=[]
    for url in (RPC,'https://base-sepolia-rpc.publicnode.com'):
        try:
            assert int(rpc('eth_chainId',[],url),16)==84532
            assert rpc('eth_getBlockByNumber',[tag,False],url)['hash']==head['hash']
            past=rpc('eth_getBlockByNumber',[hex(height-43200),False],url); timestamp=int(past['timestamp'],16)
            assert 23*3600<=now-timestamp<=25*3600
            past_index=words(POOL,'0xd15e0053'+word(USDC),past['number'],url)[0]
            measure.update(pastIndex=str(past_index),pastTimestamp=timestamp,pastBlock=height-43200)
            history_source=url; break
        except Exception as e: history_errors.append(str(e)); history_source=None
    inputs=dict(blockNumber=height,vault=vault,reserve=measure)
    try:
        r=words(FEED,'0xfeaf968c',tag); decimals=words(FEED,'0x313ce567',tag)[0]
        inputs['oracle']=dict(feed=FEED,decimals=decimals,answer=str(r[1]) if r[1]<2**255 else str(r[1]-2**256),roundId=str(r[0]),answeredInRound=str(r[4]),updatedAt=r[3])
    except Exception as e: inputs['oracleError']=str(e)
    receipts=[]
    for entry in json.loads((ROOT/'evidence/tx_index.json').read_text()):
        receipt=rpc('eth_getTransactionReceipt',[entry['txHash']])
        if receipt and receipt['from'].lower()==session.lower() and int(receipt['blockNumber'],16)<=height: receipts.append(receipt)
    receipts.sort(key=lambda r:int(r['blockNumber'],16),reverse=True)
    costs=[]; samples=[]
    for r in receipts[:20]:
        header=rpc('eth_getBlockByNumber',[r['blockNumber'],False])
        if header['hash']!=r['blockHash'] or not now-30*86400<=int(header['timestamp'],16)<=now:continue
        cost=int(r['gasUsed'],16)*int(r['effectiveGasPrice'],16)+sum(int(r.get(k) or '0x0',16) for k in ('l1Fee','operatorFee'))
        if cost>0:costs.append(cost);samples.append(dict(txHash=r['transactionHash'],costWei=str(cost)))
    costs.sort()
    if costs: inputs['medianGasWei']=str(costs[len(costs)//2] if len(costs)%2 else (costs[len(costs)//2-1]+costs[len(costs)//2]+1)//2)
    shares=words(vault,'0xef8b30f7'+format(size,'064x'),tag)[0]
    assets=words(vault,'0x07a2d13a'+format(shares,'064x'),tag)[0]
    inputs.update(quotedSizeRaw=str(size),depositSlippageRaw=str(max(size-assets,0)))
    assert rpc('eth_getBlockByNumber',[tag,False])['hash']==head['hash']
    out=dict(rpc=RPC,blockHash=head['hash'],timestamp=now,inputs=inputs,historySource=history_source,historyErrors=history_errors,feeSamples=samples,
        feedSource='https://reference-data-directory.vercel.app/feeds-ethereum-testnet-sepolia-base-1.json')
    path=ROOT/'evidence/app/agent/K44_live_inputs.json'
    return out
if __name__=='__main__':
    ap=argparse.ArgumentParser(); ap.add_argument('--size-usdc',type=int,default=10); ap.add_argument('--output')
    args=ap.parse_args(); out=capture(args.size_usdc*1_000_000)
    path=Path(args.output) if args.output else ROOT/'evidence/app/agent/K44_live_inputs.json'
    path.write_text(json.dumps(out,indent=2)+'\n'); print(path); print(json.dumps(out['inputs'],indent=2))
