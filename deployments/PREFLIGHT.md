# Base Sepolia preflight

Checked 2026-10-19T09:48:08.849056+00:00, block 48317500 (84532).

| Contract | Address | Code bytes |
|---|---|---|
| SafeL2 | `0x29fcB43b46531BcA003ddC8FCB67FFE91900C762` | 24421 |
| SafeProxyFactory | `0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67` | 3054 |
| CompatibilityFallbackHandler | `0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99` | 5637 |
| MultiSendCallOnly | `0x9641d764fc13c8B624c04430C7356C1C7C8102e2` | 410 |
| asset | `0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f` | 3981 |
| vaultA | `0xf430cb6E2b85f99222fBFA6dFEa18Ff60FA6B32a` | 1088 |
| faucet | `0xD9145b5F45Ad4519c7ACcD6E0A4A82e83bB8A6Dc` | 9129 |

Aave USDC + Aave waUSDC + one KavachiDemoVault. Permissionless 10,000 USDC mint succeeds in eth_call; actual mint must be confirmed before setup.

Official faucet address source: https://github.com/bgd-labs/aave-address-book/blob/main/src/MiscBaseSepolia.sol

waUSDC asset: `0xba50Cd2A20f6DA35D788639E581bca8d0B5d4D5f`; EIP-1967 implementation: `0x208747c806b13022db31622c0c9bd3979532a8d6`.

Deployer `0x77A9224AAc84d496a4ad0E865dcA9eA27B6a22Da` balance: 130000000000000000 wei. Simulation is not a mint transaction.

All four canonical Safe contracts exist; SL5 is unnecessary. Runtime code hashes and exact RPC results are in `preflight.json`.

Base Sepolia payee delay is 3,600 seconds (demo setting; production default 24 hours). Sanctions oracle is a TEST LIST; demo vault yield is SIMULATED.
