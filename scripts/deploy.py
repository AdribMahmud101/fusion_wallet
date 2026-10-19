#!/usr/bin/env python3
"""Rehearsable deployment stages; all signing keys are read from the process environment."""
import argparse
import json
import os
import re
import sys
import time
from chain import ROOT, CONTRACTS, IMPLEMENTATION_SLOT, Rpc, artifact, event_abi, keccak, run, signature, write_json
from collect_evidence import collect
from preflight import ADDRESSES

NAMES = {'SAFE':'safe','ASSET':'asset','VAULT_A':'vaultA','VAULT_B':'vaultB','SANCTIONS':'sanctionsList',
         'MODULE':'module','SINGLETON':'safeSingleton','FACTORY':'safeProxyFactory',
         'FALLBACK':'safeFallbackHandler','MULTISEND':'multiSendCallOnly'}
SOURCES = {'safe':('lib/safe-smart-account/contracts/proxies/SafeProxy.sol:SafeProxy','SafeProxy'),
           'asset':('src/MockUSDC.sol:MockUSDC','MockUSDC'),
           'vaultA':('src/KavachiDemoVault.sol:KavachiDemoVault','KavachiDemoVault'),
           'vaultB':('src/KavachiDemoVault.sol:KavachiDemoVault','KavachiDemoVault'),
           'sanctionsList':('src/KavachiTestSanctionsList.sol:KavachiTestSanctionsList','KavachiTestSanctionsList'),
           'module':('src/KavachiModule.sol:KavachiModule','KavachiModule'),
           'policyLibrary':('src/KavachiPolicyLib.sol:KavachiPolicyLib','KavachiPolicyLib'),
           'payeeRegistry':('src/KavachiPayeeRegistry.sol:KavachiPayeeRegistry','KavachiPayeeRegistry'),
           'safeSingleton':('lib/safe-smart-account/contracts/Safe.sol:Safe','Safe'),
           'safeProxyFactory':('lib/safe-smart-account/contracts/proxies/SafeProxyFactory.sol:SafeProxyFactory','SafeProxyFactory'),
           'safeFallbackHandler':('lib/safe-smart-account/contracts/handler/CompatibilityFallbackHandler.sol:CompatibilityFallbackHandler','CompatibilityFallbackHandler'),
           'multiSendCallOnly':('lib/safe-smart-account/contracts/libraries/MultiSendCallOnly.sol:MultiSendCallOnly','MultiSendCallOnly')}


def send(rpc, target=None, data=None, value=None, role='DEPLOYER', gas=None):
    args = ['cast','send','--rpc-url',rpc.url,'--private-key',os.environ[role+'_PRIVATE_KEY'],'--json']
    if target is not None: args.append(target)
    if data is not None: args += ['--data',data] if target else ['--create',data]
    if value is not None: args += ['--value',str(value)]
    if gas: args += ['--gas-limit',str(gas)]
    out = run(args)
    receipt = json.loads(out)
    return rpc.receipt(receipt['transactionHash'])


def read_manifest(chain):
    return json.loads((ROOT / f'deployments/{chain}.json').read_text())


def set_environment(manifest, rpc):
    os.environ['RPC_URL'] = rpc.url
    mapping = {'SAFE_ADDRESS':'safe','MODULE_ADDRESS':'module','TREASURY_ASSET':'asset',
               'VAULT_A':'vaultA','VAULT_B':'vaultB','SAFE_MULTISEND':'multiSendCallOnly'}
    for env, name in mapping.items(): os.environ[env] = manifest['contracts'][name]['address']
    os.environ['AMOUNT_DIVISOR'] = str(manifest['amountDivisor'])
    os.environ['OBLIGATIONS_DUE_AT'] = str(manifest['obligationsDueAt'])


def script(name, rpc, library):
    args = ['forge','script',f'script/{name}.s.sol:{name}','--rpc-url',rpc.url,
            '--sender',os.environ['DEPLOYER_ADDRESS'],'--broadcast','--slow','--non-interactive',
            '--skip','test','--skip','GenVectors',
            '--libraries',f'src/KavachiPolicyLib.sol:KavachiPolicyLib:{library}']
    output = run(args)
    chain_id = int(rpc.call('eth_chainId', []),16)
    broadcast = json.loads((CONTRACTS / f'broadcast/{name}.s.sol/{chain_id}/run-latest.json').read_text())
    receipts = [rpc.receipt(tx['hash']) for tx in broadcast['transactions']]
    for receipt in receipts:
        if int(receipt['status'],16) != 1: raise ValueError('Broadcast reverted: '+receipt['transactionHash'])
    return output, receipts


