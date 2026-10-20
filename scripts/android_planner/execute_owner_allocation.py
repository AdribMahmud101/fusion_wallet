#!/usr/bin/env python3
"""OnePlus runbook for the task-authorized, owner-signed allocation rehearsal.

Without --broadcast this only prepares and simulates a reviewable proposal.
With --broadcast --run-app, both Safe owners authorize the expiring strategy
mandate, then Android independently chooses deposit and later withdrawal. The
edge never chooses a candidate, loads a model or signs an APP transaction.
"""
import argparse, fcntl, json, os, pathlib, re, subprocess, time
from owner_allocation import ROOT, RPC, rpc, cast, calldata, prepare, simulate, verify
from rpc_bridge import command

def credential(name):
    for line in pathlib.Path('/root/projects/fusion_wallet/.env.local').open():
        match=re.match(r'^\s*(?:export\s+)?'+name+r'\s*=\s*([\"\']?)(0x[0-9a-fA-F]{64}|[0-9a-fA-F]{64})\1\s*(?:#.*)?$',line)
        if match: return '0x'+match.group(2).removeprefix('0x')
    raise SystemExit('Required owner credential unavailable; no credential is printed.')

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--broadcast',action='store_true'); ap.add_argument('--run-app',action='store_true')
    ap.add_argument('--resume',action='store_true',help='Reuse the verified existing OWNER mandate; never reads owner keys or re-broadcasts it'); ap.add_argument('--local-host',action='store_true'); ap.add_argument('--serial',default='119703846S004282'); a=ap.parse_args()
    out=ROOT/'evidence/app/agent'
    with open('/root/projects/fusion_wallet/.session.lock','a+') as locked:
        print('Waiting for canonical session flock ...',flush=True); fcntl.flock(locked,fcntl.LOCK_EX)
        if a.resume:
            assert a.run_app, '--resume requires --run-app'
            m=json.loads((ROOT/'deployments/84532.json').read_text())
            recorded=json.loads((out/'K38_owner_allocation_receipt.json').read_text())
            hash=recorded['txHash']; record=verify(hash,m)
            assert record['receipt']['logs']==recorded['receipt']['logs']
            assert int(rpc(RPC,'eth_getBlockByNumber',['latest',False])['timestamp'],16)<record['expiresAt'], 'Owner mandate expired'
            print('Reusing verified OWNER mandate; no owner key read or additional OWNER transaction:',hash,flush=True)
        else:
            m,p=prepare(); simulate(m,p)
            (out/'K38_owner_allocation_proposal.json').write_text(json.dumps(p,indent=2)+'\n')
            print('Fresh owner allocation mandate prepared; local-fork simulation PASS. No yield-profit claim.',flush=True)
            if not a.broadcast:
                print('Read-only preparation complete; no production credentials read or broadcast performed.'); return
            block=rpc(RPC,'eth_getBlockByNumber',['latest',False]); tip=1_000_000; fee=2*int(block['baseFeePerGas'],16)+tip
            assert fee<=10_000_000_000
            owner=credential('OWNER_PRIVATE_KEY'); approver=credential('APPROVER_PRIVATE_KEY')
            env=os.environ.copy(); env.update({'OWNER_PRIVATE_KEY':owner,'APPROVER_PRIVATE_KEY':approver,'DEPLOYER_PRIVATE_KEY':owner,
                'OWNER_ADDRESS':m['roles']['owner'],'APPROVER_ADDRESS':m['roles']['approver'],'MODULE_ADDRESS':m['module'],'SAFE_ADDRESS':m['safe'],
                'K38_AMOUNT':p['assetsRaw'],'K38_NONCE':str(p['depositNonce']),'K38_SESSION_ID':p['sessionId'],'K38_POLICY_HASH':p['policyHash'],
                'K38_EXPIRY':str(p['expiresAt']),'K38_VAULT':p['vault'],'K38_PAYLOAD_HASH':p['payloadHash'],
                'FOUNDRY_BROADCAST':str(ROOT/'scripts/android_planner/.forge/broadcast')})
            cmd=['forge','script','scripts/android_planner/OwnerAllocationMandate.s.sol:OwnerAllocationMandate','--root',str(ROOT),
                '--contracts','scripts/android_planner','--lib-paths','src/contracts/lib','--remappings','forge-std/=src/contracts/lib/forge-std/src/',
                '--use','0.8.24','--via-ir','--evm-version','cancun','--optimize','--optimizer-runs','200',
                '--cache-path','scripts/android_planner/.forge/cache','--out','scripts/android_planner/.forge/out',
                '--rpc-url',RPC,'--broadcast','--slow','--with-gas-price',str(fee),'--priority-gas-price',str(tip)]
            result=subprocess.run(cmd,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
            log=result.stdout.decode(errors='replace')
            for key in (owner,approver):
                log=re.sub(re.escape(key), '[REDACTED]', log, flags=re.IGNORECASE)
                for form in (key,key[2:],key.upper(),key[2:].upper(),str(int(key,16))): log=log.replace(form,'[REDACTED]')
            (out/'K38_owner_allocation_execution.txt').write_text(log)
            for name in ('OWNER_PRIVATE_KEY','APPROVER_PRIVATE_KEY','DEPLOYER_PRIVATE_KEY'): env.pop(name,None)
            owner=approver=None
            if result.returncode: raise SystemExit('OWNER mandate failed; see the redacted execution log. App was not triggered.')
            run=json.loads((ROOT/'scripts/android_planner/.forge/broadcast/OwnerAllocationMandate.s.sol/84532/run-latest.json').read_text())
            assert len(run['receipts'])==1
            hash=run['receipts'][0]['transactionHash']; record=verify(hash,m)
            state=rpc(RPC,'eth_call',[{'to':m['module'],'data':calldata('getState()')},record['receipt']['blockNumber']])
            assert state==p['beforeState'], 'Owner mandate must not alter module authority, reserves or nonce'
            (out/'K38_owner_allocation_receipt.json').write_text(json.dumps(record,indent=2)+'\n')
        metadata=json.dumps({'txHash':hash})+'\n'
        command(a.local_host,a.serial,'run-as','app.fusion.wallet','sh','-c','cat > files/agent/owner_allocation.json',input=metadata.encode())
        print('OWNER allocation mandate confirmed:',hash,flush=True)
        if not a.run_app: return
        child_env={k:v for k,v in os.environ.items() if not k.endswith('PRIVATE_KEY')}; child_env['K17_SESSION_LOCK_FD']=str(locked.fileno())
        def tick(kind):
            call=['python3',str(ROOT/'scripts/android_planner/run_tick.py')]
            call+=['--local-host'] if a.local_host else ['--serial',a.serial]
            subprocess.run(call,cwd=ROOT,env=child_env,pass_fds=(locked.fileno(),),check=True)
            rows=json.loads((out/'golden_app.json').read_text())
            found=[row for row in rows if row.get('allocationOwnerTx')==hash and row['kind']==kind]
            assert len(found)==1, 'Android did not choose the authorized '+kind+'; no fallback or override is permitted'
            return found[0]
        existing=[r for r in json.loads((out/'golden_app.json').read_text()) if r.get('allocationOwnerTx')==hash and r['kind']=='DEPOSIT']
        if existing:
            from verify_app import verify as verify_existing
            assert len(existing)==1; verify_existing(existing[0],m); deposit=existing[0]
        else:
            deposit=tick('DEPOSIT')
        for _ in range(30):
            if int(rpc(RPC,'eth_getBlockByNumber',['latest',False])['number'],16)>deposit['blockNumber']: break
            time.sleep(1)
        else: raise SystemExit('No later block; deposit is recorded but withdrawal was not triggered.')
        withdrawal=tick('WITHDRAW')
        assert withdrawal['blockNumber']>deposit['blockNumber']
        command(a.local_host,a.serial,'run-as','app.fusion.wallet','rm','files/agent/owner_allocation.json',stdout=subprocess.PIPE)
        print('Round trip confirmed through two separate Android decisions and signer calls:',deposit['txHash'],withdrawal['txHash'],flush=True)

if __name__=='__main__': main()
