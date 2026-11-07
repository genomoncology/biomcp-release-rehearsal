//! Outside-in helpers reach each registry command through its advertised entity tool and action.
use serde_json::{Value, json};
use std::io::Write;

/// The approved tool map: (tool, action, registry command). This is the test's own copy of
/// the 1.0 tool map, so a product change that moves a command fails here.
pub(in crate::runner) const MAP: &[(&str, &str, &str)] = &[
    ("gene", "search", "search_gene"),
    ("gene", "get", "get_gene"),
    ("gene", "pgx", "search_pgx"),
    ("gene", "pgx_get", "get_pgx"),
    ("gene", "gwas", "search_gwas"),
    ("variant", "search", "search_variant"),
    ("variant", "get", "get_variant"),
    ("variant", "normalize", "variant_normalize"),
    ("variant", "assertions", "variant_assertions"),
    ("variant", "population", "variant_population"),
    ("variant", "recurrence", "variant_recurrence"),
    ("variant", "gwas", "search_gwas"),
    ("variant", "gwas_get", "get_gwas"),
    ("disease", "search", "search_disease"),
    ("disease", "get", "get_disease"),
    ("disease", "gwas", "search_gwas"),
    ("drug", "search", "search_drug"),
    ("drug", "get", "get_drug"),
    ("drug", "pgx_get", "get_pgx"),
    ("trial", "search", "search_trial"),
    ("trial", "get", "get_trial"),
    ("trial", "document", "trial_document"),
    ("trial", "organization_search", "search_organization"),
    ("trial", "biomarker_search", "search_biomarker"),
    ("trial", "biomarker_get", "get_biomarker"),
    ("article", "search", "search_article"),
    ("article", "get", "get_article"),
    ("article", "text", "article_text"),
    ("article", "pdf", "article_pdf"),
    ("article", "supplement", "article_supplement"),
    ("article", "citations", "article_citations"),
    ("article", "references", "article_references"),
    ("article", "citation_evidence", "article_citation_evidence"),
    ("article", "recommendations", "article_recommendations"),
    ("article", "author_search", "search_author"),
    ("article", "author_get", "get_author"),
    ("article", "author_papers", "author_papers"),
    ("protein", "search", "search_protein"),
    ("protein", "get", "get_protein"),
    ("protein", "features", "protein_features"),
    ("protein", "isoforms", "protein_isoforms"),
    ("protein", "domains", "protein_domains"),
    ("protein", "complexes", "protein_complexes"),
    ("protein", "complex_members", "protein_complex_members"),
    ("protein", "structures", "protein_structures"),
    ("protein", "sequence", "protein_sequence"),
    ("pathway", "search", "search_pathway"),
    ("pathway", "get", "get_pathway"),
    ("pathway", "genes", "pathway_genes"),
    ("pathway", "events", "pathway_events"),
    ("phenotype", "search", "search_phenotype"),
    ("phenotype", "get", "get_phenotype"),
    ("adverse_event", "search", "search_adverse_event"),
    ("adverse_event", "get", "get_adverse_event"),
    ("diagnostic", "search", "search_diagnostic"),
    ("diagnostic", "get", "get_diagnostic"),
    ("cell_line", "search", "search_cell_line"),
    ("cell_line", "get", "get_cell_line"),
    ("cell_line", "assays", "cell_line_assay_count"),
    ("study", "search", "search_study"),
    ("study", "get", "get_study"),
    ("study", "gwas", "search_gwas"),
    ("study", "gwas_get", "get_gwas"),
];

pub(in crate::runner) const TOOLS: [&str; 17] = [
    "gene",
    "variant",
    "disease",
    "drug",
    "trial",
    "article",
    "protein",
    "pathway",
    "phenotype",
    "adverse_event",
    "diagnostic",
    "cell_line",
    "study",
    "help",
    "biomcp",
    "search",
    "fetch",
];

