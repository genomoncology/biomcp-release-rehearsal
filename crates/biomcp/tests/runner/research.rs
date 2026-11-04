//! Research wrappers map the existing CLI result without provider calls.
#[path = "research_links.rs"]
mod links;
use super::{
    launch::Environment,
    mcp::{exchange, request},
    trial_get::command,
};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-research/research-registry-projection-paths", || {
            links::check().map_err(Into::into)
        }),
        transcript_trial("mcp-research/research-search-fetch-parity", || {
            parity().map_err(Into::into)
        }),
        transcript_trial("mcp-research/research-phenotype-search-fetch", || {
            phenotype().map_err(Into::into)
        }),
    ]
}
pub(super) fn call(
    env: &Environment,
    name: &str,
    arguments: Value,
    globals: &[String],
) -> Result<Value, String> {
    call_checked(env, name, arguments, globals, None)
}
pub(super) fn call_checked(
    env: &Environment,
    name: &str,
    arguments: Value,
    globals: &[String],
    expected_error: Option<bool>,
) -> Result<Value, String> {
    let replies = super::mcp::exchange_with_globals(
        env,
        &[
            request(json!(0), "tools/list", json!({})),
            request(
                json!(1),
                "tools/call",
                json!({"name":name,"arguments":arguments}),
            ),
        ],
        globals,
    )?;
    let result = replies
        .iter()
        .find(|v| v.get("id") == Some(&json!(1)))
        .and_then(|v| v.get("result"))
        .ok_or("missing tool result")?;
    if expected_error.is_some_and(|expected| result.get("isError") != Some(&json!(expected))) {
        return Err("named MCP is_error mismatch".into());
    }
    let value = result
        .get("structuredContent")
        .ok_or("missing structured content")?;
    let text = result
        .pointer("/content/0/text")
        .and_then(Value::as_str)
        .ok_or("missing text")?;
    if serde_json::from_str::<Value>(text).map_err(|e| e.to_string())? != *value {
        return Err("research text differs from structured content".into());
    }
    {
        let tool = replies
            .iter()
            .find(|v| v.get("id") == Some(&json!(0)))
            .and_then(|v| v.pointer("/result/tools"))
            .and_then(Value::as_array)
            .and_then(|tools| tools.iter().find(|v| v.get("name") == Some(&json!(name))))
            .ok_or("tool definition missing")?;
        let schema = tool.get("outputSchema").ok_or("output schema missing")?;
        jsonschema::validator_for(schema)
            .map_err(|e| e.to_string())?
            .validate(value)
            .map_err(|e| e.to_string())?;
        if let Some(id) = value
            .get("schema")
            .or_else(|| value.pointer("/metadata/envelope/schema"))
            .or_else(|| value.pointer("/metadata/schema"))
            .and_then(Value::as_str)
            && !super::schema_proof::reply_ids(schema).contains(id)
        {
            return Err(format!("tool returned an unadvertised schema ID: {id}"));
        }
    }
    use std::io::Write;
    writeln!(
        std::io::stderr(),
        "research {name} compact_bytes={}",
        value.to_string().len()
    )
    .map_err(|e| e.to_string())?;
    Ok(value.clone())
}
fn parity() -> Result<(), String> {
    for (query, args, capture) in [
        (
            "protein --gene BRAF",
            vec!["search", "protein", "--gene", "BRAF"],
            "protein/synthetic",
        ),
        (
            "AKT1 E17K",
            vec!["search", "article", "AKT1 E17K"],
            "article/search/synthetic-card-step",
        ),
        (
            "trial --condition melanoma",
            vec!["search", "trial", "--condition", "melanoma"],
            "trial/condition",
        ),
        (
            "variant --query rs113488022 --source myvariant --build GRCh37",
            vec![
                "search",
                "variant",
                "--query",
                "rs113488022",
                "--source",
                "myvariant",
                "--build",
                "GRCh37",
            ],
            "variant/search/raw",
        ),
        (
            "variant --query rs113488022 --source myvariant --build GRCh37 --disable-sources oncokb",
            vec![
                "search",
                "variant",
                "--query",
                "rs113488022",
                "--source",
                "myvariant",
                "--build",
                "GRCh37",
                "--disable-sources",
                "oncokb",
            ],
            "variant/search/raw",
        ),
        (
            "article 'AKT1 E17K' --disable-sources oncokb",
            vec![
                "search",
                "article",
                "AKT1 E17K",
                "--disable-sources",
                "oncokb",
            ],
            "article/search/synthetic-card-step",
        ),
    ] {
        scenario(query, args, capture)?;
    }
    super::vaers::research_aggregate()?;
    unprojectable()?;
    invalid()
}
fn scenario(query: &str, args: Vec<&str>, capture: &str) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let recording = prepare(&env, capture)?;
    let globals = env.globals(Some(&recording), false);
    let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let cli = command(&env, &args, &globals, 0)?;
    if capture == "protein/synthetic" {
        let main = call(&env, "biomcp", json!({"args":args}), &globals)?;
        if main != cli {
            return Err("main tool schema result differs from CLI".into());
        }
    }
    let wrapped = call(&env, "search", json!({"query":query}), &globals)?;
    if wrapped.pointer("/metadata/envelope") != Some(&cli) {
        return Err(format!("search metadata lost CLI fields: {wrapped}"));
    }
    let rows = cli
        .pointer("/result/items")
        .or_else(|| cli.pointer("/result/candidates"))
        .and_then(Value::as_array)
        .ok_or("CLI rows missing")?;
    let search_cap = match args.get(1).map(String::as_str) {
        Some("article") => crate::runner::output_limit("research-measured-1732")?,
        Some("trial") => crate::runner::output_limit("research-measured-8192")?,
        _ => crate::runner::output_limit("research-measured-2864")?,
    };
    bounded(&wrapped, search_cap)?;
    let hits = wrapped
        .get("results")
        .and_then(Value::as_array)
        .ok_or("search results missing")?;
    if rows.len() != hits.len() || hits.is_empty() {
        return Err("research dropped source rows".into());
    }
    headers(rows, hits, args.get(1).ok_or("entity missing")?)?;
    let hit = if capture == "variant/search/raw" {
        hits.get(1)
    } else {
        hits.first()
    }
    .ok_or("hit missing")?;
    if args.get(1).is_some_and(|s| s == "variant")
        && hit.get("url")
            != Some(&json!(
                "https://myvariant.info/v1/variant/chr7%3Ag.140453136A%3ET"
            ))
    {
        return Err("variant citation does not name the exact source record".into());
    }
    let id = hit
        .get("id")
        .and_then(Value::as_str)
        .ok_or("hit id missing")?;
    if !shlex::split(id)
        .ok_or("invalid get id")?
        .iter()
        .any(|word| Some(word) == args.get(1))
        || hit
            .get("url")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        return Err("uncitable result".into());
    }
    if args.iter().any(|a| a == "--disable-sources") && !id.contains("--disable-sources oncokb") {
        return Err("get id dropped search source restrictions".into());
    }
    fetch_parity(&env, &args, hit, &recording)?;
    Ok(())
}
fn fetch_parity(
    env: &Environment,
    args: &[String],
    hit: &Value,
    recording: &std::path::Path,
) -> Result<(), String> {
    let id = hit.get("id").and_then(Value::as_str).ok_or("hit id")?;
    let leading_policy = format!("--disable-sources oncokb {id}");
    let id = if args.get(1).is_some_and(|a| a == "variant")
        && args.iter().any(|a| a == "--disable-sources")
    {
        leading_policy.as_str()
    } else {
        id
    };
    // Search and get captures are retained separately; exact ids come from the wrapper.
    let get_capture = match args.get(1).map(String::as_str) {
        Some("protein") => "protein/synthetic",
        Some("article") => "article/search/synthetic-card-step",
        Some("trial") => "trial/get/NCT02296125",
        _ => "variant/synthetic-components",
    };
    {
        super::response_cases::copy_capture(get_capture, recording, |_, _| {})
            .map_err(|e| e.to_string())?;
        let fetch_globals = env.globals(Some(recording), false);
        let mut get = vec!["get".to_owned()];
        get.extend(shlex::split(id).ok_or("invalid argument-list id")?);
        let cli_get = command(env, &get, &fetch_globals, 0)?;
        let fetched = call(env, "fetch", json!({"id":id}), &fetch_globals)?;
        let cap = match args.get(1).map(String::as_str) {
            Some("article") => crate::runner::output_limit("research-measured-6289")?,
            Some("trial") => crate::runner::output_limit("research-measured-5047")?,
            // Literal Markdown projection: j17918 measured 2459 bytes plus 15%.
            _ => crate::runner::output_limit("research-measured-2828")?,
        };
        bounded(&fetched, cap)?;
        if fetched.get("metadata") != Some(&cli_get)
            || fetched.get("id") != Some(&json!(id))
            || fetched.get("title") != hit.get("title")
            || fetched.get("url") != hit.get("url")
        {
            return Err("fetch differs from CLI".into());
        }
        let mut md = get;
        md.extend(["--format".to_owned(), "md".to_owned()]);
        md.extend(fetch_globals);
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &md)
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        if fetched.get("text") != Some(&json!(text)) {
            return Err("fetch Markdown differs".into());
        }
    }
    Ok(())
}
fn prepare(env: &Environment, capture: &str) -> Result<std::path::PathBuf, String> {
    let recording = env.root().join("research-replay");
    super::response_cases::copy_capture(capture, &recording, |_, _| {})
        .map_err(|e| e.to_string())?;
    if capture == "trial/condition" {
        // Controlled projection of a retained native study into the search fixture.
        let raw = super::trial_get::original("NCT02296125").map_err(|e| e.to_string())?;
        super::response_cases::copy_capture(capture, &recording, |_, record| {
            if let Some(first) =
                record.pointer_mut("/exchanges/0/body/value/studies/0/protocolSection")
            {
                *first = raw.clone();
            }
        })
        .map_err(|e| e.to_string())?;
        super::response_cases::copy_capture("trial/get/NCT02296125", &recording, |_, _| {})
            .map_err(|e| e.to_string())?;
    }
    Ok(recording)
}
fn invalid() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    for (id, args) in [
        ("", vec!["get"]),
        ("article nonsense", vec!["get", "article", "nonsense"]),
        ("gene", vec!["get", "gene"]),
    ] {
        let cli = command(
            &env,
            &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
            &[],
            2,
        )?;
        if call(&env, "fetch", json!({"id":id}), &[])? != cli {
            return Err("fetch usage differs from CLI".into());
        }
    }
    let replies = exchange(
        &env,
        &[request(
            json!(1),
            "tools/call",
            json!({"name":"search", "arguments":{"query":3}}),
        )],
    )?;
    if replies.first().and_then(|r| r.pointer("/error/code")) != Some(&json!(-32602)) {
        return Err("invalid input accepted".into());
    }
    Ok(())
}

