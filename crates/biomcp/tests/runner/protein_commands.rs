//! Feature protein-search: dynamic pool bindings and lifetimes through the shipped command.
use super::{configuration::invoke, launch::Environment};
use serde_json::Value;

pub(super) fn refused_pool_trial() -> libtest_mimic::Trial {
    super::transcript::trial("protein-search/refused-membership-is-explicit-only", || {
        refused_pool().map_err(Into::into)
    })
}

fn refused_pool() -> Result<(), String> {
    let (env, stub) = refused_fixture()?;
    let args = ["search", "protein", "Synthetic protein", "--limit", "1"]
        .map(str::to_owned)
        .to_vec();
    let first = invoke(&env, args.clone(), 0)?;
    if first.pointer("/status") != Some(&serde_json::json!("partial"))
        || first.pointer("/result/items/0/protein/accession") != Some(&serde_json::json!("P00533"))
        || first
            .get("sources")
            .and_then(Value::as_array)
            .is_none_or(|sources| {
                sources.is_empty()
                    || sources
                        .iter()
                        .any(|s| s.get("state") != Some(&serde_json::json!("ok")))
            })
        || first
            .get("problems")
            .and_then(Value::as_array)
            .is_none_or(|problems| {
                problems
                    .iter()
                    .all(|p| p.get("field") != Some(&serde_json::json!("identity")))
            })
    {
        return Err(format!(
            "refused UniProt identity lost its partial/source-Ok page: {first}"
        ));
    }
    let next = super::action(&first, "page")?;
    let second = invoke(&env, next.clone(), 0)?;
    if pool(&first)? != pool(&second)?
        || second.pointer("/status") != Some(&serde_json::json!("partial"))
        || second.pointer("/result/items/0/protein/accession") != Some(&serde_json::json!("Q8N726"))
    {
        return Err(format!("explicit protein continuation changed: {second}"));
    }
    let repeated = invoke(&env, args, 0)?;
    if pool(&first)? == pool(&repeated)? {
        return Err(format!(
            "incomplete protein search pool became the implicit default: {repeated}"
        ));
    }
    if first.pointer("/result/items") != repeated.pointer("/result/items")
        || repeated.pointer("/status") != first.pointer("/status")
    {
        return Err(format!(
            "implicit protein reconstruction changed records: {repeated}"
        ));
    }
    let kept = invoke(&env, next, 0)?;
    if pool(&second)? != pool(&kept)?
        || second.pointer("/result/items") != kept.pointer("/result/items")
        || second.pointer("/status") != kept.pointer("/status")
    {
        return Err(format!("old explicit protein page was lost: {kept}"));
    }
    stub.finish()?;
    Ok(())
}

fn pool(value: &Value) -> Result<&str, String> {
    value
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("protein window lost pool identity: {value}"))
}

fn refused_fixture() -> Result<(Environment, super::network_stub::Stub), String> {
    use serde_json::json;
    let stub = super::network_stub::Stub::with_headers(
        &["rest.uniprot.org"],
        |_| "application/json",
        |request| {
            if !request.contains("/uniprotkb/search?") {
                return Err(format!("unexpected UniProt request: {request}"));
            }
            let rows: Vec<_> = ["P00533", "Q8N726", "invalid"].into_iter().map(|id| {
                json!({"primaryAccession":id,"uniProtkbId":"SYNTHETIC_HUMAN",
                    "entryType":"UniProtKB reviewed (Swiss-Prot)","organism":{"taxonId":9606},
                    "proteinDescription":{"recommendedName":{"fullName":{"value":"Synthetic protein"}}},
                    "sequence":{"length":3,"value":"AAA"}})
            }).collect();
            Ok(json!({"results":rows}).to_string())
        },
        |_, bytes| bytes,
        |_| 200,
        |_| "x-total-results: 3\r\n".into(),
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("source-ca.pem");
    stub.retain(env.shared_root());
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok((env, stub))
}

// Measure the ordinary CLI and MCP responses against the accepted external replay.
tool_only! {
pub(super) fn native(root: &std::path::Path) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let globals = env.globals(Some(&root.join("replay")), false);
    for words in [
        vec!["get", "protein", "P00533"],
        vec!["get", "protein", "Q9UIF7"],
        vec!["get", "protein", "P38398-2"],
        vec!["get", "protein", "P42771", "Q8N726"],
        vec!["get", "protein", "BRAF"],
        vec!["get", "protein", "CDKN2A"],
        vec!["search", "protein", "--gene", "BRAF"],
        vec!["search", "protein", "--gene", "CDKN2A"],
        vec!["search", "protein", "EGFR"],
    ] {
        let args: Vec<String> = words.iter().map(|s| (*s).into()).collect();
        let cli = measure_native(&env, &globals, &args)?;
        if words == ["search", "protein", "EGFR"] {
            measure_native(&env, &globals, &super::action(&cli, "page")?)?;
        }
    }
    Ok(())
}
}
tool_only! {
fn measure_native(env: &Environment, globals: &[String], args: &[String]) -> Result<Value, String> {
    use std::io::Write;
    let mut command = globals.to_vec();
    command.extend_from_slice(args);
    let cli = invoke(env, command.clone(), 0)?;
    let mcp = super::mcp::call_with_globals(env, args, false, globals)?;
    for key in ["status", "result", "query", "page"] {
        if cli.get(key) != mcp.get(key) {
            return Err(format!("protein CLI/MCP {key} differs for {args:?}"));
        }
    }
    for (kind, value) in [("CLI", &cli), ("MCP", &mcp)] {
        let comparable = super::source_terms::size(value, "")?;
        writeln!(
            std::io::stderr(),
            "protein {kind} {args:?}: full={} comparable={comparable} ceiling={}",
            serde_json::to_vec(value).map_err(|e| e.to_string())?.len(),
            (comparable * 115).div_ceil(100)
        )
        .map_err(|e| e.to_string())?;
    }
    command.extend(["--format".into(), "md".into()]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &command)
        .output()
        .map_err(|e| e.to_string())?;
    let markdown = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let identities = cli
        .pointer("/result/items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.pointer("/header/protein/accession")
                .or_else(|| item.pointer("/protein/accession"))
                .and_then(Value::as_str)
        });
    if !output.status.success()
        || markdown.trim().is_empty()
        || identities
            .into_iter()
            .any(|id| !markdown.contains(&super::source_terms::text::escape(id)))
    {
        return Err(format!("protein Markdown failed for {args:?}"));
    }
    Ok(cli)
}
}