/// One advertised action as a per-command view: its command name, its own schema and
/// description, and the tool and action that reach it.
fn route(tool: &Value, action: &str, command: &str) -> Option<Value> {
    let schema = tool.pointer(&format!("/inputSchema/properties/{action}"))?;
    let mut input = schema.clone();
    let fields = input.as_object_mut()?;
    let description = fields.remove("description").unwrap_or(json!(""));
    fields.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    Some(
        json!({"name":command,"description":description,"inputSchema":input,
        "annotations":tool["annotations"],"x-tool":tool["name"],"x-action":action}),
    )
}

/// Every advertised action as a per-command view, with the fixed tools unchanged.
pub(in crate::runner) fn commands(tools: &[Value]) -> Vec<Value> {
    let mut result = Vec::new();
    for tool in tools {
        let name = tool["name"].as_str().unwrap_or_default();
        let actions = tool
            .pointer("/inputSchema/properties/action/enum")
            .and_then(Value::as_array);
        let Some(actions) = actions else {
            result.push(tool.clone());
            continue;
        };
        for action in actions.iter().filter_map(Value::as_str) {
            if let Some((_, _, command)) = MAP.iter().find(|row| row.0 == name && row.1 == action)
                && !result.iter().any(|seen: &Value| seen["name"] == *command)
                && let Some(view) = route(tool, action, command)
            {
                result.push(view);
            }
        }
    }
    result
}

/// The first advertised placement of a registry command, or a fixed tool by name.
pub(in crate::runner) fn command(tools: &[Value], name: &str) -> Option<Value> {
    commands(tools)
        .into_iter()
        .find(|tool| tool["name"] == name)
        .or_else(|| tools.iter().find(|tool| tool["name"] == name).cloned())
}

/// The call that reaches a route with these command arguments.
pub(in crate::runner) fn call(tool: &Value, arguments: &Value) -> Value {
    match (tool["x-tool"].as_str(), tool["x-action"].as_str()) {
        (Some(name), Some(action)) => {
            json!({"name":name,"arguments":{"action":action,action:arguments}})
        }
        _ => json!({"name":tool["name"],"arguments":arguments}),
    }
}

/// The call that reaches a registry command at its first map placement.
pub(in crate::runner) fn entity(command: &str, arguments: Value) -> Value {
    match MAP.iter().find(|row| row.2 == command) {
        Some((tool, action, _)) => {
            json!({"name":tool,"arguments":{"action":action,*action:arguments}})
        }
        None => json!({"name":command,"arguments":arguments}),
    }
}