fn unprojectable() -> Result<(), String> {
    for (id, capture, exit) in [
        ("article 99999999", "article/get/synthetic-derived", 1),
        ("variant rs113488022", "variant/synthetic-components", 0),
    ] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let globals = env.globals(
            Some(&super::launch::fixtures().join("replies").join(capture)),
            false,
        );
        let mut args = vec!["get".into()];
        args.extend(shlex::split(id).ok_or("test argument syntax")?);
        let cli = command(&env, &args, &globals, exit)?;
        let replies = super::mcp::exchange_with_globals(
            &env,
            &[request(
                json!(1),
                "tools/call",
                json!({"name":"fetch", "arguments":{"id":id}}),
            )],
            &globals,
        )?;
        let result = replies
            .first()
            .and_then(|v| v.get("result"))
            .ok_or("fetch result")?;
        if result.get("isError") != Some(&json!(true))
            || result.get("structuredContent") != Some(&cli)
        {
            return Err("fetch invented a record or changed an unresolved CLI result".into());
        }
    }
    Ok(())
}

// Ceilings use full compact projections measured in the focused retained-fixture run plus fifteen percent.
fn bounded(value: &Value, cap: usize) -> Result<(), String> {
    if value.to_string().len() > cap {
        return Err(format!("research output exceeds {cap} bytes"));
    }
    Ok(())
}

