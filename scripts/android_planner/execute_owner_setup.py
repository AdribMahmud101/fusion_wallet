#!/usr/bin/env python3
"""Execute only the reviewed 8 USDC owner setup after explicit user approval.

Default is read-only. --broadcast is never invoked automatically. Owner signatures
are OWNER-sourced, separate from APP agent evidence. Keep the canonical flock
across optional app execution via its inherited open-file-description lock.
"""
import argparse,fcntl,json,os,pathlib,re,subprocess
from prepare_demo import calldata,cast,rpc,RPC
from rpc_bridge import command
ROOT=pathlib.Path(__file__).resolve().parents[2]
AP=argparse.ArgumentParser(); AP.add_argument('--broadcast',action='store_true'); AP.add_argument('--run-app',action='store_true'); AP.add_argument('--local-host',action='store_true'); AP.add_argument('--serial',default='192.168.0.105:5555'); args=AP.parse_args()
p=json.loads((ROOT/'evidence/app/agent/m3_setup_preview.json').read_text()); m=json.loads((ROOT/'deployments/84532.json').read_text())
assert p.get('localForkSimulation',{}).get('status')=='PASS','A passing reviewable local-fork simulation is required'
bill=p['bill']['bills']; assert len(bill)==1
bill=bill[0]; assert bill['assets']=='8000000' and bill['payeeIdHash']==cast('keccak','fusion.k2.payee.acme') and bill['recipient'].lower()==m['roles']['owner'].lower()
root=cast('keccak',cast('keccak',cast('abi-encode','f(bytes32,bytes32,uint256,uint64)',bill['obligationIdHash'],bill['payeeIdHash'],8000000,bill['dueAt'])))
assert root==p['bill']['root'] and p['to'].lower()==m['contracts']['multiSendCallOnly']['address'].lower() and p['operation']==1
assert p['innerCalls']==[calldata('commitObligations(bytes32,uint256,uint256)',root,8000000,1),calldata('startSession(address,uint64,uint256)',p['before']['sessionKey'],p['before']['sessionExpiresAt'],p['before']['sessionRemainingRaw']),calldata('resume(uint256)',p['before']['policyEpoch']+1)]
if not args.broadcast:
 print('Read-only: prepared 8 USDC owner setup passed local-fork simulation. Explicit owner approval is required before --broadcast.')
 raise SystemExit(0)
def read_key(name):
 for line in pathlib.Path('/root/projects/fusion_wallet/.env.local').open():
  match=re.match(r'^\s*(?:export\s+)?'+name+r'\s*=\s*([\"\']?)(0x[0-9a-fA-F]{64}|[0-9a-fA-F]{64})\1\s*(?:#.*)?$',line)
  if match: return '0x'+match.group(2).removeprefix('0x')
 raise SystemExit('Required owner credential unavailable; no key value is printed.')
