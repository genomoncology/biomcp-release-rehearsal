//! Required publisher provenance is measured separately from unchanged payload caps.
// Test-owned Markdown contract for data cells. Product rendering is checked by callers.
pub(super) mod text {
    pub(crate) fn source_link(value: &str) -> String {
        format!("<{value}>")
    }
    pub(crate) fn escape(value: &str) -> String {
        value.chars().map(|ch| match ch {
            '&' => "&amp;".into(), '<' => "&lt;".into(), '>' => "&gt;".into(),
            ':' => "&#58;".into(), '.' => "&#46;".into(), '@' => "&#64;".into(),
            '\n' => "&#10;".into(), '\r' => "&#13;".into(),
            ch if ch.is_control()
                || matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') => {
                format!("\\u{{{:04x}}}", u32::from(ch))
            },
            ch if "\\*_`|[]#!~".contains(ch) => format!("\\{ch}"),
            ch => ch.to_string(),
        }).collect()
    }
}
use serde_json::{Value, json};

fn strip_row(row: &mut Value) -> Result<(), String> {
    let outcome: biomcp_types::SourceOutcome =
        serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
    let source = outcome.source;
    let terms = biomcp_query::sources::terms(source).ok_or("unregistered outcome source")?;
    let object = row
        .as_object_mut()
        .ok_or("source outcome is not an object")?;
    for (field, expected) in [
        ("use", json!(terms.usage)),
        ("terms", json!(terms.name)),
        ("terms_url", json!(terms.url)),
    ] {
        if object.remove(field) != Some(expected) {
            return Err(format!("{source}: missing or changed registry {field}"));
        }
    }
    Ok(())
}

/// Validate every source outcome and remove exactly its three new registry fields.
pub(super) fn comparable(value: &Value, pointer: &str) -> Result<Value, String> {
    let mut result = value.clone();
    if pointer.is_empty() {
        if let Some(rows) = result.get_mut("sources").and_then(Value::as_array_mut) {
            for row in rows {
                strip_row(row)?;
            }
        }
    } else if pointer == "/sources" {
        for row in result.as_array_mut().ok_or("sources is not an array")? {
            strip_row(row)?;
        }
    } else if pointer
        .strip_prefix("/sources/")
        .is_some_and(|p| p.parse::<usize>().is_ok())
    {
        strip_row(&mut result)?;
    }
    Ok(result)
}

pub(super) fn size(value: &Value, pointer: &str) -> Result<usize, String> {
    serde_json::to_vec(&comparable(value, pointer)?)
        .map(|b| b.len())
        .map_err(|e| e.to_string())
}

/// Measure only new publisher columns in the source table of the paired JSON view.
pub(super) fn markdown_bytes(text: &str, value: &Value) -> Result<usize, String> {
    comparable(value, "")?;
    let Some(rows) = value
        .get("sources")
        .and_then(Value::as_array)
        .filter(|r| !r.is_empty())
    else {
        return Ok(0);
    };
    let table = text
        .split_once("## Sources\n\n")
        .ok_or("missing source table")?
        .1;
    if !table.starts_with('|') {
        return markdown_object_bytes(table, rows);
    }
    let lines: Vec<_> = table
        .lines()
        .take_while(|line| line.starts_with('|'))
        .collect();
    if lines.len() != rows.len() + 2 {
        return Err("source table lost outcome rows".into());
    }
    let columns: Vec<_> = lines
        .first()
        .ok_or("missing source columns")?
        .split('|')
        .enumerate()
        .filter_map(|(i, cell)| match cell {
            " Terms " => Some((i, "terms")),
            " Terms url " => Some((i, "terms_url")),
            " Use " => Some((i, "use")),
            _ => None,
        })
        .collect();
    if columns.len() != 3
        || columns
            .iter()
            .map(|(_, field)| *field)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 3
    {
        return Err("source table lost required publisher columns".into());
    }
    let mut bytes = 0;
    for (number, line) in lines.iter().enumerate() {
        let cells: Vec<_> = line.split('|').collect();
        for (column, field) in &columns {
            let cell = cells.get(*column).ok_or("missing publisher cell")?;
            if number == 1 && *cell != " --- " {
                return Err("changed source separator".into());
            }
            if number > 1 {
                let expected = rows
                    .get(number - 2)
                    .and_then(|r| r.get(*field))
                    .and_then(Value::as_str)
                    .ok_or("missing publisher value")?;
                if *cell
                    != format!(
                        " {} ",
                        if *field == "terms_url" {
                            text::source_link(expected)
                        } else {
                            text::escape(expected)
                        }
                    )
                {
                    return Err("changed Markdown publisher terms".into());
                }
            }
            bytes += cell.len() + 1;
        }
    }
    Ok(bytes)
}

