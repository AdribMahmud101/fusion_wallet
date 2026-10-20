# K47B live Autopilot decision

The live app path ran once with `dryRun: false` at Base Sepolia block 48,366,421. The on-device MNN + AHEM-G planner selected **HOLD**. The report has no `allocationMandate`, `execution`, or transaction hash.

The recorded reason is that the candidate set was empty: `candidateSet.candidates` and `vaultQuotes` are empty, and the report has no K44 `quant` section. The report contains only the generic `HOLD` reason, so it does not expose a more specific candidate rejection. The fresh 75 USDC K44 quote from block 48,366,111 was independently ACTIONABLE at 11.69 days, but the live app tick did not produce a deposit candidate. No action was forced and no APP transaction was sent. Funds remain in Safe cash.

Evidence: `K47B_autopilot_hold.json`, `K47B_75_live_inputs.json`, `K47B_75_quant_report.json`, and `K47B_live_app.png`.