print('Waiting for shared session flock for reviewed OWNER setup ...',flush=True)
with open('/root/projects/fusion_wallet/.session.lock','a+') as locked:
 fcntl.flock(locked,fcntl.LOCK_EX)
 assert rpc(RPC,'eth_chainId',[])=='0x14a34'
 block=rpc(RPC,'eth_getBlockByNumber',['latest',False]); tip=1000000; fee=2*int(block['baseFeePerGas'],16)+tip
 assert fee<=10_000_000_000,'Owner fee cap exceeded'
 owner=read_key('OWNER_PRIVATE_KEY'); approver=read_key('APPROVER_PRIVATE_KEY')
 env=os.environ.copy(); env.update({
  'OWNER_PRIVATE_KEY':owner,'APPROVER_PRIVATE_KEY':approver,
  # The owner pays the outer gas, so no third production private key is loaded.
  'DEPLOYER_PRIVATE_KEY':owner,'OWNER_ADDRESS':m['roles']['owner'],'APPROVER_ADDRESS':m['roles']['approver'],
  'SESSION_ADDRESS':p['before']['sessionKey'],'MODULE_ADDRESS':m['module'],'SAFE_ADDRESS':m['safe'],
  'K17_MULTISEND':p['to'],'K17_EPOCH':str(p['before']['policyEpoch']),'K17_PREVIOUS_ROOT':p['before']['obligationsRoot'],
  'K17_EXPECTED_EXPIRY':str(p['before']['sessionExpiresAt']),'K17_REMAINING':p['before']['sessionRemainingRaw'],
  'K17_NEW_ROOT':root,'K17_PAYLOAD_HASH':cast('keccak',p['data']),
  'FOUNDRY_BROADCAST':str(ROOT/'scripts/android_planner/.forge/broadcast'),
 })
 cmd=['forge','script','scripts/android_planner/OwnerDemoSetup.s.sol:OwnerDemoSetup','--root',str(ROOT),'--contracts','scripts/android_planner',
  '--lib-paths','src/contracts/lib','--remappings','forge-std/=src/contracts/lib/forge-std/src/','--use','0.8.24','--via-ir','--evm-version','cancun',
  '--optimize','--optimizer-runs','200','--cache-path','scripts/android_planner/.forge/cache','--out','scripts/android_planner/.forge/out',
  '--rpc-url',RPC,'--broadcast','--slow','--with-gas-price',str(fee),'--priority-gas-price',str(tip)]
 result=subprocess.run(cmd,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 output=result.stdout.decode(errors='replace')
 for key in [owner,approver]:
  for form in [key,key[2:],key.upper(),key[2:].upper(),str(int(key,16))]: output=output.replace(form,'[REDACTED]')
 (ROOT/'evidence/app/agent/owner_setup_execution.txt').write_text(output)
 for name in ['OWNER_PRIVATE_KEY','APPROVER_PRIVATE_KEY','DEPLOYER_PRIVATE_KEY']: env.pop(name,None)
 owner=approver=None
 if result.returncode: raise SystemExit('Owner setup failed; see the redacted public execution log. App was not triggered.')
 broadcast=ROOT/'scripts/android_planner/.forge/broadcast/OwnerDemoSetup.s.sol/84532/run-latest.json'
 run=json.loads(broadcast.read_text()); receipts=run['receipts']; assert len(receipts)==1
 receipt=receipts[0]; assert int(receipt['status'],16)==1 and receipt['from'].lower()==m['roles']['owner'].lower() and receipt['to'].lower()==m['safe'].lower()
 record={'source':'OWNER','purpose':'Reviewed K17 demo commitment setup; not an APP agent transaction','receipt':receipt,'newRoot':root,'assets':'8000000','sessionExpiryExtended':False,'sessionAllowanceIncreased':False}
 (ROOT/'evidence/app/agent/owner_setup_receipt.json').write_text(json.dumps(record,indent=2)+'\n')
 public_bills=json.dumps(p['bill'],indent=2)+'\n'; (ROOT/'evidence/app/agent/m3_obligations.json').write_text(public_bills)
 command(args.local_host,args.serial,'run-as','app.fusion.wallet','mkdir','-p','files/agent',stdout=subprocess.DEVNULL)
 command(args.local_host,args.serial,'run-as','app.fusion.wallet','sh','-c','cat > files/agent/obligations.json',input=public_bills.encode())
 print('OWNER setup confirmed; public bill metadata provisioned. No APP success claimed yet.',flush=True)
 if args.run_app:
  child_env={k:v for k,v in os.environ.items() if not k.endswith('PRIVATE_KEY')}; child_env['K17_SESSION_LOCK_FD']=str(locked.fileno())
  call=['python3',str(ROOT/'scripts/android_planner/run_tick.py')]
  if args.local_host: call+=['--local-host']
  else: call+=['--serial',args.serial]
  subprocess.run(call,cwd=ROOT,env=child_env,pass_fds=(locked.fileno(),),check=True)