pub(super) fn check(tools: &[Value]) -> Result<(), String> {
    let names = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    if names != TOOLS {
        return Err(format!(
            "approved catalog is the 17 entity tools, got {names:?}"
        ));
    }
    coverage(tools)?;
    writeln!(
        std::io::stdout(),
        "TYPED_EAGER_CATALOG tools={} definition_bytes={}",
        tools.len(),
        serde_json::to_vec(tools)
            .map_err(|error| error.to_string())?
            .len()
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Every registry command sits at exactly its map placements, every placement is
/// advertised, and no schema accepts a free-form object.
pub(super) fn coverage(tools: &[Value]) -> Result<(), String> {
    for (verb, entity, _) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        if !MAP.iter().any(|row| row.2 == name) {
            return Err(format!("registry command {name} has no placement"));
        }
    }
    for (tool, action, command) in MAP {
        if !biomcp_types::command_paths()
            .iter()
            .any(|(verb, entity, _)| format!("{verb}_{entity}").replace('-', "_") == *command)
        {
            return Err(format!(
                "{tool}.{action} names no registry command {command}"
            ));
        }
    }
    coverage_schemas(tools)?;
    for tool in tools {
        let name = tool["name"].as_str().ok_or("tool name")?;
        let Some(actions) = tool
            .pointer("/inputSchema/properties/action/enum")
            .and_then(Value::as_array)
        else {
            continue;
        };
        let placed = MAP
            .iter()
            .filter(|row| row.0 == name)
            .map(|row| json!(row.1))
            .collect::<Vec<_>>();
        if actions != &placed {
            return Err(format!(
                "{name} actions {actions:?} differ from the map {placed:?}"
            ));
        }
    }
    Ok(())
}

/// Each entity tool takes a required action and one closed object per advertised action,
/// in map order, and no schema accepts a free-form object. Hosting may serve fewer actions.
pub(in crate::runner) fn coverage_schemas(tools: &[Value]) -> Result<(), String> {
    for tool in tools {
        let name = tool["name"].as_str().ok_or("tool name")?;
        free_form(&tool["inputSchema"], name)?;
        let Some(actions) = tool
            .pointer("/inputSchema/properties/action/enum")
            .and_then(Value::as_array)
        else {
            continue;
        };
        let mut placed = MAP.iter().filter(|row| row.0 == name).map(|row| row.1);
        if !actions
            .iter()
            .all(|action| placed.any(|placed| action == placed))
        {
            return Err(format!(
                "{name} actions {actions:?} are not the map's, in order"
            ));
        }
        let mut keys = tool
            .pointer("/inputSchema/properties")
            .and_then(Value::as_object)
            .ok_or("entity properties")?
            .keys()
            .filter(|key| *key != "action")
            .map(|key| json!(key))
            .collect::<Vec<_>>();
        let mut expected = actions.clone();
        keys.sort_by_key(ToString::to_string);
        expected.sort_by_key(ToString::to_string);
        if keys != expected || tool.pointer("/inputSchema/required") != Some(&json!(["action"])) {
            return Err(format!(
                "{name}: one object per action and a required action: {keys:?}"
            ));
        }
    }
    Ok(())
}

/// Fail on any object node that accepts unnamed fields.
fn free_form(node: &Value, path: &str) -> Result<(), String> {
    match node {
        Value::Object(fields) => {
            let object = fields.get("type") == Some(&json!("object"))
                || fields
                    .get("type")
                    .and_then(Value::as_array)
                    .is_some_and(|kinds| kinds.contains(&json!("object")));
            if object
                && (fields.get("additionalProperties") != Some(&json!(false))
                    || !fields.get("properties").is_some_and(Value::is_object))
            {
                return Err(format!("{path}: free-form object"));
            }
            for (key, value) in fields {
                free_form(value, &format!("{path}/{key}"))?;
            }
            Ok(())
        }
        Value::Array(values) => values
            .iter()
            .enumerate()
            .try_for_each(|(index, value)| free_form(value, &format!("{path}/{index}"))),
        _ => Ok(()),
    }
}

pub(super) fn budgets(
    env: &crate::runner::launch::Environment,
    tools: &[Value],
) -> Result<(), String> {
    let bytes = serde_json::to_vec(tools)
        .map_err(|error| error.to_string())?
        .len();
    for (config, expected) in [
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_tool_limit":tools.len(),"mcp_typed_definition_bytes":bytes}),
            None,
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_tool_limit":tools.len()-1}),
            Some("mcp_typed_tool_limit"),
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_definition_bytes":bytes-1}),
            Some("mcp_typed_definition_bytes"),
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_output_schema":true,"mcp_typed_definition_bytes":bytes}),
            Some("mcp_typed_definition_bytes"),
        ),
    ] {
        env.config(&config).map_err(|error| error.to_string())?;
        let replies = crate::runner::mcp::exchange_with_globals(
            env,
            &[crate::runner::mcp::request(
                json!(1),
                "tools/list",
                json!({}),
            )],
            &["--offline".into()],
        )?;
        let reply = replies.first().ok_or("budget reply")?;
        if let Some(field) = expected {
            if !reply
                .pointer("/error/message")
                .and_then(Value::as_str)
                .is_some_and(|message| message.contains(field))
            {
                return Err(format!("missing actionable budget error: {reply}"));
            }
        } else if reply.pointer("/result/tools") != Some(&json!(tools)) {
            return Err("exact byte/count budgets changed the catalog".into());
        }
    }
    Ok(())
}
