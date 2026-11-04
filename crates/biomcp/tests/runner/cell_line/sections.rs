//! Feature cell-line-sections: callers retrieve whole native rows independently.
use super::{
    Environment, Stub, Value, Write, budget, ceiling, fixture, invoke, json, parity, path,
    recorded_stub, setup,
};
mod controls;
pub(super) mod expected;
pub(super) fn run(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    cache(measure)?;
    controls::run(measure)?;
    for id in [
        "CVCL_2119",
        "CVCL_0064",
        "CVCL_1844",
        "CVCL_0007",
        "CVCL_0002",
        "CVCL_A794",
        "CVCL_8821",
    ] {
        let source: Value =
            serde_json::from_str(&fixture(&format!("portable-cellosaurus-cell-{id}.json"))?)
                .map_err(|e| e.to_string())?;
        let row = source
            .pointer("/Cellosaurus/cell-line-list/0")
            .ok_or("native card")?;
        for section in ["names", "identity", "xrefs", "variants"] {
            let phase = format!("sections/native/{id}/{section}");
            let value = parity(&phase, &env, &["get", "cell-line", id, section], 0, measure)?;
            if id == "CVCL_2119" {
                next(section, &value)?;
            }
            expected::page(
                row,
                section,
                &value,
                0,
                if section == "variants" { 3 } else { 5 },
            )?;
        }
        let value = parity(
            &format!("sections/full/{id}/variants"),
            &env,
            &["get", "cell-line", id, "variants", "--section-limit", "25"],
            0,
            measure,
        )?;
        expected::page(row, "variants", &value, 0, 25)?;
    }
    boundaries(&env, measure)?;
    qualifiers(&env, measure)?;
    stub.finish()?;
    Ok(())
}
fn cache(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    for section in ["names", "identity", "xrefs", "variants"] {
        let value = invoke(&env, &["get", "cell-line", "CVCL_2119", section], 0)?;
        controls::ordinary(&format!("sections/cache/{section}"), &value, measure)?;
        path(&value, "/status", json!("complete"))?;
    }
    let requests = stub.finish()?;
    if requests
        .iter()
        .filter(|(_, r)| r.contains("/cell-line/CVCL_2119?"))
        .count()
        != 1
    {
        return Err("later section pages repeated the complete request".into());
    }
    Ok(())
}
fn boundaries(env: &Environment, measure: bool) -> Result<(), String> {
    for (section, limit, offset) in [
        ("names", "1", "1"),
        ("variants", "25", "100"),
        ("identity", "1", "1"),
        ("xrefs", "1", "1"),
    ] {
        let phase = format!("sections/bounds/{section}/{limit}/{offset}");
        let value = parity(
            &phase,
            env,
            &[
                "get",
                "cell-line",
                "CVCL_2119",
                section,
                "--section-limit",
                limit,
                "--section-offset",
                offset,
            ],
            0,
            measure,
        )?;
        let source: Value =
            serde_json::from_str(&fixture("portable-cellosaurus-cell-CVCL_2119.json")?)
                .map_err(|e| e.to_string())?;
        expected::page(
            source
                .pointer("/Cellosaurus/cell-line-list/0")
                .ok_or("native")?,
            section,
            &value,
            offset.parse().map_err(|_| "offset")?,
            limit.parse().map_err(|_| "limit")?,
        )?;
    }
    for limit in ["0", "26"] {
        let value = invoke(
            env,
            &[
                "get",
                "cell-line",
                "CVCL_2119",
                "names",
                "--section-limit",
                limit,
            ],
            2,
        )?;
        path(&value, "/status", json!("usage"))?;
    }
    Ok(())
}

fn qualifiers(env: &Environment, measure: bool) -> Result<(), String> {
    for (id, section) in [
        ("CVCL_8821", "names"),
        ("CVCL_8821", "xrefs"),
        ("CVCL_0002", "names"),
    ] {
        let value = parity(
            &format!("sections/full/{id}/{section}"),
            env,
            &["get", "cell-line", id, section, "--section-limit", "25"],
            0,
            measure,
        )?;
        let native: Value =
            serde_json::from_str(&fixture(&format!("portable-cellosaurus-cell-{id}.json"))?)
                .map_err(|e| e.to_string())?;
        expected::page(
            native
                .pointer("/Cellosaurus/cell-line-list/0")
                .ok_or("native")?,
            section,
            &value,
            0,
            25,
        )?;
    }
    let value = parity(
        "sections/native/CVCL_0007/xrefs-qualified",
        env,
        &[
            "get",
            "cell-line",
            "CVCL_0007",
            "xrefs",
            "--section-offset",
            "119",
        ],
        0,
        measure,
    )?;
    path(&value, "/result/items/items/0/discontinued", json!("true"))?;
    path(
        &value,
        "/result/items/items/1/discontinued",
        json!("probable"),
    )
}
pub(super) fn markdown(
    phase: &str,
    env: &Environment,
    words: &[&str],
    replay: &[String],
    value: &Value,
    measure: bool,
) -> Result<(), String> {
    let mut args = words.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    args.extend_from_slice(replay);
    args.extend(["--format".into(), "md".into()]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("section Markdown failed".into());
    }
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    writeln!(
        std::io::stderr(),
        "CELL_LINE_MEASURE {phase} [markdown] {}",
        text.len()
    )
    .map_err(|e| e.to_string())?;
    if !measure && text.len() > ceiling(phase, "markdown")? {
        return Err(format!("{phase}: Markdown exceeds measured budget"));
    }
    strings(value.get("result").ok_or("result")?, &text)
}
fn strings(value: &Value, text: &str) -> Result<(), String> {
    match value {
        Value::String(value)
            if !text.contains(&super::super::source_terms::text::escape(value)) =>
        {
            return Err(format!("Markdown lost complete native value: {value}"));
        }
        Value::Array(values) => {
            for value in values {
                strings(value, text)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                strings(value, text)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn next(section: &str, value: &Value) -> Result<(), String> {
    if section == "variants" {
        path(
            value,
            "/next/0/args",
            json!(["get", "gene", "HGNC:7132", "HGNC:7136", "HGNC:3765"]),
        )?;
        path(
            value,
            "/next/1/args",
            json!([
                "get", "article", "15381384", "14671638", "9305600", "27750403", "12529668",
                "10233379"
            ]),
        )?;
    } else if section == "xrefs" {
        path(
            value,
            "/next/0/args",
            json!([
                "get",
                "cell-line",
                "CVCL_2119",
                "xrefs",
                "--section-offset",
                "5"
            ]),
        )?;
    }
    path(value, "/result/release/version", json!("57.0"))?;
    if value
        .pointer("/result/fetched")
        .and_then(Value::as_u64)
        .is_none()
    {
        return Err("section lost original retrieval timestamp".into());
    }
    Ok(())
}

tool_only! {
pub(super) fn measure_release() -> Result<(), String> {
    controls::release_failure(true)
}
}
