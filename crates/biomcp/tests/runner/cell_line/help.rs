use super::{Environment, Value, ceiling, fs, json};
use std::io::Write;
pub(super) fn run(measure: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    for (phase, words) in [
        ("help/search", ["help", "search", "cell-line"]),
        ("help/get", ["help", "get", "cell-line"]),
        (
            "chembl/help/assay-count",
            ["help", "cell-line", "assay-count"],
        ),
    ] {
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &words.map(String::from))
            .output()
            .map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        let retained = env.root().join(phase);
        fs::create_dir_all(&retained).map_err(|e| e.to_string())?;
        let mcp = super::super::mcp::call_retained(
            &env,
            &words.map(String::from),
            false,
            &[],
            Some(&retained),
        )?;
        let rpc = fs::read(retained.join("mcp.stdout")).map_err(|e| e.to_string())?;
        let frame: Value = serde_json::from_slice(&rpc).map_err(|e| e.to_string())?;
        let text = frame
            .pointer("/result/content/0/text")
            .and_then(Value::as_str)
            .ok_or("MCP text")?;
        writeln!(
            std::io::stderr(),
            "CELL_LINE_HELP {phase} cli={} bare={} mcp={} rpc={}",
            output.stdout.len(),
            serde_json::to_vec(value.get("result").ok_or("help result")?)
                .map_err(|e| e.to_string())?
                .len(),
            text.len(),
            rpc.len()
        )
        .map_err(|e| e.to_string())?;
        if value != mcp {
            return Err("help CLI/MCP mismatch".into());
        }
        for (surface, bytes) in [("cli", output.stdout.len()), ("mcp", text.len())] {
            super::chembl::surface::measure_bytes(phase, surface, bytes, measure)?;
        }
        markdown(&env, &words, phase, &value, measure)?;
    }
    discovery(&env)?;
    Ok(())
}

fn markdown(
    env: &Environment,
    words: &[&str; 3],
    phase: &str,
    value: &Value,
    measure: bool,
) -> Result<(), String> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &words
                .iter()
                .copied()
                .chain(["--format", "md"])
                .map(String::from)
                .collect::<Vec<_>>(),
        )
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("Markdown help failed".into());
    }
    let markdown = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let _ = writeln!(
        std::io::stderr(),
        "CELL_LINE_MEASURE {phase} [markdown] {}",
        markdown.len()
    );
    if !measure && markdown.len() > ceiling(phase, "markdown")? {
        return Err("Markdown help exceeds measured cap".into());
    }
    for argument in value
        .pointer("/result/commands/0/arguments")
        .and_then(Value::as_array)
        .ok_or("arguments")?
    {
        let name = argument
            .get("name")
            .and_then(Value::as_str)
            .ok_or("argument name")?;
        if !markdown.contains(name) {
            return Err(format!("Markdown omitted {name}"));
        }
    }
    Ok(())
}

fn discovery(env: &Environment) -> Result<(), String> {
    let replies = super::super::mcp::exchange(
        env,
        &[super::super::mcp::request(
            json!(1),
            "tools/list",
            json!({}),
        )],
    )?;
    let reply = replies.first().ok_or("tools/list reply")?;
    let description = reply
        .pointer("/result/tools/0/description")
        .and_then(Value::as_str)
        .ok_or("description")?;
    writeln!(
        std::io::stderr(),
        "CELL_LINE_DISCOVERY description={} rpc={}",
        description.len(),
        serde_json::to_vec(reply).map_err(|e| e.to_string())?.len() + 1
    )
    .map_err(|e| e.to_string())?;
    for words in [vec!["help"], vec!["schema", "biomcp.source/1"]] {
        let output = env
            .command(
                env!("CARGO_BIN_EXE_biomcp"),
                &words.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
            )
            .output()
            .map_err(|e| e.to_string())?;
        writeln!(
            std::io::stderr(),
            "CELL_LINE_DISCOVERY {:?} cli={}",
            words,
            output.stdout.len()
        )
        .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!("discovery command failed: {words:?}"));
        }
        if words.first() == Some(&"schema") {
            let schema: Value =
                serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
            writeln!(
                std::io::stderr(),
                "CELL_LINE_DISCOVERY source-enum={}",
                serde_json::to_vec(schema.get("x-extensible-enum").ok_or("source enum")?)
                    .map_err(|e| e.to_string())?
                    .len()
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
