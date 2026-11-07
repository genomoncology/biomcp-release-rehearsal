//! Feature maude-reports: synthetic output ceilings retain native facts and component labels.
use serde_json::{Value, json};
pub(super) fn check(value: &Value) -> Result<(), String> {
    let full = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    let mut comparable = value.clone();
    if let Some(outcomes) = comparable.get_mut("sources").and_then(Value::as_array_mut) {
        for row in outcomes {
            let fields = row.as_object_mut().ok_or("source outcome")?;
            let source: biomcp_types::Source =
                serde_json::from_value(fields.get("source").ok_or("source identity")?.clone())
                    .map_err(|e| e.to_string())?;
            let terms = biomcp_query::sources::terms(source).ok_or("source declaration")?;
            if fields.get("use") != Some(&json!(terms.usage))
                || fields.get("terms") != Some(&json!(terms.name))
                || fields.get("terms_url") != Some(&json!(terms.url))
            {
                return Err("source terms changed before budget comparison".into());
            }
            for key in ["use", "terms", "terms_url"] {
                fields.remove(key).ok_or("source terms field")?;
            }
        }
    }
    let bytes = serde_json::to_vec(&comparable)
        .map_err(|e| e.to_string())?
        .len();
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or("schema identity")?;
    let caps = ceiling(value, schema).ok_or("unbudgeted synthetic MAUDE schema")?;
    if full > caps.0 || bytes > caps.1 {
        return Err(format!(
            "synthetic MAUDE budget exceeded for {schema}: {full}/{bytes} > {caps:?}"
        ));
    }
    Ok(())
}

// j13034 observed each class maximum; ceilings add 15 percent and round up.
fn ceiling(value: &Value, schema: &str) -> Option<(usize, usize)> {
    if value.get("error").is_some() {
        return Some(if value.get("next").is_some() {
            (
                crate::runner::output_limit("maude-envelope-250").ok()?,
                crate::runner::output_limit("maude-envelope-250").ok()?,
            )
        } else {
            // j19028: typed argument names, 218 compact bytes plus 15 percent.
            (
                crate::runner::output_limit("maude-envelope-251").ok()?,
                crate::runner::output_limit("maude-envelope-251").ok()?,
            )
        });
    }
    if value.get("status") == Some(&json!("failed")) {
        return match schema {
            "biomcp.adverse-event-page/6" => Some((
                crate::runner::output_limit("maude-envelope-364").ok()?,
                crate::runner::output_limit("maude-envelope-232").ok()?,
            )),
            "biomcp.adverse-event-maude-card/4" => Some((
                crate::runner::output_limit("maude-envelope-472").ok()?,
                crate::runner::output_limit("maude-envelope-340").ok()?,
            )),
            "biomcp.adverse-event-devices/4" => Some((
                crate::runner::output_limit("maude-envelope-481").ok()?,
                crate::runner::output_limit("maude-envelope-349").ok()?,
            )),
            "biomcp.adverse-event-narrative/4" => Some((
                crate::runner::output_limit("maude-envelope-500").ok()?,
                crate::runner::output_limit("maude-envelope-367").ok()?,
            )),
            _ => None,
        };
    }
    if value.get("status") == Some(&json!("absent")) {
        return Some((
            crate::runner::output_limit("maude-envelope-733").ok()?,
            crate::runner::output_limit("maude-envelope-601").ok()?,
        ));
    }
    match schema {
        "biomcp.adverse-event-page/6" => Some((
            crate::runner::output_limit("maude-envelope-948").ok()?,
            crate::runner::output_limit("maude-envelope-816").ok()?,
        )),
        "biomcp.adverse-event-maude-card/4" => Some((
            crate::runner::output_limit("maude-envelope-1497").ok()?,
            crate::runner::output_limit("maude-envelope-1364").ok()?,
        )),
        "biomcp.adverse-event-devices/4" => Some((
            crate::runner::output_limit("maude-envelope-1495").ok()?,
            crate::runner::output_limit("maude-envelope-1355").ok()?,
        )),
        "biomcp.adverse-event-narrative/4" => Some((
            crate::runner::output_limit("maude-envelope-1147").ok()?,
            crate::runner::output_limit("maude-envelope-1007").ok()?,
        )),
        _ => None,
    }
}
