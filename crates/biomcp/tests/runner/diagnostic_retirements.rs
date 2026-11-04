use serde_json::json;
pub(super) fn bounded(value: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing schema")?;
    let kind = value
        .pointer("/result/kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let status = value
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing status")?;
    let recall = value.pointer("/query/source") == Some(&json!("fda-device-recall"));
    // Full compact synthetic maxima measured by j13382.
    let key = match (schema, kind, status) {
        // Native recall mechanism maxima measured by j20265.
        ("biomcp.diagnostic-page/5", "", "complete") if recall => "diagnostic-baseline-760",
        ("biomcp.diagnostic-page/5", "", "partial") if recall => "diagnostic-baseline-922",
        ("biomcp.diagnostic-page/5", "", "absent") if recall => "diagnostic-baseline-399",
        ("biomcp.diagnostic-card/4", "device-recall", "complete") => "diagnostic-baseline-924",
        ("biomcp.diagnostic-card/4", "device-recall", "partial") => "diagnostic-baseline-773",
        ("biomcp.diagnostic-card/4", "", "failed") if recall => "diagnostic-baseline-451",
        // New WHO search view, full compact maximum measured by j13465.
        ("biomcp.diagnostic-page/5", "", _)
            if value.pointer("/query/source") == Some(&json!("who-ivd")) =>
        {
            "diagnostic-baseline-2249"
        }
        ("biomcp.diagnostic-card/4", "", "absent") => "diagnostic-baseline-281",
        ("biomcp.diagnostic-card/4", "", "failed") => "diagnostic-baseline-436",
        ("biomcp.diagnostic-card/4", "", "partial") => "diagnostic-baseline-393",
        ("biomcp.diagnostic-card/4", "clearance", "complete") => "diagnostic-baseline-725",
        ("biomcp.diagnostic-card/4", "gtr-test", "complete") => "diagnostic-baseline-1519",
        ("biomcp.diagnostic-card/4", "gtr-test", "partial") => "diagnostic-baseline-1559",
        ("biomcp.diagnostic-card/4", "who-ivd-product", "complete") => "diagnostic-baseline-2578",
        ("biomcp.diagnostic-card/4", "who-ivd-product", "partial") => "diagnostic-baseline-2576",
        ("biomcp.diagnostic-companion/4", "", "complete") => "diagnostic-baseline-7879",
        ("biomcp.diagnostic-companion/4", "", "failed") => "diagnostic-baseline-410",
        ("biomcp.diagnostic-companion/4", "", "partial") => "diagnostic-baseline-7035",
        // New GTR field-refusal views measured together by j21540.
        ("biomcp.diagnostic-page/5", "", "complete")
            if value.pointer("/query/source") == Some(&json!("gtr")) =>
        {
            "diagnostic-baseline-3682"
        }
        ("biomcp.diagnostic-page/5", "", "partial")
            if value.pointer("/query/source") == Some(&json!("gtr")) =>
        {
            "diagnostic-baseline-3760"
        }
        ("biomcp.diagnostic-genes/5", "", "partial") => "diagnostic-baseline-1006",
        ("biomcp.diagnostic-conditions/5", "", "complete") => "diagnostic-baseline-625",
        ("biomcp.diagnostic-conditions/5", "", "partial") => "diagnostic-baseline-695",
        ("biomcp.diagnostic-genes/5", "", "complete") => "diagnostic-baseline-867",
        ("biomcp.diagnostic-page/5", "", "failed") => "diagnostic-baseline-379",
        ("biomcp.diagnostic-regulatory/3", "", "complete") => "diagnostic-baseline-7260",
        ("biomcp.diagnostic-supplements/3", "", "complete") => "diagnostic-baseline-1414",
        ("biomcp.error/4", "", "usage") => "diagnostic-usage",
        ("biomcp.sync/7", "", "complete") => "diagnostic-baseline-348",
        ("biomcp.sync/7", "", "failed") => "diagnostic-baseline-337",
        _ => {
            return Err(format!(
                "unmeasured diagnostic envelope: {schema}/{kind}/{status}"
            ));
        }
    };
    let baseline = super::output_limit(key)?;
    writeln!(
        std::io::stdout(),
        "diagnostic {schema}/{kind}/{status}: {bytes}"
    )
    .map_err(|e| e.to_string())?;
    if bytes > (baseline * 115).div_ceil(100) {
        return Err(format!(
            "diagnostic output exceeds measured envelope ceiling: {bytes}"
        ));
    }
    Ok(())
}
