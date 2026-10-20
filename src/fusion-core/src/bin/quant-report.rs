//! Keyless deterministic report from a captured K44 measurement envelope.
use alloy::primitives::U256;
use fusion_core::quant::{QuantInputs, QuantReport};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: quant-report CAPTURE.json")?;
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let inputs: QuantInputs = serde_json::from_value(value["inputs"].clone())?;
    let timestamp = value["timestamp"].as_u64().ok_or("timestamp required")?;
    let size = inputs
        .quoted_size_raw
        .as_deref()
        .ok_or("quote required")?
        .parse::<U256>()?;
    let mut report = QuantReport::from_inputs(inputs.block_number, timestamp, Some(&inputs));
    report
        .candidates
        .push(report.evaluate(inputs.vault, size, 0, Some(&inputs)));
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
