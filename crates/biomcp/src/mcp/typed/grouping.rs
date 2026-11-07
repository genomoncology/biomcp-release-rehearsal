//! One clinical catalog policy shared by discovery and executable follow-ups.
use super::catalog::front_map::{ENTITY_TOOLS, Kind};
use serde_json::{Value, json};
use std::{collections::BTreeSet, io};

pub(in crate::mcp) struct Catalog {
    pub(super) tools: Vec<Value>,
    commands: Vec<Value>,
    globals: crate::grammar::Globals,
    read_only: bool,
    excluded: BTreeSet<biomcp_types::Source>,
    /// The entity tool being called; a next call names it when it carries the command.
    pub(super) current: Option<String>,
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

    /// The advertised actions of an entity tool.
    pub(super) fn actions(&self, tool: &str) -> Vec<String> {
        self.tools
            .iter()
            .find(|advertised| advertised["name"] == tool)
            .and_then(|advertised| advertised.pointer("/inputSchema/properties/action/enum"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(Into::into)
            .collect()
    }
}

/// One advertised registry command, before it is placed on its entity tools.
pub(super) struct Command {
    pub(super) path: (&'static str, &'static str, &'static str),
    pub(super) description: String,
    pub(super) input: Value,
    pub(super) read_only: bool,
    pub(super) sources: BTreeSet<String>,
    /// The caller schema the follow-up check reads, keyed by the command name.
    pub(super) canonical: Value,
}

pub(super) fn catalog(
    context: crate::command::Context<'_>,
    read_only: bool,
    commands: Vec<Command>,
    excluded: BTreeSet<biomcp_types::Source>,
) -> io::Result<Catalog> {
    let output_schema = crate::config::mcp_typed_schema_options(context.settings).0;
    let mut tools = ENTITY_TOOLS
        .iter()
        .filter_map(|tool| entity(tool, &commands, output_schema, &excluded).transpose())
        .collect::<io::Result<Vec<_>>>()?;
    tools.push(super::help::tool());
    tools.extend(super::super::protocol::tools(context.root, read_only)?);
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
        commands: commands
            .into_iter()
            .map(|command| command.canonical)
            .collect(),
        globals: context.globals.clone(),
        read_only,
        excluded,
        current: None,
    })
}

/// An entity tool: `action` picks one command, whose own schema is the object named after it.
fn entity(
    &(name, label, placed): &super::catalog::front_map::EntityTool,
    commands: &[Command],
    output_schema: bool,
    excluded: &BTreeSet<biomcp_types::Source>,
) -> io::Result<Option<Value>> {
    let actions = placed
        .iter()
        .filter_map(|&(action, command, kind)| {
            commands
                .iter()
                .find(|found| super::name(found.path.0, found.path.1) == command)
                .map(|found| (action, kind, found))
        })
        .collect::<Vec<_>>();
    if actions.is_empty() {
        return Ok(None);
    }
    let mut properties = serde_json::Map::new();
    properties.insert(
        "action".into(),
        json!({"type":"string","enum":actions.iter().map(|(action, _, _)| *action).collect::<Vec<_>>(),
            "description":"Which operation; its arguments go in the object of the same name."}),
    );
    for (action, _, command) in &actions {
        let mut input = command.input.clone();
        if let Some(fields) = input.as_object_mut() {
            fields.remove("$schema");
            fields.insert(
                "description".into(),
                json!(super::description::action(action, &command.description)),
            );
        }
        properties.insert((*action).into(), input);
    }
    let sources = actions
        .iter()
        .flat_map(|(_, _, command)| command.sources.iter().cloned())
        .collect::<BTreeSet<_>>();
    let names = actions
        .iter()
        .map(|(action, _, _)| *action)
        .collect::<Vec<_>>();
    let facets = actions
        .iter()
        .filter(|(_, kind, _)| *kind == Kind::Facet)
        .map(|(action, _, _)| *action)
        .collect::<Vec<_>>();
    let mut tool = json!({"name":name,
        "description":super::description::tool(label, &sources, &names, &facets, excluded),
        "inputSchema":{"$schema":super::super::protocol::SCHEMA_DIALECT,"type":"object",
            "properties":properties,"required":["action"],"additionalProperties":false},
        "annotations":{"readOnlyHint":actions.iter().all(|(_, _, command)| command.read_only),
            "destructiveHint":false,"openWorldHint":true}});
    if output_schema && let Some(fields) = tool.as_object_mut() {
        let paths = actions
            .iter()
            .map(|(_, _, command)| command.path)
            .collect::<Vec<_>>();
        fields.insert(
            "outputSchema".into(),
            super::super::output_schema::for_commands(&paths)?,
        );
    }
    Ok(Some(tool))
}

/// Write a canonical `{name: command, arguments}` call in its advertised entity form.
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
    let arguments = canonical.get("arguments")?;
    let mut placements = ENTITY_TOOLS
        .iter()
        .flat_map(|(tool, _, actions)| {
            actions
                .iter()
                .filter(|(_, placed, _)| *placed == name)
                .map(move |(action, _, _)| (*tool, *action))
        })
        .collect::<Vec<_>>();
    // Prefer the tool already in use, so a disease reply's GWAS step stays on disease.
    placements.sort_by_key(|(tool, _)| catalog.current.as_deref() != Some(*tool));
    placements.into_iter().find_map(|(tool, action)| {
        let call = json!({"name":tool,"arguments":{"action":action,action:arguments}});
        super::followups::allowed(&call, tools).then_some(call)
    })
}
