//! One clinical catalog policy shared by discovery and executable follow-ups.
use serde_json::{Value, json};
use std::io;

pub(in crate::mcp) struct Catalog {
    pub(super) tools: Vec<Value>,
    commands: Vec<Value>,
    globals: crate::grammar::Globals,
    read_only: bool,
    excluded: std::collections::BTreeSet<biomcp_types::Source>,
}

impl Catalog {
    pub(super) fn permits(&self, parsed: &mut crate::grammar::Parsed) -> bool {
        parsed.globals.inherit(&self.globals).is_ok()
            && crate::command::mcp_policy::refusal(parsed, self.read_only).is_none()
            && parsed.action.as_ref().is_none_or(|action| match action {
                biomcp_query::Action::Run { request, .. } => matches!(
                    super::catalog::request_allowed(request, &self.excluded),
                    Ok(true)
                ),
                _ => true,
            })
    }
}

fn direct(name: &str) -> bool {
    name.split_once('_').is_some_and(|(verb, entity)| {
        matches!(verb, "search" | "get")
            && matches!(
                entity,
                "drug" | "trial" | "article" | "variant" | "disease" | "diagnostic"
            )
            || verb == "get" && entity == "gene"
            || verb == "search" && entity == "adverse_event"
    })
}

pub(super) fn catalog(
    context: crate::command::Context<'_>,
    read_only: bool,
    definitions: Vec<(Value, Value)>,
    excluded: std::collections::BTreeSet<biomcp_types::Source>,
) -> io::Result<Catalog> {
    let (commands, canonical): (Vec<_>, Vec<_>) = definitions.into_iter().unzip();
    let (mut tools, grouped): (Vec<_>, Vec<_>) = commands
        .iter()
        .cloned()
        .partition(|tool| tool["name"].as_str().is_some_and(direct));
    let shared = super::super::protocol::tools(context.root, read_only)?;
    let mut reference = reference(&grouped);
    if commands
        .iter()
        .any(|tool| tool.get("outputSchema").is_some())
    {
        let output = shared
            .iter()
            .find(|tool| tool["name"] == "biomcp")
            .and_then(|tool| tool.get("outputSchema"))
            .cloned()
            .ok_or_else(|| io::Error::other("missing reference output schema"))?;
        reference
            .as_object_mut()
            .ok_or_else(|| io::Error::other("missing reference tool"))?
            .insert("outputSchema".into(), output);
    }
    tools.push(reference);
    tools.push(super::help::tool());
    tools.extend(shared);
    let (tool_limit, byte_limit) = crate::config::mcp_typed_budget(context.settings);
    let bytes = serde_json::to_vec(&tools)?.len();
    if tools.len() > tool_limit || bytes > byte_limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Typed catalog requires {} tools and {bytes} definition bytes; \
                mcp_typed_tool_limit={tool_limit}, mcp_typed_definition_bytes={byte_limit}. \
                Clinical tools and complete descriptions are retained; configure sufficient budgets.",
                tools.len(),
            ),
        ));
    }
    Ok(Catalog {
        tools,
        commands: canonical,
        globals: context.globals.clone(),
        read_only,
        excluded,
    })
}

fn reference(commands: &[Value]) -> Value {
    let names = commands
        .iter()
        .map(|tool| tool["name"].clone())
        .collect::<Vec<_>>();
    let read_only = commands
        .iter()
        .all(|tool| tool.pointer("/annotations/readOnlyHint") == Some(&json!(true)));
    json!({"name":"biomcp_reference",
        "description":"Run specialist reference commands and additional entity operations with their typed arguments. \
        IDs and options follow the selected command's schema. \
        Example: {\"command\":\"get_protein\",\"arguments\":{\"id\":\"P04637\"}}.",
        "inputSchema":{"$schema":super::super::protocol::SCHEMA_DIALECT,"type":"object",
            "properties":{
                "command":{"type":"string","enum":names,"description":"Registered typed command name, such as get_protein."},
                "arguments":{"type":"object","additionalProperties":true,
                    "description":"The selected command's typed arguments; example for get_protein: {\"id\":\"P04637\"}."}},
            "required":["command","arguments"],"additionalProperties":false},
        "annotations":{"readOnlyHint":read_only,"destructiveHint":false,"openWorldHint":true}})
}

// Unwrap without introducing another validator or argument-to-CLI converter.
pub(super) fn unwrap(params: &Value) -> Option<Value> {
    let fields = params.get("arguments")?.as_object()?;
    if fields.len() != 2 || !fields.get("arguments")?.is_object() {
        return None;
    }
    Some(json!({"name":fields.get("command")?.as_str()?,"arguments":fields.get("arguments")?}))
}

pub(super) fn call(canonical: Value, catalog: &Catalog) -> Option<Value> {
    let tools = &catalog.tools;
    let name = canonical.get("name")?.as_str()?;
    let command = catalog.commands.iter().find(|tool| tool["name"] == name);
    if command
        .is_some_and(|tool| !super::followups::allowed(&canonical, std::slice::from_ref(tool)))
    {
        return None;
    }
    if tools.iter().any(|tool| tool["name"] == name) {
        return super::followups::allowed(&canonical, tools).then_some(canonical);
    }
    command?;
    let reference = tools
        .iter()
        .find(|tool| tool["name"] == "biomcp_reference")?;
    if !reference
        .pointer("/inputSchema/properties/command/enum")?
        .as_array()?
        .contains(&json!(name))
    {
        return None;
    }
    let call = json!({"name":"biomcp_reference","arguments":{"command":name,"arguments":canonical.get("arguments")?}});
    super::followups::allowed(&call, tools).then_some(call)
}
