#!/usr/bin/env python3
"""Source-verify every project deployment with Sourcify (no API key required)."""
import argparse
import json
import os
import sys
import tempfile
from pathlib import Path
import urllib.error
import urllib.request
from datetime import datetime, timezone
from chain import ROOT, run, write_json


def sourcify_status(address):
    url='https://sourcify.dev/server/v2/contract/84532/'+address
    try:
        with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'curl/8.0'}),timeout=30) as r:
            data=json.load(r)
        return data
    except urllib.error.HTTPError as error:
        if error.code==404:return None
        raise


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--manifest',default=str(ROOT/'deployments/84532.json'))
    a=p.parse_args();path=ROOT/a.manifest if not a.manifest.startswith('/') else __import__('pathlib').Path(a.manifest)
    manifest=json.loads(path.read_text())
    if manifest['chainId']!=84532:raise ValueError('This verification script is for Base Sepolia only')
    library=manifest['contracts']['policyLibrary']['address'];failed=[]
    for name,item in manifest['contracts'].items():
        address=item['address'];source=item['source']
        url=f'https://repo.sourcify.dev/contracts/full_match/84532/{address}/'
        verification=item.setdefault('verification',{})
        if not item.get('deploymentTxHash'):
            # External contracts retain their explorer link; do not submit a different compiler build.
            verification.update(status='EXTERNAL',url=f'https://sepolia.basescan.org/address/{address}#code',
                                note='Canonical external contract; runtime and implementation pins verified independently')
            write_json(path,manifest);continue
        args=['forge','verify-contract',address,source,'--chain','84532','--verifier','sourcify',
              '--compiler-version','0.8.24','--evm-version','cancun','--num-of-optimizations','200',
              '--creation-transaction-hash',item['deploymentTxHash'],'--watch','--retries','40','--delay','3']
        if name not in ['safe','safeSingleton','safeProxyFactory','safeFallbackHandler','multiSendCallOnly']:
            args+=['--via-ir']
        else:args+=['--compilation-profile','safe-legacy']
        if name=='module':args+=['--libraries',f'src/KavachiPolicyLib.sol:KavachiPolicyLib:{library}']
        try:
            if name == 'safe' and not manifest['contracts']['safeProxyFactory'].get('deploymentTxHash'):
                # The canonical factory emits official 0.7.6 / Istanbul / unoptimized proxies.
                # Compile the same unmodified source under its original source-unit path.
                with tempfile.TemporaryDirectory(prefix='safe-verify-',dir=ROOT/'scripts') as temp:
                    work=Path(temp);(work/'contracts/proxies').mkdir(parents=True)
                    (work/'contracts/proxies/SafeProxy.sol').write_bytes(
                        (ROOT/'src/contracts/lib/safe-smart-account/contracts/proxies/SafeProxy.sol').read_bytes())
                    (work/'foundry.toml').write_text('[profile.default]\nsrc="contracts"\nlibs=[]\nsolc_version="0.7.6"\nevm_version="istanbul"\noptimizer=false\noptimizer_runs=200\nuse_literal_content=true\n')
                    args=['forge','verify-contract',address,'contracts/proxies/SafeProxy.sol:SafeProxy',
                          '--root',str(work),'--chain','84532','--verifier','sourcify','--compiler-version','0.7.6',
                          '--creation-transaction-hash',item['deploymentTxHash'],'--watch','--retries','40','--delay','3']
                    output=run(args,cwd=work)
            else:
                output=run(args)
            result=sourcify_status(address)
            if not result or result.get('runtimeMatch') not in ['exact_match','match']:
                raise ValueError('Sourcify API did not confirm a runtime source match: '+str(result))
            verification.update(status='VERIFIED',verifier='sourcify',url=url,
                                checkedAt=datetime.now(timezone.utc).isoformat(),result=result)
            (ROOT/f'deployments/verification-{name}.txt').write_text(output+'\n')
            print('VERIFIED',name,address,flush=True)
        except Exception as error:
            failed.append(name);verification.update(status='FAILED',error=str(error))
            print('FAILED',name,str(error),flush=True)
        write_json(path,manifest)
    return bool(failed)


if __name__=='__main__':sys.exit(main())
