#!/usr/bin/env python3
"""Prepare a reviewable owner setup. Never signs or broadcasts to Base Sepolia.

--simulate uses an ephemeral local Anvil fork and impersonated public accounts.
Production owner keys are never read by this program.
"""
import argparse,json,pathlib,secrets,subprocess,time,urllib.request,socket
ROOT=pathlib.Path(__file__).resolve().parents[2]
RPC='https://base-sepolia-rpc.publicnode.com'
def rpc(url,method,params):
    req=urllib.request.Request(url,json.dumps({'jsonrpc':'2.0','id':1,'method':method,'params':params}).encode(),{'Content-Type':'application/json','User-Agent':'fusion-wallet-android/1.0'})
    with urllib.request.urlopen(req,timeout=25) as f: value=json.load(f)
    if 'error'in value: raise RuntimeError(value['error'])
    return value['result']
def cast(*args): return subprocess.check_output(['cast',*map(str,args)],text=True).strip()
def calldata(signature,*args): return cast('calldata',signature,*args)
def call(url,to,data): return rpc(url,'eth_call',[{'to':to,'data':data},'latest'])
def words(value): return [value[i:i+64] for i in range(2,len(value),64)]
def entry(to,data): return '00'+to[2:].lower()+'0'*64+format((len(data)-2)//2,'064x')+data[2:]
def prepare():
    m=json.loads((ROOT/'deployments/84532.json').read_text())
    assert rpc(RPC,'eth_chainId',[])=='0x14a34'
    block=rpc(RPC,'eth_getBlockByNumber',['latest',False]); st=words(call(RPC,m['module'],calldata('getState()')))
    assert int(st[6],16)==0,'Refuse to replace an unpaid commitment'
    queue=words(call(RPC,m['module'],calldata('getQueue()')))
    assert int(queue[0],16)==0,'Refuse to replace an active queue'
    key='0x'+st[13][-40:]; assert key.lower()==m['roles']['session'].lower()
    expiry=int(st[14],16); remaining=int(st[15],16)-int(st[16],16); epoch=int(st[2],16)
    assert int(st[0],16)==1 and int(st[1],16)==0 and expiry>int(block['timestamp'],16)+600 and remaining>=8_000_000
    payee=cast('keccak','fusion.k2.payee.acme')
    payee_state=words(call(RPC,m['module'],calldata('getPayee(bytes32)',payee)))
    assert '0x'+payee_state[0][-40:]==m['roles']['owner'].lower(),'Unexpected payee recipient'
    assert int(payee_state[3],16)==0 and int(payee_state[2],16)<=int(block['timestamp'],16),'Payee must be ACTIVE'
    policy=words(call(RPC,m['module'],calldata('getPolicy()')))
    assert int(policy[1],16)==2 and int(policy[3],16)>=8_000_000,'Bill must be autonomous instant-tier'
    cash=int(call(RPC,m['asset'],calldata('balanceOf(address)',m['safe'])),16)
    assert cash>=8_000_000+int(policy[2],16),'Cash must cover bill and operating buffer'
    bill=cast('keccak',f'fusion.k17.app.bill.{int(block["timestamp"],16)}.{secrets.token_hex(8)}'); due=int(block['timestamp'],16)+3600
    inner=cast('keccak',cast('abi-encode','f(bytes32,bytes32,uint256,uint64)',bill,payee,8_000_000,due))
    root=cast('keccak',inner)
    calls=[calldata('commitObligations(bytes32,uint256,uint256)',root,8_000_000,1),
           calldata('startSession(address,uint64,uint256)',key,expiry,remaining),calldata('resume(uint256)',epoch+1)]
    batch='0x'+''.join(entry(m['module'],data) for data in calls)
    multisend=m['contracts']['multiSendCallOnly']['address']
    proposal={'source':'PREPARED_ONLY','network':'Base Sepolia; no production broadcast',
      'before':{'block':int(block['number'],16),'hash':block['hash'],'policyEpoch':epoch,'reservedRaw':str(int(st[6],16)),
                'obligationsRoot':'0x'+st[5],'sessionKey':key,'sessionExpiresAt':expiry,'sessionRemainingRaw':str(remaining)},
      'bill':{'root':root,'bills':[{'obligationIdHash':bill,'payeeIdHash':payee,'assets':'8000000','dueAt':str(due),
                'recipient':m['roles']['owner'],'leaf':root,'proof':[]}]},
      'preflight':{'payeeStatus':'ACTIVE','cashRaw':str(cash),'microMaxRaw':str(int(policy[3],16)),
                   'permissionTier':'AUTONOMOUS','uniqueObligation':True},
      'safe':m['safe'],'module':m['module'],'to':multisend,'operation':1,'value':'0',
      'data':calldata('multiSend(bytes)',batch),'innerCalls':calls,
      'expectedAfter':{'policyEpoch':epoch+2,'paused':False,'reservedRaw':'8000000','obligationCount':'1',
                       'sessionKey':key,'sessionExpiresAt':expiry,'sessionLimitRaw':str(remaining)},
      'effect':'Replace a fully paid commitment with one new 8 USDC test bill to existing Acme recipient. Re-arm same session with remaining allowance and original expiry. Only app model choice A may cause payment.'}
    return m,proposal

def simulate(m,p):
    with socket.socket() as s: s.bind(('127.0.0.1',0)); port=s.getsockname()[1]
    url='http://127.0.0.1:'+str(port)
    node=subprocess.Popen(['anvil','--silent','--host','127.0.0.1','--port',str(port),'--chain-id','84532','--fork-url',RPC,'--fork-block-number',str(p['before']['block'])],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    def local(method,params): return rpc(url,method,params)
    def send(sender,to,data):
        local('anvil_impersonateAccount',[sender]); local('anvil_setBalance',[sender,hex(10**20)])
        h=local('eth_sendTransaction',[{'from':sender,'to':to,'data':data,'gas':'0x2dc6c0'}])
        receipt=None
        for _ in range(200):
            receipt=local('eth_getTransactionReceipt',[h])
            if receipt is not None: break
            time.sleep(.1)
        assert receipt is not None and receipt['status']=='0x1',receipt
        return receipt
    try:
        for _ in range(200):
            try: local('eth_chainId',[]); break
            except Exception: time.sleep(.1)
        else: raise RuntimeError('Local fork did not start')
        nonce=int(call(url,m['safe'],calldata('nonce()')),16)
        zero='0x'+'00'*20
        digest=call(url,m['safe'],calldata('getTransactionHash(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,uint256)',p['to'],0,p['data'],1,0,0,0,zero,zero,nonce))
        owners=sorted(m['owners'],key=str.lower)
        for owner in owners: send(owner,m['safe'],calldata('approveHash(bytes32)',digest))
        sigs='0x'+''.join(o[2:].lower().zfill(64)+'0'*64+'01' for o in owners)
        tx=calldata('execTransaction(address,uint256,bytes,uint8,uint256,uint256,uint256,address,address,bytes)',p['to'],0,p['data'],1,0,0,0,zero,zero,sigs)
        receipt=send(owners[0],m['safe'],tx)
        st=words(call(url,m['module'],calldata('getState()')))
        assert int(st[1],16)==0 and int(st[2],16)==p['expectedAfter']['policyEpoch']
        assert '0x'+st[5]==p['bill']['root'] and int(st[6],16)==8_000_000 and int(st[7],16)==1
        assert int(st[14],16)==p['expectedAfter']['sessionExpiresAt'] and int(st[15],16)==int(p['expectedAfter']['sessionLimitRaw'])
        p['localForkSimulation']={'status':'PASS','productionBroadcast':False,'safeTxStatus':1,'gasUsed':int(receipt['gasUsed'],16),
            'ownerAuthorization':'Public account impersonation + approved-hash signatures on local Anvil only; no production keys read.',
            'verified':'Commitment, reserve, count, epoch, pause state, unchanged expiry, preserved remaining allowance.'}
    finally:
        node.terminate(); node.wait(timeout=10)
if __name__=='__main__':
    ap=argparse.ArgumentParser(); ap.add_argument('--simulate',action='store_true'); a=ap.parse_args()
    m,p=prepare()
    if a.simulate: simulate(m,p)
    out=ROOT/'evidence/app/agent/m3_setup_preview.json'; out.write_text(json.dumps(p,indent=2)+'\n')
    print('Prepared owner setup:',out)
    print('Local fork simulation:',p.get('localForkSimulation',{}).get('status','NOT_RUN'))
    print('No production signing or broadcast performed.')