fn headers(rows: &[Value], hits: &[Value], entity: &str) -> Result<(), String> {
    for (row, hit) in rows.iter().zip(hits) {
        let (native_id, title, url) = match entity {
            "protein" => (
                row.pointer("/protein/accession")
                    .and_then(Value::as_str)
                    .ok_or("protein accession")?,
                row.pointer("/protein/name"),
                row.get("url")
                    .and_then(Value::as_str)
                    .ok_or("source URL")?
                    .to_owned(),
            ),
            "article" => {
                let id = row
                    .get("pmid")
                    .and_then(Value::as_str)
                    .ok_or("PMID missing")?;
                (
                    id,
                    row.pointer("/header/title"),
                    format!("https://pubmed.ncbi.nlm.nih.gov/{id}/"),
                )
            }
            "trial" => (
                row.get("nct_id").and_then(Value::as_str).ok_or("NCT id")?,
                row.get("title"),
                row.get("url")
                    .and_then(Value::as_str)
                    .ok_or("source URL")?
                    .to_owned(),
            ),
            _ => (
                row.pointer("/record/id")
                    .and_then(Value::as_str)
                    .ok_or("variant key")?,
                row.pointer("/record/id"),
                String::new(),
            ),
        };
        let args = hit
            .get("id")
            .and_then(Value::as_str)
            .and_then(shlex::split)
            .ok_or("get arguments")?;
        if !args.iter().any(|s| s == native_id)
            || hit.get("title") != title
            || (!url.is_empty() && hit.get("url") != Some(&json!(url)))
        {
            return Err("research header differs from source row".into());
        }
    }
    Ok(())
}

