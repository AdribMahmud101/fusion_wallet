#!/usr/bin/env python3
"""Public, keyless preparation and verification of a one-time owner strategy mandate.

The 2/2 Safe signs inert calldata to its owner EOA. It authorizes at most one
<=10 USDC allocation and one withdrawal, bound to session, policy, nonces and
expiry. It changes the strategy purpose, never the standard yield economics or
any on-chain safety limit. Only the Android planner can choose an APP action.
"""
import json, socket, subprocess, time
from prepare_demo import ROOT, RPC, rpc, cast, calldata, call, words
SIGNATURE='fusionAllocationMandate(uint256,address,address,address,bytes32,bytes32,uint256,uint256,uint64)'
SUCCESS='0x442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e'

def prepare():
    m=json.loads((ROOT/'deployments/84532.json').read_text())
    assert rpc(RPC,'eth_chainId',[])=='0x14a34'
    block=rpc(RPC,'eth_getBlockByNumber',['latest',False]); tag=block['number']
    def view(to,data): return rpc(RPC,'eth_call',[{'to':to,'data':data},tag])
    s=words(view(m['module'],calldata('getState()')))
    p=words(view(m['module'],calldata('getPolicy()')))
    assert int(s[0],16)==1 and int(s[1],16)==0 and int(s[6],16)==0
    assert int(words(view(m['module'],calldata('getQueue()')))[0],16)==0
    assert int(p[1],16)==2 and int(p[3],16)>=10_000_000
    assert '0x'+s[13][-40:]==m['roles']['session'].lower()
    assert int(s[15],16)-int(s[16],16)>=20_000_000
    now=int(block['timestamp'],16); expiry=min(now+3600,int(s[14],16))
    assert expiry>=now+1200
    cash=int(view(m['asset'],calldata('balanceOf(address)',m['safe'])),16)
    assert cash>=10_000_000+int(p[2],16)
    assert int(view(m['vaultA'],calldata('balanceOf(address)',m['safe'])),16)==0, 'Rehearsal starts with no Aave shares'
    assert rpc(RPC,'eth_getCode',[m['roles']['owner'],tag])=='0x'
    nonce=int(s[8],16)
    payload=calldata(SIGNATURE,84532,m['safe'],m['module'],m['vaultA'],'0x'+s[11],'0x'+s[3],10_000_000,nonce,expiry)
    proposal={'source':'PREPARED_ONLY','chainId':84532,'purpose':'One-time gas-paying allocation rehearsal, not a profitable yield strategy',
        'to':m['roles']['owner'],'safe':m['safe'],'module':m['module'],'vault':m['vaultA'],'value':'0','operation':0,
        'data':payload,'payloadHash':cast('keccak',payload),'assetsRaw':'10000000','depositNonce':nonce,'withdrawNonce':nonce+1,
        'sessionId':'0x'+s[11],'policyHash':'0x'+s[3],'expiresAt':expiry,'snapshotBlock':int(tag,16),'snapshotBlockHash':block['hash'],
        'expectedEffects':{'ownerTransaction':'Inert EOA calldata commitment; no assets or module state changed',
            'deposit':'Only a fresh Android MNN/AHEM-G A choice may deposit exactly 10 USDC',
            'withdraw':'Only a later fresh Android A choice may withdraw up to 10 USDC back to Safe cash',
            'safety':'Unchanged allowlist, reserves, exposure, tiers, daily/session limits, code pins, simulation and signer allowlist',
            'profitability':'Standard yield economics unchanged; this separately owner-signed intent authorizes gas expense'},
        'beforeState':'0x'+''.join(s)}
    return m,proposal

