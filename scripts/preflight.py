#!/usr/bin/env python3
"""Probe canonical deployments and simulate Aave faucet access. Sends no transactions."""
import json
import os
from datetime import datetime, timezone
from chain import ROOT, Rpc, keccak, run, write_json

ADDRESSES = {
    'SafeL2': '0x29fcB43b46531BcA003ddC8FCB67FFE91900C762',
    'SafeProxyFactory': '0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67',
    'CompatibilityFallbackHandler': '0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99',
    'MultiSendCallOnly': '0x9641d764fc13c8B624c04430C7356C1C7C8102e2',
    'asset': '0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f',
    'vaultA': '0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a',
    'faucet': '0xD9145b5F45Ad4519c7ACcD6E0A4A82e83bB8A6Dc',
}


def main():
    url = os.environ.get('RPC_URL', 'https://sepolia.base.org')
    rpc = Rpc(url)
    assert int(rpc.call('eth_chainId', []), 16) == 84532
    block = rpc.call('eth_blockNumber', [])
    result = {'chainId': 84532, 'blockNumber': int(block, 16),
              'checkedAt': datetime.now(timezone.utc).isoformat(), 'contracts': {}}
    for name, addr in ADDRESSES.items():
        # Deliberately use the acceptance command, then pin the evidence at a block.
        code = run(['cast', 'code', addr, '--block', block, '--rpc-url', url])
        result['contracts'][name] = {'address': addr, 'codeBytes': (len(code)-2)//2,
                                      'runtimeCodeHash': keccak(code)}
        if code == '0x': raise ValueError('Missing code: ' + name + '; apply SL5 for Safe')
    deployer = os.environ['DEPLOYER_ADDRESS']
    result['deployer'] = deployer
    result['deployerBalanceWei'] = str(int(rpc.call('eth_getBalance', [deployer, block]), 16))
    result['waUSDCAsset'] = run(['cast', 'call', ADDRESSES['vaultA'], 'asset()(address)', '--rpc-url', url])
    from chain import IMPLEMENTATION_SLOT
    implementation = rpc.call('eth_getStorageAt', [ADDRESSES['vaultA'], IMPLEMENTATION_SLOT, block])
    result['waUSDCImplementation'] = '0x' + implementation[-40:]
    result['faucetSource'] = 'https://github.com/bgd-labs/aave-address-book/blob/main/src/MiscBaseSepolia.sol'
    try:
        result['mintSimulation'] = run(['cast', 'call', ADDRESSES['faucet'],
            'mint(address,address,uint256)', ADDRESSES['asset'], deployer, '10000000000',
            '--from', deployer, '--block', block, '--rpc-url', url])
        result['profile'] = 'FULL'
        result['choice'] = 'Aave USDC + Aave waUSDC + one KavachiDemoVault. Permissionless 10,000 USDC mint succeeds in eth_call; actual mint must be confirmed before setup.'
    except RuntimeError as error:
        result['profile'] = 'SCALED'
        result['choice'] = 'D2/SL2: Circle USDC + two KavachiDemoVaults; obtain Circle faucet tokens before setup.'
        result['mintSimulationError'] = str(error)
    write_json(ROOT / 'deployments/preflight.json', result)
    rows = ['# Base Sepolia preflight', '', f"Checked {result['checkedAt']}, block {result['blockNumber']} (84532).", '',
            '| Contract | Address | Code bytes |', '|---|---|---|']
    rows += [f"| {name} | `{v['address']}` | {v['codeBytes']} |" for name,v in result['contracts'].items()]
    rows += ['', result['choice'], '', 'Official faucet address source: ' + result['faucetSource'], '',
             f"waUSDC asset: `{result['waUSDCAsset']}`; EIP-1967 implementation: `{result['waUSDCImplementation']}`.", '',
             f"Deployer `{deployer}` balance: {result['deployerBalanceWei']} wei. Simulation is not a mint transaction.", '',
             'All four canonical Safe contracts exist; SL5 is unnecessary. Runtime code hashes and exact RPC results are in `preflight.json`.', '',
             'Base Sepolia payee delay is 3,600 seconds (demo setting; production default 24 hours). Sanctions oracle is a TEST LIST; demo vault yield is SIMULATED.', '']
    (ROOT / 'deployments/PREFLIGHT.md').write_text('\n'.join(rows))
    print(result['profile'], 'preflight at block', result['blockNumber'], 'deployer wei', result['deployerBalanceWei'])


if __name__ == '__main__': main()