fn phenotype() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let globals = env.globals(
        Some(&super::launch::fixtures().join("replies/phenotype/synthetic")),
        false,
    );
    let searched = call(
        &env,
        "search",
        json!({"query":"phenotype synthetic"}),
        &globals,
    )?;
    let hit = searched
        .pointer("/results/0")
        .ok_or("phenotype hit missing")?;
    if hit.get("title") != Some(&json!("Synthetic term"))
        || hit.get("url") != Some(&json!("https://monarchinitiative.org/HP:0000001/"))
    {
        return Err(format!("phenotype search lost its label: {hit}"));
    }
    let id = hit
        .get("id")
        .and_then(Value::as_str)
        .ok_or("phenotype get id")?;
    let fetched = call(
        &env,
        "fetch",
        json!({"id":"phenotype HP:0000001"}),
        &globals,
    )?;
    let args = vec!["get".into(), "phenotype".into(), "HP:0000001".into()];
    let card = command(&env, &args, &globals, 0)?;
    if fetched.get("title") != Some(&json!("Synthetic term"))
        || fetched.get("url") != Some(&json!("https://hpo.jax.org/browse/term/HP:0000001"))
        || fetched.get("url") != card.pointer("/result/term/attribution/url")
        || fetched.get("metadata") != Some(&card)
        || !id.contains("HP:0000001")
    {
        return Err(format!(
            "phenotype fetch lost its source identity: {fetched}"
        ));
    }
    let mut args = args;
    args.extend(["--format".into(), "md".into()]);
    args.extend(globals);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    if fetched.get("text") != Some(&json!(text)) || !text.contains("Complete synthetic comment") {
        return Err("phenotype fetch lost complete card Markdown".into());
    }
    Ok(())
}
