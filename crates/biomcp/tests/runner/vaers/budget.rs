//! Synthetic case budgets; native table measurements remain separate.
use super::Check;
use serde_json::{Value, json};
pub(super) fn check(value: &Value) -> Check {
    let full = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    let comparable = crate::runner::source_terms::size(value, "")?;
    // j13466 measured complete shapes; j13429 measured failures and expiry.
    // j19310 measured the deadline reply with source recovery guidance.
    let (measured_full, measured_comparable): (usize, usize) = if value.get("error").is_some() {
        (
            crate::runner::output_limit("vaers-budget-measured-224")?,
            crate::runner::output_limit("vaers-budget-measured-224")?,
        )
    } else if value.get("status") != Some(&json!("complete")) {
        if value.pointer("/query/snapshot").is_some() {
            (
                crate::runner::output_limit("vaers-budget-measured-373")?,
                crate::runner::output_limit("vaers-budget-measured-261")?,
            )
        } else {
            let measured = match value.pointer("/sources/0/reason").and_then(Value::as_str) {
                Some("rate-limited") => crate::runner::output_limit("vaers-budget-measured-298")?,
                Some("unavailable") => crate::runner::output_limit("vaers-budget-measured-297")?,
                Some("deadline-exceeded") => {
                    crate::runner::output_limit("vaers-budget-measured-551")?
                }
                Some("rejected") => crate::runner::output_limit("vaers-budget-measured-294")?,
                _ => crate::runner::output_limit("vaers-budget-measured-295")?,
            };
            (measured, measured - 112)
        }
    } else if value
        .pointer("/query/offset")
        .and_then(Value::as_u64)
        .is_some_and(|v| v > 0)
    {
        (
            crate::runner::output_limit("vaers-budget-measured-1835")?,
            crate::runner::output_limit("vaers-budget-measured-1723")?,
        )
    } else if value.pointer("/query/section") == Some(&json!("seriousness")) {
        (
            crate::runner::output_limit("vaers-budget-measured-1918")?,
            crate::runner::output_limit("vaers-budget-measured-1806")?,
        )
    } else if value.pointer("/query/section") == Some(&json!("age")) {
        (
            crate::runner::output_limit("vaers-budget-measured-1919")?,
            crate::runner::output_limit("vaers-budget-measured-1807")?,
        )
    } else {
        (
            crate::runner::output_limit("vaers-budget-measured-1993")?,
            crate::runner::output_limit("vaers-budget-measured-1881")?,
        )
    };
    if full > (measured_full * 115).div_ceil(100)
        || comparable > (measured_comparable * 115).div_ceil(100)
    {
        return Err(format!(
            "VAERS synthetic output exceeds measured ceiling: {full}/{comparable}"
        ));
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "VAERS synthetic status={} reason={} bytes={full}/{comparable}",
        value.get("status").unwrap_or(&Value::Null),
        value.pointer("/sources/0/reason").unwrap_or(&Value::Null)
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