def metadata(rpc, name, addr, receipt=None, source=None):
    block = int(rpc.call('eth_blockNumber', []),16)
    code = rpc.call('eth_getCode', [addr,'latest'])
    if code == '0x': raise ValueError('Empty code for ' + name)
    item = {'address':addr,'runtimeCodeHash':keccak(code),'runtimeCodeBytes':(len(code)-2)//2,
            'observedAtBlock':block,'source':source or SOURCES.get(name,('',''))[0]}
    if receipt:
        item.update(deploymentTxHash=receipt['transactionHash'],deploymentBlock=int(receipt['blockNumber'],16))
    slot = rpc.call('eth_getStorageAt', [addr,IMPLEMENTATION_SLOT,'latest'])
    if int(slot,16): item['implementation'] = '0x'+slot[-40:]
    if int(rpc.call('eth_chainId',[]),16) == 84532:
        item['explorer'] = 'https://sepolia.basescan.org/address/'+addr
        item['verification'] = {'status':'PENDING','url':item['explorer']+'#code'}
    return item


def deploy(chain, rpc, profile):
    if int(rpc.call('eth_chainId',[]),16) != chain: raise ValueError('RPC chain mismatch')
    target = ROOT / f'deployments/{chain}.json'
    if target.exists(): raise ValueError('Manifest already exists; use setup/verify, or a fresh Anvil port and archive the old local manifest')
    for role in ['DEPLOYER','OWNER','APPROVER','GUARDIAN','SESSION']:
        derived = run(['cast','wallet','address','--private-key',os.environ[role+'_PRIVATE_KEY']])
        if derived.lower() != os.environ[role+'_ADDRESS'].lower(): raise ValueError(role+' key/address mismatch')
    if chain == 31337:
        rpc.call('anvil_setBalance',[os.environ['DEPLOYER_ADDRESS'],hex(10**18)])
        for name in ['SAFE_SINGLETON','SAFE_FACTORY','SAFE_FALLBACK_HANDLER','SAFE_MULTISEND','TREASURY_ASSET','VAULT_A']:
            os.environ.pop(name,None)
    else:
        balance = int(rpc.call('eth_getBalance',[os.environ['DEPLOYER_ADDRESS'],'latest']),16)
        if balance < 6*10**16:
            raise ValueError(f'DEPLOYER requires at least 0.06 ETH for the authorized 0.05 ETH role funding plus deployment gas; currently {balance} wei. No transactions sent.')
        os.environ.update(SAFE_SINGLETON=ADDRESSES['SafeL2'], SAFE_FACTORY=ADDRESSES['SafeProxyFactory'],
                          SAFE_FALLBACK_HANDLER=ADDRESSES['CompatibilityFallbackHandler'], SAFE_MULTISEND=ADDRESSES['MultiSendCallOnly'])
        os.environ['TREASURY_ASSET'] = ADDRESSES['asset'] if profile == 'FULL' else '0x036CbD53842c5426634e7929541eC2318f3dCF7e'
        os.environ['VAULT_A'] = ADDRESSES['vaultA'] if profile == 'FULL' else '0x0000000000000000000000000000000000000000'
    print('Deploying on',chain,profile,flush=True)
    run(['forge','build','--skip','test','--skip','GenVectors','--quiet'])
    library_receipt = send(rpc,data=artifact('KavachiPolicyLib')['bytecode']['object'])
    library = library_receipt['contractAddress']
    os.environ['DEPLOY_SALT'] = str(int(time.time()))
    output, receipts = script('Deploy',rpc,library)
    addresses = {NAMES[label]:addr for label,addr in re.findall(r'DEPLOY_(\w+)\s+(0x[0-9a-fA-F]{40})',output)}
    if set(addresses) != set(NAMES.values()): raise ValueError('Deployment address logs incomplete')
    addresses['policyLibrary'] = library
    addresses['payeeRegistry'] = run(['cast','compute-address',addresses['module'],'--nonce','1']).split()[-1]
    block = rpc.call('eth_getBlockByNumber',['latest',False])
    manifest = {'chainId':chain,'status':'DEPLOYED','profile':profile,'amountDivisor':1 if profile=='FULL' else 100,
        'rpcUrl':rpc.url,'compiler':{'version':'0.8.24+commit.e11b9ed9','evmVersion':'cancun','viaIR':True,
                                  'optimizer':True,'optimizerRuns':200,'officialSafeViaIR':False},
        'owners':[os.environ['OWNER_ADDRESS'],os.environ['APPROVER_ADDRESS']],'threshold':2,
        'roles':{k.lower():os.environ[k+'_ADDRESS'] for k in ['DEPLOYER','OWNER','APPROVER','GUARDIAN','SESSION','OPS']},
        'payeeDelaySeconds':3600,'payeeDelayLabel':'demo setting; production default 24 h',
        'obligationsDueAt':int(block['timestamp'],16)+2*86400,'contracts':{},
        'labels':{'sanctions':'TEST LIST','demoVaultYield':'SIMULATED','agentSource':'SCRIPT','planner':'deterministic'}}
    for name,addr in addresses.items():
        receipt = next((r for r in receipts if (r.get('contractAddress') or '').lower()==addr.lower()),None)
        if name=='policyLibrary':receipt=library_receipt
        if name in ['safe','payeeRegistry']:
            event_name = 'ProxyCreation' if name=='safe' else None
            if event_name:
                topic=keccak(signature(event_abi(event_name)))
                receipt=next(r for r in receipts if any(l['topics'] and l['topics'][0].lower()==topic for l in r['logs']))
            else: receipt=next(r for r in receipts if (r.get('contractAddress') or '').lower()==addresses['module'].lower())
        source = SOURCES[name][0]
        if chain==84532 and name=='asset': source='Aave test USDC (external)' if profile=='FULL' else 'Circle test USDC (external)'
        if chain==84532 and name=='vaultA' and profile=='FULL': source='Aave waUSDC (external ERC-4626)'
        if chain==84532 and name=='safeSingleton': source='Safe v1.4.1 SafeL2 (canonical external deployment)'
        manifest['contracts'][name]=metadata(rpc,name,addr,receipt,source)
    write_json(target,manifest)
    evidence = ROOT / ('evidence/anvil.json' if chain==31337 else 'evidence/base-sepolia.json')
    safe_receipt = next(r for r in receipts if any(l['address'].lower()==addresses['safe'].lower() for l in r['logs']))
    collect(evidence,rpc,tx=safe_receipt['transactionHash'],evidence_id='E1',kind='SAFE_CREATION',
            scenario='Official Safe v1.4.1, two owners, threshold 2',event='SafeSetup',emitter=addresses['safe'])
    print('Manifest written:',target.relative_to(ROOT),flush=True)
    return manifest


