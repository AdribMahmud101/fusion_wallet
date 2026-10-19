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

Owner allocation rows are a single threshold-two Safe batch allocating 3,000 USDC to Aave waUSDC and 4,000 to the demo vault. They are separate from the agent DEPOSIT and preserve the working-cash reserve boundary.

The operational passport publishes the complete pre-decision action and domain. The verifier reconstructs its final digest using the published passport hash; this avoids embedding a digest in the bytes that the digest itself depends on. These K2 passports are explicitly operational, not production Rust planner passports.
