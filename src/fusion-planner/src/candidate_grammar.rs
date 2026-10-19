pub const PREFIX: &str = "{\"choice\":\"";
pub const SUFFIX: &str = "\"}";
pub fn legal(labels: &[String]) -> bool {
    !labels.is_empty()
        && labels.len() <= 8
        && labels.iter().any(|l| l == "H")
        && labels.iter().enumerate().all(|(i, l)| {
            l.len() == 1 && matches!(l.as_bytes()[0], b'A'..=b'H') && !labels[..i].contains(l)
        })
}
pub fn validate(raw: &str, labels: &[String]) -> Option<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        choice: String,
    }
    let v: Output = serde_json::from_str(raw).ok()?;
    labels.iter().find(|x| x.as_str() == v.choice).cloned()
}
