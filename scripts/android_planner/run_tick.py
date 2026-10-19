#!/usr/bin/env python3
"""Holds the canonical session flock across app trigger, broadcast, receipt and capture.

RPC forwarding only. The edge never loads a planner or session key and never
creates/signs a transaction. The app performs all financial decisions and signing.
"""
import argparse,json,pathlib,subprocess,threading,time,uuid
from rpc_bridge import Server,command,provision,wait_for_receipt
ROOT=pathlib.Path(__file__).resolve().parents[2]
AP=argparse.ArgumentParser(); AP.add_argument('--local-host',action='store_true'); AP.add_argument('--serial',default='192.168.0.105:5555'); AP.add_argument('--dry-run',action='store_true'); args=AP.parse_args()
def app_read(path):
 return command(args.local_host,args.serial,'run-as','app.fusion.wallet','cat',path,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL).stdout
request_id=str(uuid.uuid4())
try: previous=app_read('files/agent/edge_rpc.json')
except subprocess.CalledProcessError: previous=None
print('Waiting for /root/projects/fusion_wallet/.session.lock ...',flush=True)
server=Server(0,locked=True,request_id=request_id)
thread=threading.Thread(target=server.serve_forever,daemon=True); thread.start()
try:
 print('Session flock held; triggering Android app tick',request_id,flush=True)
 provision(server,args.local_host,args.serial)
 mode='dry-run' if args.dry_run else 'broadcast'
 command(args.local_host,args.serial,'input','keyevent','KEYCODE_WAKEUP',stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 command(args.local_host,args.serial,'wm','dismiss-keyguard',stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 # Android's cmd passes stdio FDs to system_server over Binder. A redirected
 # edge log is a chroot file which system_server cannot accept; use pipes.
 triggered=command(args.local_host,args.serial,'am','broadcast','--allow-background-activity-starts','-n','app.fusion.wallet/.agent.AgentControlReceiver','--es','mode',mode,'--es','requestId',request_id,
                   stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 print(triggered.stdout.decode(errors='replace').strip(),flush=True)
 result=None; deadline=time.monotonic()+360
 while time.monotonic()<deadline:
  try:
   current=json.loads(app_read('files/agent/control_result.json'))
   if current.get('requestId')==request_id: result=current; break
  except (subprocess.CalledProcessError,json.JSONDecodeError): pass
  time.sleep(1)
 if result is None:
  # Never release the session lock with a known unresolved app-signed attempt.
  try: pending=json.loads(app_read('no_backup/agent/pending.json'))
  except (subprocess.CalledProcessError,json.JSONDecodeError): pending=None
  if pending:
   print('App reply timed out; keeping flock while waiting for its signed transaction receipt.',flush=True)
   wait_for_receipt(server,pending['txHash'])
  raise SystemExit('App tick did not return a complete verified result; no success claimed.')
 out=ROOT/'evidence/app/agent'; out.mkdir(parents=True,exist_ok=True)
 if 'error'in result:
  try: pending=json.loads(app_read('no_backup/agent/pending.json'))
  except (subprocess.CalledProcessError,json.JSONDecodeError): pending=None
  if pending:
   print('Signer has a durable attempt; holding flock through its receipt before reporting the error.',flush=True)
   wait_for_receipt(server,pending['txHash'])
  (out/'last_tick_error.json').write_text(json.dumps(result,indent=2)+'\n')
  raise SystemExit('App tick refused: '+result['error'])
 if args.dry_run or 'execution'not in result:
  (out/'locked_dry_run.json').write_text(json.dumps(result,indent=2)+'\n')
  print('App chose',json.loads(result['passportJson'])['choiceLabel'],'without a transaction.',flush=True)
 else:
  execution=result['execution']; receipt=execution['receipt']; tx=execution['transaction']; passport=json.loads(result['passportJson'])
  manifest=json.loads((ROOT/'deployments/84532.json').read_text())
  assert execution['status']==1 and int(receipt['status'],16)==1
  assert receipt['from'].lower()==manifest['roles']['session'].lower() and receipt['to'].lower()==manifest['module'].lower()
  assert result['decisionHash'][2:].lower() in tx['input'].lower()
  assert passport['planner']['kind']=='mnn+ahem-g@android' and passport['planner']['host']=='device'
  assert result['model']['process'].endswith(':inference') and execution['signerProcess'].endswith(':signer')
  tx_hash=execution['txHash']
  for folder in ['passports','receipts','transactions','runs']: (out/folder).mkdir(exist_ok=True)
  passport_path=out/'passports'/f'{tx_hash}.json'; passport_path.write_text(result['passportJson'])
  (out/'receipts'/f'{tx_hash}.json').write_text(json.dumps(receipt,indent=2)+'\n')
  (out/'transactions'/f'{tx_hash}.json').write_text(json.dumps(tx,indent=2)+'\n')
  (out/'runs'/f'{tx_hash}.json').write_text(json.dumps(result,indent=2)+'\n')
  # Independent on-edge hash check on exact app-produced passport bytes.
  assert subprocess.check_output(['cast','keccak',result['passportJson']],text=True).strip()==result['decisionHash']
  row={'id':'K17-APP','source':'APP','chainId':84532,'planner':passport['planner'],'host':'device','device':result['device'],
       'txHash':tx_hash,'decisionHash':result['decisionHash'],'actionDigest':execution['actionDigest'],
       'candidateId':passport['choiceId'],'choiceLabel':passport['choiceLabel'],
       'kind':['DEPOSIT','WITHDRAW','REBALANCE','PAY_INSTANT'][result['chosenCandidate']['kind']],
       'route':passport['route'],'status':1,'blockNumber':int(receipt['blockNumber'],16),'gasUsed':int(receipt['gasUsed'],16),
       'from':receipt['from'],'to':receipt['to'],'passport':str(passport_path.relative_to(ROOT)),
       'receipt':str((out/'receipts'/f'{tx_hash}.json').relative_to(ROOT)),
       'transaction':str((out/'transactions'/f'{tx_hash}.json').relative_to(ROOT)),
       'sessionLock':'fcntl.flock LOCK_EX held by edge run_tick.py through verified receipt'}
  golden=out/'golden_app.json'; rows=json.loads(golden.read_text()) if golden.exists() else []
  rows=[r for r in rows if r['txHash']!=tx_hash]+[row]; golden.write_text(json.dumps(rows,indent=2)+'\n')
  print('APP transaction confirmed:',tx_hash,'status=1; session sender, event, calldata and passport hash verified.',flush=True)
finally:
 # Stop admitting writes while retaining the actual flock. Any in-flight send
 # completes before this mutex returns, and its durable journal then exists.
 server.disable_writes()
 try: pending=json.loads(app_read('no_backup/agent/pending.json'))
 except (subprocess.CalledProcessError,json.JSONDecodeError): pending=None
 if pending:
  print('Durable attempt remains; flock stays held until its canonical receipt.',flush=True)
  wait_for_receipt(server,pending['txHash'])
 if previous is not None:
  command(args.local_host,args.serial,'run-as','app.fusion.wallet','sh','-c','cat > files/agent/edge_rpc.json',input=previous)
 server.shutdown(); server.server_close(); thread.join(timeout=5)
 if not args.local_host:
  subprocess.run(['adb','-s',args.serial,'reverse','--remove','tcp:'+str(server.server_address[1])],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 print('Session flock released.',flush=True)
