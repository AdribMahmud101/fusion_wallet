"""Real local RPC transactions. No external RPC or private keys are accepted."""
import json, pathlib, subprocess, urllib.request, urllib.error, time
ROOT=pathlib.Path(__file__).resolve().parents[1]
ZERO='0x'+'00'*20; B0='0x'+'00'*32
ACTION='(uint8,address,address,uint256,uint256,uint256,bytes32,bytes32,address,uint64,uint256,uint256,uint64,bytes32)'
POLICY='(uint8,uint256,uint256,uint256,uint256,uint256,uint32,uint16,address,(address,uint256)[],uint32,uint32)'
def cast(*args):return subprocess.check_output(['cast',*map(str,args)],text=True).strip()
def calldata(sig,*args):return cast('calldata',sig,*args)
def tup(values):return '('+','.join(map(str,values))+')'
class Chain:
 def __init__(self,url,out):self.url=url;self.out=out;self.setup=[];self.n=0
 def rpc(self,method,*params):
  self.n+=1
  req=urllib.request.Request(self.url,data=json.dumps(dict(jsonrpc='2.0',id=self.n,method=method,params=params)).encode(),headers={'Content-Type':'application/json'})
  with urllib.request.urlopen(req,timeout=60) as f:r=json.load(f)
  if 'error' in r:raise RuntimeError(r['error'])
  return r['result']
 def tx(self,to,data,sender=None,expected=None):
  tx={'from':sender or self.owner,'data':data,'gas':hex(25_000_000)}
  if to:tx['to']=to
  h=self.rpc('eth_sendTransaction',tx)
  r=None
  for _ in range(200):
   r=self.rpc('eth_getTransactionReceipt',h)
   if r is not None:break
   time.sleep(0.02)
  if r is None:raise TimeoutError(f'Receipt timeout for tx {h}')
  status=int(r['status'],16)
  return_value=''
  try:
   trace=self.rpc('debug_traceTransaction',h,{'disableMemory':True,'disableStorage':True,'disableStack':True})
   return_value=trace.get('returnValue','')
  except Exception:
   pass
  if status==0 and not return_value:
   call_tx={'from':tx['from'],'data':data}
   if to:call_tx['to']=to
   blk=hex(int(r['blockNumber'],16)-1) if int(r['blockNumber'],16)>0 else 'latest'
   try:
    req=urllib.request.Request(self.url,data=json.dumps(dict(jsonrpc='2.0',id=999999,method='eth_call',params=[call_tx,blk])).encode(),headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=10) as f:cr=json.load(f)
    if 'error' in cr and isinstance(cr['error'],dict) and 'data' in cr['error']:
     return_value=cr['error']['data']
   except Exception:
    pass
  result=dict(txHash=h,status=status,blockNumber=int(r['blockNumber'],16),to=to,returnValue=return_value,gasUsed=int(r['gasUsed'],16))
  if expected is not None and result['status']!=expected:raise AssertionError((result,return_value))
  self.setup.append(result)
  return result,r.get('contractAddress')
 def deploy(self,name,types='',args=()):
  matches=list((ROOT/'src/contracts/out').glob(f'*/{name}.json'))
  if not matches:raise FileNotFoundError(f'Artifact for {name} not found')
  artifact=json.loads(matches[0].read_text())
  code=artifact['bytecode']['object']
  link_refs=artifact['bytecode'].get('linkReferences',{})
  for file,libs in link_refs.items():
   for lib,locs in libs.items():
    lib_addr=getattr(self,'lib_'+lib,None)
    if not lib_addr:
     lib_addr=self.deploy(lib)
     setattr(self,'lib_'+lib,lib_addr)
    clean_addr=lib_addr.lower().removeprefix('0x')
    placeholder=code[locs[0]['start']*2+2:locs[0]['start']*2+2+locs[0]['length']*2]
    code=code.replace(placeholder,clean_addr)
  if types:code+=cast('abi-encode','constructor('+types+')',*args)[2:]
  return self.tx(None,code,expected=1)[1]
 def send(self,to,sig,*args,sender=None,expected=1):return self.tx(to,calldata(sig,*args),sender,expected)[0]
 def call(self,to,sig,*args):return self.rpc('eth_call',{'to':to,'data':calldata(sig,*args)},'latest')
 def words(self,to,sig):
  raw=self.call(to,sig)[2:];return [int(raw[i:i+64],16) for i in range(0,len(raw),64)]
 def balance(self,who):return int(self.call(self.token,'balanceOf(address)',who),16)
 def time(self):return int(self.rpc('eth_getBlockByNumber','latest',False)['timestamp'],16)
 def advance(self,seconds):self.rpc('evm_increaseTime',seconds);self.rpc('evm_mine')
 def policy(self,daily=4_000_000_000):
  vaults=sorted([self.a,self.b],key=lambda x:int(x,16))
  return tup([2,500_000_000,10_000_000,100_000_000,2_500_000_000,daily,180,50,self.guardian,'['+','.join(tup([v,6_000_000_000]) for v in vaults)+']',3600,259200])
 def configure(self,daily=4_000_000_000):self.send(self.module,'configurePolicy('+POLICY+')',self.policy(daily),sender=self.safe)
 def commit(self,amount=10_000_000,payee=None):
  leaf=cast('keccak',cast('keccak',cast('abi-encode','f(bytes32,bytes32,uint256,uint64)',self.bill,payee or self.payee,amount,self.due)))
  self.send(self.module,'commitObligations(bytes32,uint256,uint256)',leaf,amount,1,sender=self.safe)
 def session(self):
  self.send(self.module,'startSession(address,uint64,uint256)',self.agent,self.time()+86400,8_000_000_000,sender=self.safe)
  self.send(self.module,'resume(uint256)',self.words(self.module,'getState()')[2],sender=self.safe)
 def action(self,kind=0,amount=1_000_000):
  s=self.words(self.module,'getState()')
  return [kind,ZERO,self.a if kind==0 else ZERO,amount,amount if kind==0 else 0,0,self.bill if kind==3 else B0,self.payee if kind==3 else B0,self.vendor if kind==3 else ZERO,self.due if kind==3 else 0,s[8],s[2],self.time()+120,self.decision]
 def execute(self,x,expected=None):return self.send(self.module,'execute('+ACTION+',bytes32[])',tup(x),'[]',sender=self.agent,expected=expected)
 def initialize(self):
  assert int(self.rpc('eth_chainId'),16)==31337
  accounts=self.rpc('eth_accounts');self.owner,self.agent,self.guardian,self.vendor,self.attacker,self.approver=accounts[:6]
  self.token=self.deploy('MockUSDC');self.a=self.deploy('KavachiDemoVault','address,string,string',(self.token,'Arena A','AA'));self.b=self.deploy('KavachiDemoVault','address,string,string',(self.token,'Arena B','AB'))
  singleton=self.deploy('Safe');self.safe=self.deploy('SafeProxy','address',(singleton,))
  self.send(self.safe,'setup(address[],uint256,address,bytes,address,address,uint256,address)',f'[{self.owner},{self.approver}]',2,ZERO,'0x',ZERO,ZERO,0,ZERO)
  self.sanctions=self.deploy('KavachiTestSanctionsList','address',(self.safe,))
  self.module=self.deploy('KavachiModule','address,address,address,address,address',(self.safe,self.token,self.a,self.b,self.sanctions))
  # Privileged fixture setup only. Attack calls always use the session key.
  self.rpc('anvil_impersonateAccount',self.safe);self.rpc('anvil_setBalance',self.safe,hex(10**20))
  self.send(self.safe,'enableModule(address)',self.module,sender=self.safe)
  self.naive=self.deploy('NaiveAgentWallet','address',(self.agent,))
  for wallet in [self.safe,self.naive]:self.send(self.token,'mint(address,uint256)',wallet,10_000_000_000)
  self.configure();self.payee=cast('keccak','0x76656e646f72');self.bill=cast('keccak','0x62696c6c');self.decision=cast('keccak','0x6172656e61')
  self.send(self.module,'addPayee(bytes32,address,bytes32)',self.payee,self.vendor,B0,sender=self.safe)
  self.advance(3600);self.due=self.time()+86400;self.commit();self.session()
 def naive_call(self,to,data):return self.send(self.naive,'execute(address,bytes)',to,data,sender=self.agent)
 def attack(self,id):
  extra=[]; amount=10_000_000; mode='transfer';target=self.vendor
  if id in ['A2','A4']:
   x=self.action(3,amount);x[8]=self.attacker;target=self.attacker
  elif id=='A3':
   fresh=cast('keccak','0x6e6577');self.send(self.module,'addPayee(bytes32,address,bytes32)',fresh,self.attacker,B0,sender=self.safe);self.commit(payee=fresh);self.session();x=self.action(3,amount);x[7]=fresh;x[8]=self.attacker;target=self.attacker
  elif id=='A5':
   self.commit(75_000_000);self.session();amount=750_000_000;x=self.action(3,amount);self.send(self.module,'approveAction('+ACTION+',uint64)',tup(x),self.time()+3600,sender=self.safe)
  elif id=='A6':
   extra.append(self.execute(self.action(3,amount),1));self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount));x=self.action(3,amount)
  elif id=='A7':
   for w in [self.safe,self.naive]:
    if w==self.safe:self.send(self.token,'transfer(address,uint256)',self.owner,9_490_000_000,sender=w)
    else:self.naive_call(self.token,calldata('transfer(address,uint256)',self.owner,9_490_000_000))
   amount=1_000_000;x=self.action(0,amount);mode='deposit'
  elif id=='A8':
   x1=self.action(0,2_500_000_000);self.send(self.module,'approveAction('+ACTION+',uint64)',tup(x1),self.time()+3600,sender=self.safe);extra.append(self.execute(x1,1))
   x2=self.action(0,1_500_000_000);self.send(self.module,'approveAction('+ACTION+',uint64)',tup(x2),self.time()+3600,sender=self.safe);extra.append(self.execute(x2,1))
   self.naive_call(self.token,calldata('transfer(address,uint256)',target,4_000_000_000));amount=1_000_000;x=self.action(0,amount);mode='deposit'
  elif id in ['A9','A22','A24']:
   amount=11_000_000;x=self.action(0,amount);mode='deposit'
   if id!='A9':
    x[12]=self.time()+1000;extra.append(self.send(self.module,'queueAction('+ACTION+',bytes32[])',tup(x),'[]',sender=self.agent))
    if id=='A22':self.advance(1001)
  elif id in ['A10','A20']:
   extra.append(self.send(self.module,'pause(bytes32)',self.decision,sender=self.guardian));x=self.action(3,amount)
  elif id=='A11':
   self.send(self.sanctions,'setSanctioned(address,bool)',self.vendor,'true',sender=self.safe);x=self.action(3,amount)
  elif id=='A14':
   self.send(self.module,'ownerPayObligation(bytes32,bytes32,uint256,uint64,bytes32[])',self.bill,self.payee,amount,self.due,'[]',sender=self.safe);self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount));x=self.action(3,amount)
  elif id=='A15':
   attack=self.send(self.module,'configurePolicy('+POLICY+')',self.policy(),sender=self.agent,expected=0)
   extra.append(self.send(self.module,'resume(uint256)',self.words(self.module,'getState()')[2],sender=self.agent,expected=0))
   before=self.balance(self.naive);naive=self.naive_call(self.token,calldata('approve(address,uint256)',self.attacker,2**256-1))
   return dict(attack=attack,extra=extra,stopped=attack['status']==0,revert_selector='0x'+attack['returnValue'].removeprefix('0x')[:8],naive=dict(receipt=naive,treasury_outflow_raw=before-self.balance(self.naive),outcome='Unlimited token allowance granted by agent; semantic authority comparator',allowance_raw=str(int(self.call(self.token,'allowance(address,address)',self.naive,self.attacker),16))))
  elif id=='A17':
   proxy=self.deploy('ArenaVaultProxy','address',(self.a,));self.a=proxy
   self.module=self.deploy('KavachiModule','address,address,address,address,address',(self.safe,self.token,self.a,self.b,self.sanctions));self.send(self.safe,'enableModule(address)',self.module,sender=self.safe);self.configure();self.commit();self.session()
   replacement=self.deploy('KavachiDemoVault','address,string,string',(self.token,'Changed implementation','CI'));extra.append(self.send(proxy,'upgrade(address)',replacement));x=self.action(0,1_000_000);amount=1_000_000;mode='deposit'
  elif id=='A18':x=self.action(3,amount);x[11]-=1
  elif id=='A19':
   amount=1_000_000;x=self.action(0,amount);x[2]=self.attacker;mode='transfer';target=self.attacker
  elif id=='A21':
   x=self.action(3,amount);extra.append(self.execute(x,1));self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount))
  elif id=='A23':
   amount=2_500_000_001;x=self.action(0,amount);mode='deposit'
  else:raise ValueError(id)
  attack=self.execute(x)
  before=self.balance(self.naive)
  if mode=='deposit':
   self.naive_call(self.token,calldata('approve(address,uint256)',self.a,amount));naive=self.naive_call(self.a,calldata('deposit(uint256,address)',amount,self.naive));outcome='Agent deposited without the attacked policy/risk check (exposure, not realized loss)'
  else:
   naive=self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount));outcome='Agent transferred the attacked payment without policy checks'
  trace=attack['returnValue'].removeprefix('0x')
  return dict(attack=attack,extra=extra,stopped=attack['status']==0,revert_selector='0x'+trace[:8] if (attack['status']==0 and trace) else None,naive=dict(receipt=naive,outcome=outcome,treasury_outflow_raw=before-self.balance(self.naive),realized_attacker_receipt_raw=amount if target==self.attacker and mode=='transfer' else 0))

def run(url,out,ids):
 c=Chain(url,out);c.initialize();base=c.rpc('evm_snapshot');saved=(c.a,c.module)
 initial=list(c.setup);results={}
 for id in ids:
  c.rpc('evm_revert',base);base=c.rpc('evm_snapshot');c.a,c.module=saved;c.setup=[]
  results[id]=c.attack(id);results[id]['transactions']=c.setup
  (out/f'anvil-{id}.json').write_text(json.dumps(results[id],indent=2)+'\n')
 (out/'anvil-setup.json').write_text(json.dumps(initial,indent=2)+'\n')
 return results
