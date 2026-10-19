"""Real local RPC transactions. No external RPC or private keys are accepted."""
import json, pathlib, subprocess, urllib.request, urllib.error, time
ROOT=pathlib.Path(__file__).resolve().parents[1]
MULTICALL3='0xcA11bde05977b3631167028862bE2a173976CA11'
MULTICALL3_CODE='0x6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b60408051808201909152600081526060602082015281526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a0000000000000000000000000000000000000000000000000000000006000526020600452601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c0033'
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
  self.rpc('anvil_setCode',MULTICALL3,MULTICALL3_CODE)
  accounts=self.rpc('eth_accounts');self.owner,self.agent,self.guardian,self.vendor,self.attacker,self.approver=accounts[:6]
  self.agent_pk='0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d'
  self.guardian_pk='0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a'
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
  elif id=='A13':
   before=self.balance(self.naive)
   self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount))
   naive=self.naive_call(self.token,calldata('transfer(address,uint256)',target,amount))
   naive_outflow=before-self.balance(self.naive)
   man_path=self.out/'manifest-A13.json'
   man_path.write_text(json.dumps({"profile":"FULL","chainId":31337,"rpcUrl":self.url,"safe":self.safe,"module":self.module,"asset":self.token,"vaultA":self.a,"vaultB":self.b,"sanctionsList":self.sanctions,"deploymentBlock":0,"explorerBaseUrl":"http://127.0.0.1","vaultImplementations":{}},indent=2))
   ob_path=self.out/'obligations-A13.json'
   ob_path.write_text(json.dumps({"bills":[{"obligationIdHash":self.bill,"payeeIdHash":self.payee,"assets":str(amount),"dueAt":self.due,"recipient":self.vendor,"proof":[]}]},indent=2))
   journal_path=self.out/'journal-A13.jsonl'
   if journal_path.exists():journal_path.unlink()
   runner=ROOT/'src/fusion-core/target/release/agent-runner'
   p1=subprocess.run([str(runner),'--manifest',str(man_path),'--obligations',str(ob_path),'--private-key',self.agent_pk,'--journal',str(journal_path),'--crash-after-broadcast','--evidence-id','A13','--evidence-out',str(self.out/'agent-A13.json')],capture_output=True,text=True)
   assert p1.returncode in [137,-9],f"Expected crash exit code 137 or -9, got {p1.returncode}: stdout={p1.stdout} stderr={p1.stderr}"
   lines=journal_path.read_text().strip().splitlines()
   assert len(lines)>=1
   entry1=json.loads(lines[0])
   assert entry1['status']=='UNKNOWN'
   tx_hash=entry1.get('rawTxHash') or entry1['raw_tx_hash']
   p2=subprocess.run([str(runner),'--manifest',str(man_path),'--obligations',str(ob_path),'--private-key',self.agent_pk,'--journal',str(journal_path),'--evidence-id','A13','--evidence-out',str(self.out/'agent-A13.json')],capture_output=True,text=True)
   assert p2.returncode==0,f"Run 2 failed ({p2.returncode}): stdout={p2.stdout} stderr={p2.stderr}"
   lines2=journal_path.read_text().strip().splitlines()
   assert len(lines2)>=2
   entry2=json.loads(lines2[-1])
   assert entry2['status']=='RECONCILED'
   state=self.words(self.module,'getState()')
   assert state[8]==1,f"Expected nonce 1, got {state[8]}"
   receipt=self.rpc('eth_getTransactionReceipt',tx_hash)
   attack=dict(txHash=tx_hash,status=int(receipt['status'],16),blockNumber=int(receipt['blockNumber'],16),to=receipt['to'],returnValue='',gasUsed=int(receipt['gasUsed'],16))
   return dict(attack=attack,extra=extra,stopped=True,stopping_layer="durable_journal",revert_selector=None,naive=dict(receipt=naive,outcome='Duplicate broadcast after restart without durable journal (double spend)',treasury_outflow_raw=naive_outflow,realized_attacker_receipt_raw=0))
  elif id=='A17':
   proxy=self.deploy('ArenaVaultProxy','address',(self.a,));original_a=self.a;self.a=proxy
   self.module=self.deploy('KavachiModule','address,address,address,address,address',(self.safe,self.token,self.a,self.b,self.sanctions));self.send(self.safe,'enableModule(address)',self.module,sender=self.safe);self.configure();self.commit();self.session()
   slot='0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc'
   self.rpc('anvil_setStorageAt',proxy,slot,'0x'+original_a.lower().removeprefix('0x').rjust(64,'0'))
   replacement=self.deploy('KavachiDemoVault','address,string,string',(self.token,'Changed implementation','CI'))
   extra.append(self.send(proxy,'upgrade(address)',replacement))
   self.rpc('anvil_setStorageAt',proxy,slot,'0x'+replacement.lower().removeprefix('0x').rjust(64,'0'))
   man_path=self.out/'manifest-A17.json'
   man_path.write_text(json.dumps({"profile":"FULL","chainId":31337,"rpcUrl":self.url,"safe":self.safe,"module":self.module,"asset":self.token,"vaultA":proxy,"vaultB":self.b,"sanctionsList":self.sanctions,"deploymentBlock":0,"explorerBaseUrl":"http://127.0.0.1","vaultImplementations":{proxy:original_a}},indent=2))
   passport_dir=self.out/'passports-A17'
   evidence_out=self.out/'agent-A17.json'
   runner=ROOT/'src/fusion-core/target/release/agent-runner'
   res=subprocess.run([str(runner),'--manifest',str(man_path),'--guardian-key',self.guardian_pk,'--passport-dir',str(passport_dir),'--evidence-out',str(evidence_out),'--evidence-id','A17'],capture_output=True,text=True)
   assert res.returncode==0,f"Agent runner failed ({res.returncode}): stdout={res.stdout} stderr={res.stderr}"
   assert bool(self.words(self.module,'getState()')[1]),"Module should be paused by guardian"
   before=self.balance(self.naive)
   amount=1_000_000
   self.naive_call(self.token,calldata('approve(address,uint256)',proxy,amount))
   naive=self.naive_call(proxy,calldata('deposit(uint256,address)',amount,self.naive))
   x=self.action(0,amount)
   attack=self.execute(x,expected=0)
   trace=attack['returnValue'].removeprefix('0x')
   return dict(attack=attack,extra=extra,stopped=True,stopping_layer="off-chain impl pin",revert_selector='0x'+trace[:8] if trace else None,naive=dict(receipt=naive,outcome='Agent deposited without the attacked policy/risk check (exposure, not realized loss)',treasury_outflow_raw=before-self.balance(self.naive),realized_attacker_receipt_raw=0))
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
