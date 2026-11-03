//! Feature article-fulltext: compact outline presentation retains card and part facts.
use super::launch::Environment;
use libtest_mimic::Trial;
use serde_json::Value;
use std::{io::Write, path::Path};

pub(super) fn trials() -> Vec<Trial> {
    vec![super::transcript::trial(
        "article-fulltext/compact-outline",
        || {
            for extra in [
                "",
                concat!(
                    "<sec><title>Introduction</title><p>Later introduction.</p></sec>",
                    "<sec><title>Other section</title><p>Other text.</p></sec>",
                    "<sec><title>Introduction</title><p>Final introduction.</p></sec>",
                    "<sec><title> </title><p>Untitled section.</p></sec>",
                ),
                "<sec><title>^</title><p>Literal marker title.</p></sec>",
            ] {
                check(extra).map_err(libtest_mimic::Failed::from)?;
            }
            Ok(())
        },
    )]
}

fn check(extra: &str) -> Result<(), String> {
    let env = Environment::new().map_err(|error| error.to_string())?;
    super::response_cases::copy_capture(
        "article/fulltext/synthetic",
        &env.root().join("outline-replay"),
        |_, record| {
            for exchange in record
                .get_mut("exchanges")
                .and_then(Value::as_array_mut)
                .into_iter()
                .flatten()
            {
                if let Some(value) = exchange.pointer_mut("/body/value")
                    && let Some(xml) = value.as_str().filter(|xml| xml.starts_with("<article>"))
                {
                    *value = Value::String(xml.replace("</body>", &format!("{extra}</body>")));
                }
            }
        },
    )
    .map_err(|error| error.to_string())?;
    let binary = Path::new(env!("CARGO_BIN_EXE_biomcp"));
    let json = read(&env, binary, "json", false)?;
    let reply: Value = serde_json::from_str(&json).map_err(|error| error.to_string())?;
    let markdown = read(&env, binary, "md", false)?;
    let item = reply
        .pointer("/result/items/0")
        .ok_or("missing article card")?;
    let find = serde_json::json!(["article", "text", "PMC5606870", "--find", "<text>"]);
    if !reply
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| steps.iter().any(|step| step.get("args") == Some(&find)))
    {
        return Err("outline did not offer the literal --find fill-in suggestion".into());
    }
    if markdown.matches("**title:** ").count() != 1
        || markdown.lines().any(|line| line.starts_with("# "))
    {
        return Err("article title was repeated or lost".into());
    }
    let parts = item
        .pointer("/sections/fulltext/data/parts")
        .and_then(Value::as_array)
        .ok_or("missing outline parts")?;
    if parts.len() < 2
        || !parts.windows(2).any(|pair| {
            let [left, right] = pair else {
                return false;
            };
            left.get("title").is_some() && left.get("title") == right.get("title")
        })
    {
        return Err("fixture must exercise repeated part titles".into());
    }
    outline(&markdown, parts)?;
    let disabled: Value = serde_json::from_str(&read(&env, binary, "json", true)?)
        .map_err(|error| error.to_string())?;
    if disabled
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| steps.iter().any(|step| step.get("args") == Some(&find)))
    {
        return Err("disabled fulltext source retained a find suggestion".into());
    }
    compare_baseline(&env, &reply, &markdown)
}

fn read(env: &Environment, binary: &Path, format: &str, disabled: bool) -> Result<String, String> {
    let replay = env.root().join("outline-replay");
    let mut args = ["get", "article", "28944238", "fulltext", "--format", format]
        .map(str::to_owned)
        .to_vec();
    if disabled {
        args.extend(["--disable-sources".into(), "pmc-oa".into()]);
    }
    let output = env
        .command(
            binary,
            &super::launch::arguments(&args, &env.globals(Some(&replay), false)),
        )
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "outline failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

fn outline(markdown: &str, parts: &[Value]) -> Result<(), String> {
    let mut current = "";
    let repeats = markdown.contains("^ repeats the preceding title.");
    let mut rows = Vec::new();
    for line in markdown.lines().filter(|line| line.starts_with("| ")) {
        let cells = line.split('|').skip(1).map(str::trim).collect::<Vec<_>>();
        if let [kind, number, title, tokens, ""] = cells.as_slice()
            && matches!(*kind, "section" | "table")
        {
            if *title != "^" || !repeats {
                current = title;
            }
            rows.push((
                current.to_owned(),
                (*kind).to_owned(),
                (*number).to_owned(),
                (*tokens).to_owned(),
            ));
        }
    }

    let expected = parts
        .iter()
        .map(|part| {
            (
                part.get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                part.get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                part.get("number").map(Value::to_string).unwrap_or_default(),
                part.get("tokens").map(Value::to_string).unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    if rows != expected {
        return Err(format!(
            "outline title-to-part association changed: {rows:?} != {expected:?}"
        ));
    }
    Ok(())
}

fn compare_baseline(env: &Environment, reply: &Value, markdown: &str) -> Result<(), String> {
    let Some(binary) = std::env::var_os("BIOMCP_LEGACY_EXECUTABLE") else {
        return Ok(());
    };
    let before: Value = serde_json::from_str(&read(env, Path::new(&binary), "json", false)?)
        .map_err(|error| error.to_string())?;
    for field in ["result", "query"] {
        let current = reply.get(field).ok_or("missing article result or query")?;
        if before.get(field) != Some(current) {
            return Err("compact outline changed article facts or query".into());
        }
    }
    let old = read(env, Path::new(&binary), "md", false)?;
    if old
        .lines()
        .filter(|line| line.starts_with("**title:** "))
        .collect::<Vec<_>>()
        != markdown
            .lines()
            .filter(|line| line.starts_with("**title:** "))
            .collect::<Vec<_>>()
    {
        return Err("compact outline changed the card title".into());
    }
    if markdown.len() >= old.len() {
        return Err(format!(
            "outline did not shrink: {} -> {} bytes",
            old.len(),
            markdown.len()
        ));
    }
    writeln!(
        std::io::stdout(),
        "article outline Markdown bytes: {} -> {} (saved {})",
        old.len(),
        markdown.len(),
        old.len() - markdown.len()
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
