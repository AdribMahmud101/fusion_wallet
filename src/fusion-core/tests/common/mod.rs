// Labelled deterministic quant fixture, never real-world evidence.
pub fn measured(
    block: u64,
    time: u64,
    vault: alloy::primitives::Address,
    size: alloy::primitives::U256,
) -> fusion_core::quant::QuantInputs {
    serde_json::from_value(serde_json::json!({"blockNumber":block,"vault":vault,
        "medianGasWei":"1","quotedSizeRaw":size.to_string(),"depositSlippageRaw":"0",
        "oracle":{"feed":fusion_core::quant::ETH_USD_FEED,"decimals":8,"answer":"200000000000","roundId":"1","answeredInRound":"1","updatedAt":time},
        "reserve":{"availableRaw":"1000000000000","variableDebtRaw":"100000000","stableDebtRaw":"0","liquidityRateRay":"50000000000000000000000000","liquidityIndex":"1000000000000000000000000000"}})).unwrap()
}