def simulate(m,p):
    with socket.socket() as s: s.bind(('127.0.0.1',0)); port=s.getsockname()[1]
    url=f'http://127.0.0.1:{port}'
    node=subprocess.Popen(['anvil','--silent','--host','127.0.0.1','--port',str(port),'--chain-id','84532','--fork-url',RPC,
                           '--fork-block-number',str(p['snapshotBlock'])],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    def local(method,params): return rpc(url,method,params)
    def send(sender,to,data):
        local('anvil_impersonateAccount',[sender]); local('anvil_setBalance',[sender,hex(10**20)])
        h=local('eth_sendTransaction',[{'from':sender,'to':to,'data':data,'gas':'0x2dc6c0'}])
        for _ in range(100):
            r=local('eth_getTransactionReceipt',[h])
            if r: break
            time.sleep(.1)
        assert r and r['status']=='0x1'
        return r
    try:
        for _ in range(200):
            try: local('eth_chainId',[]); break
            except Exception: time.sleep(.1)
        else: raise RuntimeError('Local fork failed to start')
        zero='0x'+'00'*20; nonce=int(call(url,m['safe'],calldata('nonce()')),16)
        digest=call(url,m['safe'],calldata('getTransactionHash(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,uint256)',p['to'],0,p['data'],0,0,0,0,zero,zero,nonce))
        owners=sorted(m['owners'],key=str.lower)
        for owner in owners: send(owner,m['safe'],calldata('approveHash(bytes32)',digest))
        sigs='0x'+''.join(o[2:].lower().zfill(64)+'0'*64+'01' for o in owners)
        tx=calldata('execTransaction(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,bytes)',p['to'],0,p['data'],0,0,0,0,zero,zero,sigs)
        r=send(owners[0],m['safe'],tx)
        assert any(log['address'].lower()==m['safe'].lower() and log['topics'][0]==SUCCESS for log in r['logs'])
        assert call(url,m['module'],calldata('getState()'))==p['beforeState']
        p['localForkSimulation']={'status':'PASS','productionBroadcast':False,'safeTxStatus':1,'gasUsed':int(r['gasUsed'],16),
            'verified':'2-of-2 Safe executes inert mandate; entire module state unchanged; no token transfer',
            'keys':'Public account impersonation on local Anvil only; no production keys read'}
    finally:
        node.terminate(); node.wait(timeout=10)

def verify(hash,m,snapshot=None):
    assert rpc(RPC,'eth_chainId',[])=='0x14a34'
    r=rpc(RPC,'eth_getTransactionReceipt',[hash]); tx=rpc(RPC,'eth_getTransactionByHash',[hash])
    assert r['transactionHash']==tx['hash']==hash and int(r['status'],16)==1 and int(tx['chainId'],16)==84532
    assert r['to'].lower()==tx['to'].lower()==m['safe'].lower()
    assert r['from'].lower()==tx['from'].lower()==m['roles']['owner'].lower()
    assert rpc(RPC,'eth_getBlockByNumber',[r['blockNumber'],False])['hash']==r['blockHash']==tx['blockHash']
    assert rpc(RPC,'eth_getCode',[m['roles']['owner'],r['blockNumber']])=='0x'
    assert sum(log['address'].lower()==m['safe'].lower() and log['topics'][0]==SUCCESS for log in r['logs'])==1
    def view(data): return rpc(RPC,'eth_call',[{'to':m['safe'],'data':data},r['blockNumber']])
    assert int(view(calldata('getThreshold()')),16)==2
    owners=words(view(calldata('getOwners()')))
    assert int(owners[1],16)==2 and {'0x'+w[-40:] for w in owners[2:]}=={v.lower() for v in m['owners']}
    assert tx['input'].startswith('0x6a761202')
    outer=words('0x'+tx['input'][10:]); assert '0x'+outer[0][-40:]==m['roles']['owner'].lower()
    assert int(outer[1],16)==0 and all(int(w,16)==0 for w in outer[3:9])
    offset=int(outer[2],16)//32; length=int(outer[offset],16)
    assert length==4+9*32
    payload='0x'+''.join(outer[offset+1:])[:length*2]
    assert payload.startswith(cast('sig',SIGNATURE))
    mandate=words('0x'+payload[10:]); assert len(mandate)==9 and int(mandate[0],16)==84532
    assert ['0x'+w[-40:] for w in mandate[1:4]]==[m[k].lower() for k in ('safe','module','vaultA')]
    amount=int(mandate[6],16); nonce=int(mandate[7],16); expiry=int(mandate[8],16)
    assert 0<amount<=10_000_000
    if snapshot is not None:
        st=snapshot['moduleState']; current=int(st['actionNonce'])
        assert int(r['blockNumber'],16)<=snapshot['blockNumber'] and snapshot['blockTimestamp']<expiry<=st['sessionExpiresAt']
        assert '0x'+mandate[4]==st['sessionId'] and '0x'+mandate[5]==st['policyHash'] and st['reservedRaw']=='0'
        assert current in (nonce,nonce+1) and amount<=int(snapshot['policy']['microMaxRaw'])
    return {'source':'OWNER','txHash':hash,'kind':'ALLOCATION_MANDATE','chainId':84532,'receipt':r,'transaction':tx,
        'assetsRaw':str(amount),'depositNonce':nonce,'withdrawNonce':nonce+1,'expiresAt':expiry,
        'sessionId':'0x'+mandate[4],'policyHash':'0x'+mandate[5],
        'purpose':'2-of-2 owner strategy authorization for one bounded allocation rehearsal; not an APP transfer'}
