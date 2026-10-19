# Base Sepolia transaction evidence

All rows in `base-sepolia.json` are real chain 84532 transactions. K2 operational actions are labelled `SCRIPT / deterministic / linux-host`; they are not Android or model executions.

Safe: [`0x029869910714D9e09b854a2688311AFd42fBb44E`](https://sepolia.basescan.org/address/0x029869910714D9e09b854a2688311AFd42fBb44E).
Module: [`0xA560ec10259657E9bADd297E68286d2838Cd5a8f`](https://sepolia.basescan.org/address/0xA560ec10259657E9bADd297E68286d2838Cd5a8f#code).

Project deployments have exact Sourcify source matches. Addresses, code hashes, compiler settings and verification links are in [`deployments/84532.json`](../deployments/84532.json).

```bash
bash scripts/audit_secrets.sh
python3 scripts/verify_evidence.py evidence/base-sepolia.json
```

The verifier uses public read-only RPC methods and `cast keccak`; no signing keys or Python dependencies are required. Trace access is optional: an HTTP 401 or unavailable trace API triggers the historical `eth_call` fallback for deliberate failures. Receipt collection waits for a nonzero canonical block hash rather than persisting Flashblocks preconfirmations.

`anvil.json` and `obligations-31337.json` are separate local rehearsal records. Their hashes are not Base Sepolia transactions. Operational passports are stored under `passports/`.

Payee delay: 3,600 seconds (demo setting; production default 24 hours). Sanctions list: TEST LIST. Demo vault yield: SIMULATED.

## Current rows

| ID | Kind | Transaction |
|---|---|---|
| E1 | SAFE_CREATION | [0x957ef32ef5e4244b6c76ec20c6aa857b2ca57d2402ed1176eecf4bafcccb9e7b](https://sepolia.basescan.org/tx/0x957ef32ef5e4244b6c76ec20c6aa857b2ca57d2402ed1176eecf4bafcccb9e7b) |
| E2 | MODULE_ENABLE | [0xd8d145e6795471b8fcc68c98c3b1e3763e5e95cbb383140fbcc04c256065e299](https://sepolia.basescan.org/tx/0xd8d145e6795471b8fcc68c98c3b1e3763e5e95cbb383140fbcc04c256065e299) |
| E3 | POLICY_CONFIGURED | [0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) |
| E5 | OBLIGATIONS_COMMITTED | [0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) |
| E6 | SESSION_STARTED | [0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) |
| E6 | RESUMED | [0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa](https://sepolia.basescan.org/tx/0xb8927af265a7d73d84bbefcafe512698bd6547a3cbbef64a6baa1c49659a27fa) |
| E4 | PAYEE_ADDED | [0x8c2a208fed80010e4dfdeaf227f344d80d4411ee1669389abfaabd9d1a4e6ad0](https://sepolia.basescan.org/tx/0x8c2a208fed80010e4dfdeaf227f344d80d4411ee1669389abfaabd9d1a4e6ad0) |
| E4 | PAYEE_ADDED | [0xabb5bd793393a221a72d773a8b65708d65ae7cc0be1a3064fe98ed088adf4f94](https://sepolia.basescan.org/tx/0xabb5bd793393a221a72d773a8b65708d65ae7cc0be1a3064fe98ed088adf4f94) |
| E9 | ACTION_APPROVED | [0x4fe8a794c15a91bf70e841549b2bf1907290100ee46f6a1abfa98cdc1221b603](https://sepolia.basescan.org/tx/0x4fe8a794c15a91bf70e841549b2bf1907290100ee46f6a1abfa98cdc1221b603) |
| E10 | DEPOSIT | [0x35ff759c17bc48442a1d25b02a75fb66aac2307502cd0402e36b40916d65d726](https://sepolia.basescan.org/tx/0x35ff759c17bc48442a1d25b02a75fb66aac2307502cd0402e36b40916d65d726) |
| E16 | INTENTIONAL_REVERT | [0x8f9dde942e571fad7b19c7e1f7a75564a80d18a893731539438a817100d30a70](https://sepolia.basescan.org/tx/0x8f9dde942e571fad7b19c7e1f7a75564a80d18a893731539438a817100d30a70) |
| E16 | INTENTIONAL_REVERT | [0xe1d5dba152fb9a183192794e37bd17c98fe5530e1fcb937970ef2739904baa17](https://sepolia.basescan.org/tx/0xe1d5dba152fb9a183192794e37bd17c98fe5530e1fcb937970ef2739904baa17) |
| FUNDING | OWNER_ALLOCATION | [0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba](https://sepolia.basescan.org/tx/0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba) |
| FUNDING | OWNER_ALLOCATION | [0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba](https://sepolia.basescan.org/tx/0xfd7cd40d688fddbaa2dbf68d9dabbbfd65c163934d23ac2a88cbcdf6d293dbba) |
| E16 | INTENTIONAL_REVERT | [0x55d44a2b3f7dedab3d898528a14cdd66aefaa49cf1eaa658b84dd3b777505d27](https://sepolia.basescan.org/tx/0x55d44a2b3f7dedab3d898528a14cdd66aefaa49cf1eaa658b84dd3b777505d27) |
| E7 | PAY_INSTANT | [0x3d5682afb067993bf04ae88a344eff6d642138062f56e8539d79ed975cffe6c7](https://sepolia.basescan.org/tx/0x3d5682afb067993bf04ae88a344eff6d642138062f56e8539d79ed975cffe6c7) |
| E8 | ACTION_QUEUED | [0x01316eaeebf051827e46b0fa61645376ea531342cb6d526e4252acc6b0db202a](https://sepolia.basescan.org/tx/0x01316eaeebf051827e46b0fa61645376ea531342cb6d526e4252acc6b0db202a) |
| E8 | PAY_DELAYED | [0xc63f9d98e7a2d28ebe88eea2ffcec9288a39c4be0aa218da1acb584670b1fe7f](https://sepolia.basescan.org/tx/0xc63f9d98e7a2d28ebe88eea2ffcec9288a39c4be0aa218da1acb584670b1fe7f) |
| E8 | ACTION_QUEUED | [0xdac5920996832c73375b584905d34cf430f35ad548ee1368a7695c927736548c](https://sepolia.basescan.org/tx/0xdac5920996832c73375b584905d34cf430f35ad548ee1368a7695c927736548c) |
| E12 | QUEUE_CANCELLED | [0x7e069c86c94e06fda563190567876b467f0b2facf9189ebb1a0939253182c4b5](https://sepolia.basescan.org/tx/0x7e069c86c94e06fda563190567876b467f0b2facf9189ebb1a0939253182c4b5) |
| E16 | INTENTIONAL_REVERT | [0x5af192e04e2ac730bd3eebc0a4f7f76d9ee76525896d40b0c9084f84d98e96b4](https://sepolia.basescan.org/tx/0x5af192e04e2ac730bd3eebc0a4f7f76d9ee76525896d40b0c9084f84d98e96b4) |
| E9 | ACTION_APPROVED | [0xddbc47e9037a49a0db323136d79360a9bd89f7e8220fb32f1ed987e4ef2601e0](https://sepolia.basescan.org/tx/0xddbc47e9037a49a0db323136d79360a9bd89f7e8220fb32f1ed987e4ef2601e0) |
| E9 | PAY_OWNER_TIER | [0x97e5cd9c1c5bc351b32d2819e6735ef161fda91d3ab6285ccd4a06aec3ba232d](https://sepolia.basescan.org/tx/0x97e5cd9c1c5bc351b32d2819e6735ef161fda91d3ab6285ccd4a06aec3ba232d) |
| E13 | OWNER_PAY | [0x46470601cddedd6214792c494472affad03e5457e246bb7b68e35f38cf422826](https://sepolia.basescan.org/tx/0x46470601cddedd6214792c494472affad03e5457e246bb7b68e35f38cf422826) |
| E4 | PAYEE_NEW_BANK | [0x8c7216a9f5b69f9e7cdde1bc7085051510fda7a05f47c90574acaa82751597b3](https://sepolia.basescan.org/tx/0x8c7216a9f5b69f9e7cdde1bc7085051510fda7a05f47c90574acaa82751597b3) |
| E16 | INTENTIONAL_REVERT | [0xc04b77c2ffe8d0d945c1cc13aa70d3e34827a85b8af3629b2a2575e30f775d02](https://sepolia.basescan.org/tx/0xc04b77c2ffe8d0d945c1cc13aa70d3e34827a85b8af3629b2a2575e30f775d02) |
| E11 | WITHDRAW | [0x06530794bc195da12a84ee3ae4538ed1cca3b91f727f2ba4eaeb87f76da67950](https://sepolia.basescan.org/tx/0x06530794bc195da12a84ee3ae4538ed1cca3b91f727f2ba4eaeb87f76da67950) |
| E14 | GUARDIAN_PAUSE | [0xe33f6e100a88d1008bcb226302ec2e90c147929b1ae71167c6ba134059d7c61c](https://sepolia.basescan.org/tx/0xe33f6e100a88d1008bcb226302ec2e90c147929b1ae71167c6ba134059d7c61c) |
| E6 | OWNER_RESUME | [0x4e097ab5cf673de69cd1e99283424b4935a9cc7ac536e4746b2889057bf43cf7](https://sepolia.basescan.org/tx/0x4e097ab5cf673de69cd1e99283424b4935a9cc7ac536e4746b2889057bf43cf7) |
| E15 | EMERGENCY_UNWIND | [0xbcbc2d8a0e6e460d2a19319f59a9a05ca4685ac5b284c105e75ef15e58518759](https://sepolia.basescan.org/tx/0xbcbc2d8a0e6e460d2a19319f59a9a05ca4685ac5b284c105e75ef15e58518759) |

All E1–E16 are recorded: 30 rows across 26 unique transactions. The final emergency unwind recovered both vault positions; the module is paused and its session is cleared.

Owner allocation rows are a threshold-two Safe batch allocating 3,000 USDC to Aave waUSDC and 4,000 to the demo vault, separate from the agent DEPOSIT.

The operational passport publishes the complete pre-decision action and domain. The verifier reconstructs the final digest using the passport hash, avoiding a circular hash. These K2 passports are explicitly operational, not production Rust planner passports.

Integration handoff: [`deployments/HANDOFF.md`](../deployments/HANDOFF.md).