fn markdown_object_bytes(table: &str, rows: &[Value]) -> Result<usize, String> {
    let mut rest = table;
    let mut bytes = 0;
    for row in rows {
        for field in ["terms", "terms_url", "use"] {
            let value = row
                .get(field)
                .and_then(Value::as_str)
                .ok_or("missing publisher value")?;
            let rendered = if field == "terms_url" {
                text::source_link(value)
            } else {
                text::escape(value)
            };
            let line = format!("**{}:** {rendered}\n\n", text::escape(field));
            let (before, after) = rest
                .split_once(&line)
                .ok_or("changed Markdown publisher terms")?;
            if before.contains("**use:**") {
                return Err("source objects lost publisher fields".into());
            }
            bytes += line.len();
            rest = after;
        }
    }
    Ok(bytes)
}

tool_only! {
fn subtraction() -> Result<(), String> {
    let outcome = biomcp_query::sources::outcome(
        biomcp_types::Source::Hgnc,
        biomcp_types::SourceState::Ok,
        None,
    );
    let mut envelope = json!({"sources":[outcome], "result":{"value":1}});
    let cap = size(&envelope, "")?;
    let stdout = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    for (limit, amount) in [("max_bytes", cap), ("max_tokens", cap.div_ceil(4))] {
        for full in [false, true] {
            let step: super::Step = serde_json::from_value(json!({
                "run": [], "full_output_bytes": full, limit: amount,
            }))
            .map_err(|e| e.to_string())?;
            let result = super::assertions::check(&stdout, &step, &Vec::new());
            if result.is_err() != full {
                return Err(format!(
                    "{limit}: complete-output basis did not count policy fields"
                ));
            }
        }
    }
    envelope
        .get_mut("result")
        .and_then(Value::as_object_mut)
        .ok_or("missing result")?
        .insert("added".into(), json!("ordinary payload"));
    if size(&envelope, "")? <= cap {
        return Err("ordinary payload earned a terms deduction".into());
    }
    envelope
        .pointer_mut("/sources/0")
        .and_then(Value::as_object_mut)
        .ok_or("missing source")?
        .insert("terms".into(), json!("wrong terms"));
    if size(&envelope, "").is_ok() {
        return Err("malformed terms earned a deduction".into());
    }
    envelope
        .pointer_mut("/sources/0")
        .and_then(Value::as_object_mut)
        .ok_or("missing row")?
        .remove("terms");
    if size(&envelope, "").is_ok() {
        return Err("missing terms earned a deduction".into());
    }
    Ok(())
}
}

tool_only! {
fn markdown_subtraction() -> Result<(), String> {
    let outcome = biomcp_query::sources::outcome(
        biomcp_types::Source::Hgnc,
        biomcp_types::SourceState::Ok,
        None,
    );
    let text = format!(
        "## Sources\n\n| Source | State | Terms | Terms url | Use |\n| --- | --- | --- | --- | --- |\n| hgnc | ok | {} | {} | open |\n\n",
        text::escape(&outcome.terms),
        text::source_link(&outcome.terms_url)
    );
    let value = json!({"sources":[outcome]});
    let expected = "## Sources\n\n| Source | State |\n| --- | --- |\n| hgnc | ok |\n\n";
    if text.len() - markdown_bytes(&text, &value)? != expected.len() {
        return Err("Markdown changed prior table bytes".into());
    }
    let added = format!("{text}ordinary added content");
    if added.len() - markdown_bytes(&added, &value)? <= expected.len() {
        return Err("ordinary Markdown earned a deduction".into());
    }
    for changed in [
        text.replace(&text::escape("CC0 1.0"), "wrong terms"),
        text.replace(" Terms ", " Missing "),
    ] {
        if markdown_bytes(&changed, &value).is_ok() {
            return Err("invalid Markdown terms earned a deduction".into());
        }
    }
    Ok(())
}
}

tool_only! {
pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    // These call the runner's accounting directly; product output cannot prove the checker rejects bad budgets.
    vec![
        super::transcript::trial("schema-discovery/terms-subtraction-keeps-payload-ceilings", || subtraction().map_err(Into::into)),
        super::transcript::trial("schema-discovery/markdown-terms-subtraction-keeps-payload-ceilings", || markdown_subtraction().map_err(Into::into)),
    ]
}
}
