#!/usr/bin/env python3
"""K2 operational evidence, explicitly SCRIPT/deterministic/linux-host. No Golden* files."""
import argparse
import json
import os
from pathlib import Path
import time
from chain import ROOT, Rpc, event_abi, keccak, run, signature, write_json
from collect_evidence import collect
from deploy import read_manifest, script, send, set_environment

ZERO_ADDRESS='0x'+'00'*20
ZERO_HASH='0x'+'00'*32
ACTION_FIELDS=['kind','sourceVault','destinationVault','assets','minSharesOut','maxSharesBurn',
               'obligationIdHash','payeeIdHash','recipient','dueAt','expectedNonce','policyEpoch','deadline','decisionHash']
ACTION_TYPE='(uint8,address,address,uint256,uint256,uint256,bytes32,bytes32,address,uint64,uint256,uint256,uint64,bytes32)'


def word(value):
    if isinstance(value,str) and value.startswith('0x'): return value[2:].zfill(64)
    return f'{int(value):064x}'


def encode_action(action): return '0x'+''.join(word(action[k]) for k in ACTION_FIELDS)


def action_digest(chain_id,safe,module,mandate,session,action):
    domain=[keccak('KAVACHI_ACTION_V4_1'),chain_id,safe,module,mandate,session]
    return keccak('0x'+''.join(word(v) for v in domain)+encode_action(action)[2:])


def proof_data(proof): return '0x'+word(32)+word(len(proof))+''.join(word(x) for x in proof)


def action_calldata(operation,action,proof):
    selector=keccak(operation+'('+ACTION_TYPE+',bytes32[])')[:10]
    return selector+encode_action(action)[2:]+word(15*32)+word(len(proof))+''.join(word(x) for x in proof)


def state(rpc,module):
    raw=rpc.call('eth_call',[{'to':module,'data':keccak('getState()')[:10]},'latest'])[2:]
    data=[raw[i:i+64] for i in range(0,len(raw),64)]
    return {'paused':bool(int(data[1],16)),'epoch':int(data[2],16),'mandate':'0x'+data[4],
            'nonce':int(data[8],16),'session':'0x'+data[11],'expiresAt':int(data[14],16)}


def obligations(manifest):
    divisor=manifest['amountDivisor'];due=manifest['obligationsDueAt']
    amounts=[8,75,150,300,50]
    bills=[]
    for i,amount in enumerate(amounts):
        bill=keccak('0x'+(b'fusion.k2.bill.'+bytes([i])).hex())
        payee=keccak('fusion.k2.payee.'+('acme' if i<2 else 'payroll' if i<4 else 'newbank'))
        amount=amount*10**6//divisor
        inner=keccak('0x'+word(bill)+word(payee)+word(amount)+word(due))
        leaf=keccak(inner)
        bills.append({'obligationIdHash':bill,'payeeIdHash':payee,'assets':str(amount),'dueAt':str(due),'leaf':leaf})
    leaves=sorted(b['leaf'] for b in bills)
    tree=[None]*(len(leaves)*2-1)
    for i,leaf in enumerate(leaves):tree[len(tree)-1-i]=leaf
    for i in reversed(range(len(leaves)-1)):
        pair=sorted([tree[2*i+1],tree[2*i+2]])
        tree[i]=keccak('0x'+pair[0][2:]+pair[1][2:])
    for bill in bills:
        index=len(tree)-1-leaves.index(bill['leaf']);proof=[]
        while index>0:
            proof.append(tree[index-1 if index%2==0 else index+1]);index=(index-1)//2
        bill['proof']=proof
        bill['recipient']=manifest['roles']['owner' if bills.index(bill)<2 else 'approver' if bills.index(bill)<4 else 'ops']
    return tree[0],bills