def setup(chain, rpc):
    manifest=read_manifest(chain)
    if manifest['status']=='SETUP_COMPLETE':raise ValueError('Setup already complete; refusing to reset policy/session')
    set_environment(manifest,rpc)
    evidence=ROOT / ('evidence/anvil.json' if chain==31337 else 'evidence/base-sepolia.json')
    funding=manifest.setdefault('funding',[])
    # Idempotent target balances avoid double-funding on a retry.
    for role,amount in [('OWNER',10**16),('APPROVER',10**16),('GUARDIAN',10**16),('SESSION',2*10**16)]:
        balance=int(rpc.call('eth_getBalance',[os.environ[role+'_ADDRESS'],'latest']),16)
        if balance < amount:
            receipt=send(rpc,os.environ[role+'_ADDRESS'],value=amount-balance)
            funding.append({'role':role,'txHash':receipt['transactionHash'],'amountWei':str(amount-balance)})
            write_json(ROOT/f'deployments/{chain}.json',manifest)
    safe=manifest['contracts']['safe']['address']; asset=manifest['contracts']['asset']['address']
    balance=int(run(['cast','call',asset,'balanceOf(address)(uint256)',safe,'--rpc-url',rpc.url]).split()[0])
    target=3000*10**6 if chain==31337 else 10000*10**6
    if manifest['profile']=='SCALED': target=30*10**6
    if balance<target:
        if chain==31337:
            calldata=run(['cast','calldata','mint(address,uint256)',safe,str(target-balance)])
            receipt=send(rpc,asset,data=calldata)
        elif manifest['profile']=='FULL':
            calldata=run(['cast','calldata','mint(address,address,uint256)',asset,safe,str(target-balance)])
            receipt=send(rpc,ADDRESSES['faucet'],data=calldata)
        else:raise ValueError('SCALED needs at least 30 Circle USDC in the Safe; obtain faucet tokens before setup')
        if int(receipt['status'],16)!=1:raise ValueError('Treasury funding transaction reverted')
        funding.append({'role':'TREASURY_USDC','txHash':receipt['transactionHash'],'amountRaw':str(target-balance)})
        write_json(ROOT/f'deployments/{chain}.json',manifest)
    _,receipts=script('Setup',rpc,manifest['contracts']['policyLibrary']['address'])
    events=[('EnabledModule','E2','MODULE_ENABLE',safe),('PolicyConfigured','E3','POLICY_CONFIGURED',os.environ['MODULE_ADDRESS']),
            ('ObligationsCommitted','E5','OBLIGATIONS_COMMITTED',os.environ['MODULE_ADDRESS']),
            ('SessionStarted','E6','SESSION_STARTED',os.environ['MODULE_ADDRESS']),('Resumed','E6','RESUMED',os.environ['MODULE_ADDRESS'])]
    for event,eid,kind,emitter in events:
        topic=keccak(signature(event_abi(event)))
        receipt=next(r for r in receipts if any(l['address'].lower()==emitter.lower() and l['topics'] and l['topics'][0].lower()==topic for l in r['logs']))
        collect(evidence,rpc,tx=receipt['transactionHash'],evidence_id=eid,kind=kind,scenario='D17 threshold-two owner setup',event=event,emitter=emitter)
    topic=keccak(signature(event_abi('PayeeAdded')))
    active_at=[]
    for receipt in receipts:
        if any(l['topics'] and l['topics'][0].lower()==topic for l in receipt['logs']):
            row=collect(evidence,rpc,tx=receipt['transactionHash'],evidence_id='E4',kind='PAYEE_ADDED',scenario='Payee cooling started (3600-second demo setting)',event='PayeeAdded',emitter=os.environ['MODULE_ADDRESS'])
            active_at.append(int(row['expectedEvent']['fields']['activeAt']))
    manifest.update(status='SETUP_COMPLETE',payeesActiveAt=max(active_at),setupTxHashes=[r['transactionHash'] for r in receipts])
    write_json(ROOT/f'deployments/{chain}.json',manifest)
    print('E1–E6 receipts collected. Payees active at Unix',manifest['payeesActiveAt'],flush=True)
    return manifest


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('stage',choices=['deploy','setup','rehearse'])
    p.add_argument('--chain',type=int,choices=[31337,84532],default=84532)
    p.add_argument('--rpc-url');p.add_argument('--profile',choices=['FULL','SCALED'],default='FULL')
    p.add_argument('--reset-local',action='store_true',help='Reset chain 31337 and replace its local manifest/evidence; rehearsal only')
    a=p.parse_args(); chain=31337 if a.stage=='rehearse' else a.chain
    rpc=Rpc(a.rpc_url or ('http://127.0.0.1:18545' if chain==31337 else 'https://sepolia.base.org'))
    if a.reset_local:
        if a.stage!='rehearse' or int(rpc.call('eth_chainId',[]),16)!=31337:
            raise ValueError('--reset-local requires an Anvil rehearsal on chain 31337')
        rpc.call('anvil_reset',[])
        for path in [ROOT/'deployments/31337.json',ROOT/'evidence/anvil.json']:
            if path.exists():path.unlink()
    if a.stage in ['deploy','rehearse']: deploy(chain,rpc,a.profile)
    if a.stage in ['setup','rehearse']: setup(chain,rpc)
    if a.stage=='rehearse':
        from evidence_actions import rehearse
        rehearse(read_manifest(chain),rpc)


if __name__=='__main__':
    try: main()
    except Exception as error:
        from chain import redact
        print('BLOCKED:',redact(str(error)),file=sys.stderr);sys.exit(1)
