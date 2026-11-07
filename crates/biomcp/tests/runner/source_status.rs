//! Source status agrees with reasons in single and typed MCP replies.
use super::{launch::Environment, mcp, network_stub::Stub};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript::trial("diagnostic-cdx/source-status", || {
            diagnostic().map_err(Into::into)
        }),
        super::transcript::trial("variant-hotspots/source-status", || {
            hotspots().map_err(Into::into)
        }),
    ]
}

fn call(
    env: &Environment,
    mode: &str,
    name: &str,
    arguments: Value,
    args: &[&str],
    globals: &[String],
) -> Result<Value, String> {
    let (name, arguments) = if mode == "single" {
        ("biomcp", json!({"args":args}))
    } else {
        (name, arguments)
    };
    let replies = mcp::exchange_with_globals(
        env,
        &[mcp::request(
            json!(1),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(name, arguments),
        )],
        globals,
    )?;
    replies
        .first()
        .and_then(|v| v.pointer("/result/structuredContent"))
        .cloned()
        .ok_or_else(|| format!("missing structured reply: {replies:?}"))
}

fn diagnostic() -> Result<(), String> {
    let mut failures = Vec::new();
    for mode in ["single", "typed"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        env.config(&json!({"mcp_tool_mode":mode}))
            .map_err(|e| e.to_string())?;
        let replay = super::diagnostic_cdx::seed(&env)?;
        let globals = env.globals(Some(&replay), false);
        let mut sync = globals.clone();
        sync.extend(["sync".into(), "fda-cdx".into()]);
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &sync)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("companion fixture sync failed".into());
        }
        let value = call(
            &env,
            mode,
            "search_diagnostic",
            json!({"source":"fda-cdx","biomarker":"KRAS"}),
            &[
                "search",
                "diagnostic",
                "--source",
                "fda-cdx",
                "--biomarker",
                "KRAS",
            ],
            &globals,
        )?;
        let source = value.pointer("/sources/0").ok_or("source missing")?;
        if source.get("state") != Some(&json!("ok"))
            || source.get("reason").is_some()
            || value
                .pointer("/page/returned")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                == 0
            || !problem(&value, "table_1.biomarker")
        {
            failures.push(format!("{mode}: {source}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn hotspots() -> Result<(), String> {
    let mut failures = Vec::new();
    for mode in ["single", "typed"] {
        let stub = Stub::for_sources(
            &["myvariant.info", "www.cancerhotspots.org"],
            |_| "application/json",
            super::variant_hotspots::body,
        )?;
        let env = Environment::new().map_err(|e| e.to_string())?;
        let ca = env.root().join("synthetic-ca.pem");
        stub.retain(env.shared_root());
        fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
        env.config(
            &json!({"mcp_tool_mode":mode,"proxy":format!("http://{}",stub.address),"ca_bundle":ca}),
        )
        .map_err(|e| e.to_string())?;
        let value = call(
            &env,
            mode,
            "get_variant",
            json!({"input":"chr7:g.140753336A>T","build":"GRCh38","section":["hotspots"]}),
            &[
                "get",
                "variant",
                "chr7:g.140753336A>T",
                "--build",
                "GRCh38",
                "hotspots",
            ],
            &[],
        )?;
        let section = value
            .pointer("/result/candidates/0/sections/hotspots")
            .ok_or_else(|| format!("hotspots missing: {value}"))?;
        if section.get("state") != Some(&json!("data")) || section.pointer("/data/reason").is_some()
        {
            failures.push(format!("{mode}: {section}"));
        }
        if section.pointer("/data/records/0/membership") != Some(&json!("candidate")) {
            return Err("candidate join disclosure lost".into());
        }
        if !problem(&value, "transcript_consequence_join") {
            return Err("candidate join problem lost".into());
        }
        stub.finish()?;
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn problem(value: &Value, field: &str) -> bool {
    value
        .get("problems")
        .and_then(Value::as_array)
        .is_some_and(|problems| {
            problems
                .iter()
                .any(|problem| problem.get("field") == Some(&json!(field)))
        })
}