class Evidence:
    def __init__(self,manifest,rpc):
        self.manifest=manifest;self.rpc=rpc;set_environment(manifest,rpc)
        self.chain=manifest['chainId'];self.module=os.environ['MODULE_ADDRESS'];self.safe=os.environ['SAFE_ADDRESS']
        self.file=ROOT/('evidence/anvil.json' if self.chain==31337 else 'evidence/base-sepolia.json')
        self.root,self.bills=obligations(manifest)
        write_json(ROOT/f'evidence/obligations-{self.chain}.json',{'root':self.root,'bills':self.bills})

    def has(self,kind=None,scenario=None):
        rows=json.loads(self.file.read_text())['rows']
        return any((kind is None or r['kind']==kind) and (scenario is None or r['scenario']==scenario) for r in rows)

    def action(self,kind,amount=None,bill=None,delayed=False,approved=False):
        st=state(self.rpc,self.module)
        block=self.rpc.call('eth_getBlockByNumber',['latest',False]);now=int(block['timestamp'],16)
        a=dict(kind=str(kind),sourceVault=ZERO_ADDRESS,destinationVault=ZERO_ADDRESS,assets=str(amount or 0),
               minSharesOut='0',maxSharesBurn='0',obligationIdHash=ZERO_HASH,payeeIdHash=ZERO_HASH,
               recipient=ZERO_ADDRESS,dueAt='0',expectedNonce=str(st['nonce']),policyEpoch=str(st['epoch']),
               deadline=str(now+(850 if approved else 800 if delayed else 115)),decisionHash=ZERO_HASH)
        if kind==0:
            a['destinationVault']=self.manifest['contracts']['vaultA']['address']
            preview=int(run(['cast','call',a['destinationVault'],'previewDeposit(uint256)(uint256)',str(amount),'--rpc-url',self.rpc.url]).split()[0])
            # 40 bps leaves room inside the 50 bps policy as the live Aave index moves.
            a['minSharesOut']=str(max(1,preview*9960//10000))
        elif kind==1:
            a['sourceVault']=self.manifest['contracts']['vaultA']['address']
            preview=int(run(['cast','call',a['sourceVault'],'previewWithdraw(uint256)(uint256)',str(amount),'--rpc-url',self.rpc.url]).split()[0])
            a['maxSharesBurn']=str((preview*10040+9999)//10000)
        elif kind==3:
            a.update({k:bill[k] for k in ['assets','obligationIdHash','payeeIdHash','recipient','dueAt']})
        digest_zero=action_digest(self.chain,self.safe,self.module,st['mandate'],st['session'],a)
        passport={'version':'k2-operational-v1','chainId':self.chain,'safe':self.safe,'module':self.module,
                  'snapshot':{'block':str(int(block['number'],16)),'hash':block['hash']},
                  'mandateHash':st['mandate'],'sessionId':st['session'],'action':a.copy(),
                  'actionDigestWithoutDecision':digest_zero,
                  'planner':{'kind':'deterministic','host':'linux-host'},
                  'source':'SCRIPT','reasonCode':'K2_OPERATIONAL_EVIDENCE','createdAtChain':str(now)}
        raw=json.dumps(passport,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        decision=keccak('0x'+raw.hex());a['decisionHash']=decision
        path=Path(f'evidence/passports/{self.chain}-{decision[2:]}.json')
        (ROOT/path).write_bytes(raw)
        return a,(bill['proof'] if bill else []),path.as_posix()

    def operation(self,operation,action=None,proof=None):
        os.environ['EVIDENCE_OPERATION']=operation
        if action:os.environ['ACTION_DATA']=encode_action(action)
        os.environ['PROOF_DATA']=proof_data(proof or [])
        _,receipts=script('EvidenceActions',self.rpc,self.manifest['contracts']['policyLibrary']['address'])
        return receipts[-1]

    def record(self,receipt,eid,kind,event,passport=None,fields=None):
        row=collect(self.file,self.rpc,tx=receipt['transactionHash'],evidence_id=eid,kind=kind,
            scenario='K2 operational script; deterministic, Linux host',event=event,emitter=self.module,
            fields=fields,passport=passport)
        print(eid,kind,row['txHash'],flush=True)
        return row

    def execute(self,a,proof,passport,eid,kind,approved=False):
        if approved:
            r=self.operation('approve',a,proof);self.record(r,'E9','ACTION_APPROVED','ActionApproved')
        r=self.operation('execute',a,proof)
        return self.record(r,eid,kind,'ActionExecuted',passport)

    def queue(self,a,proof,passport):
        r=self.operation('queue',a,proof)
        return self.record(r,'E8','ACTION_QUEUED','ActionQueued',passport)

    def revert(self,a,proof,passport,error,scenario,operation='execute'):
        data=action_calldata(operation,a,proof)
        selector=keccak(error+'()')[:10]
        # Pre-check the exact error before deliberately paying gas for the labelled attack.
        try:self.rpc.call('eth_call',[{'from':self.manifest['roles']['session'],'to':self.module,'data':data},'latest'])
        except RuntimeError as failure:
            if selector.lower() not in str(failure).lower():raise ValueError('Wrong simulated attack error: '+str(failure))
        else:raise ValueError('Attack simulation unexpectedly succeeded; refusing broadcast')
        r=send(self.rpc,self.module,data=data,role='SESSION',gas=1000000)
        # Reverted transactions have no emitted decision event; publish their signed input and passport instead.
        row=collect(self.file,self.rpc,tx=r['transactionHash'],evidence_id='E16',kind='INTENTIONAL_REVERT',
            scenario=scenario,status=0,revert_selector=selector,decision_hash=a['decisionHash'],passport=passport)
        published=json.loads((ROOT/passport).read_bytes())
        row['actionDigest']=action_digest(self.chain,self.safe,self.module,published['mandateHash'],published['sessionId'],a)
        doc=json.loads(self.file.read_text());doc['rows'][-1]=row;write_json(self.file,doc)
        print('E16',scenario,r['transactionHash'],flush=True)


def wait_until(rpc,stamp,local=False):
    block=rpc.call('eth_getBlockByNumber',['latest',False]);now=int(block['timestamp'],16)
    if now>=stamp:return
    if local:
        if int(rpc.call('eth_chainId',[]),16)!=31337:raise ValueError('Time manipulation is Anvil only')
        rpc.call('evm_increaseTime',[stamp-now]);rpc.call('evm_mine',[])
    else:
        raise ValueError(f'Chain time {now}; action is eligible at {stamp}. Run again after {stamp-now} seconds.')


def before_cooling(e):
    scale=e.manifest['amountDivisor']
    # Owner-tier investment starts before payee cooling finishes.
    if not e.has(kind='DEPOSIT'):
        a,p,path=e.action(0,1850*10**6//scale,approved=True)
        e.execute(a,p,path,'E10','DEPOSIT',approved=True)
    # The instant PAY uses a cooling payee and must fail with PayeeNotActive.
    if not e.has(scenario='Cooling payee rejected'):
        a,p,path=e.action(3,bill=e.bills[0]);e.revert(a,p,path,'PayeeNotActive','Cooling payee rejected')
    if not e.has(scenario='Delayed action cannot bypass queue'):
        a,p,path=e.action(0,50*10**6//scale)
        e.revert(a,p,path,'QueueRequired','Delayed action cannot bypass queue')
    if e.chain==31337 and not e.has(scenario='Reserve raid rejected before queue creation'):
        a,p,path=e.action(0,100*10**6//scale,delayed=True)
        e.revert(a,p,path,'ReserveViolation','Reserve raid rejected before queue creation','queueAction')


def after_cooling(e,local=False):
    scale=e.manifest['amountDivisor']
    wait_until(e.rpc,e.manifest['payeesActiveAt'],local)
    a,p,path=e.action(3,bill=e.bills[0]);e.execute(a,p,path,'E7','PAY_INSTANT')
    # Queue/pay execution intentionally spans the on-chain 180-second delay.
    a,p,path=e.action(3,bill=e.bills[1],delayed=True);e.queue(a,p,path)
    queue_time=int(e.rpc.call('eth_getBlockByNumber',['latest',False])['timestamp'],16)
    if local:wait_until(e.rpc,queue_time+181,True)
    else:
        write_json(ROOT/f'evidence/pending-queue-{e.chain}.json',{'action':a,'proof':p,'passport':path,'readyAt':queue_time+181})
        print('Queued PAY. Resume with evidence_actions.py finish-queue after Unix',queue_time+181,flush=True)
        return
    e.execute(a,p,path,'E8','PAY_DELAYED')
    finish_after_queue(e)


def finish_after_queue(e):
    scale=e.manifest['amountDivisor']
    a,p,path=e.action(0,50*10**6//scale,delayed=True);e.queue(a,p,path)
    e.record(e.operation('cancel'),'E12','QUEUE_CANCELLED','QueueCancelled')
    a,p,path=e.action(3,bill=e.bills[0]);e.revert(a,p,path,'ObligationAlreadyPaid','Duplicate committed bill payment rejected')
    a,p,path=e.action(3,bill=e.bills[2],approved=True);e.execute(a,p,path,'E9','PAY_OWNER_TIER',approved=True)
    a,p,path=e.action(3,bill=e.bills[3]);e.record(e.operation('ownerPay',a,p),'E13','OWNER_PAY','ObligationPaid')
    e.record(e.operation('newPayee'),'E4','PAYEE_NEW_BANK','PayeeAdded')
    a,p,path=e.action(3,bill=e.bills[4],delayed=True)
    e.revert(a,p,path,'PayeeNotActive','Changed bank details require fresh cooling','queueAction')
    a,p,path=e.action(1,10*10**6//scale);e.execute(a,p,path,'E11','WITHDRAW')
    e.record(e.operation('pause'),'E14','GUARDIAN_PAUSE','Paused')
    e.record(e.operation('resume'),'E6','OWNER_RESUME','Resumed')
    e.record(e.operation('unwind'),'E15','EMERGENCY_UNWIND','EmergencyUnwind')


def rehearse(manifest,rpc):
    if manifest['chainId']!=31337:raise ValueError('Rehearsal time jumps require chain 31337')
    e=Evidence(manifest,rpc);before_cooling(e);after_cooling(e,local=True)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('stage',choices=['before-cooling','after-cooling','finish-queue','rehearse'])
    p.add_argument('--chain',type=int,choices=[31337,84532],default=84532);p.add_argument('--rpc-url')
    a=p.parse_args();manifest=read_manifest(a.chain);rpc=Rpc(a.rpc_url or manifest['rpcUrl'])
    if int(rpc.call('eth_chainId',[]),16)!=a.chain:raise ValueError('RPC chain mismatch')
    if a.stage=='rehearse':return rehearse(manifest,rpc)
    e=Evidence(manifest,rpc)
    if a.stage=='before-cooling':before_cooling(e)
    elif a.stage=='after-cooling':after_cooling(e)
    else:
        pending=json.loads((ROOT/f'evidence/pending-queue-{a.chain}.json').read_text())
        wait_until(rpc,pending['readyAt'])
        e.execute(pending['action'],pending['proof'],pending['passport'],'E8','PAY_DELAYED')
        finish_after_queue(e)


if __name__=='__main__':main()
