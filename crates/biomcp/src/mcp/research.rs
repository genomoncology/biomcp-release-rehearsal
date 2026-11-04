//! Read-only research projections over the same parsed CLI commands.
mod records;
use serde_json::{Value, json};
use std::{ffi::OsString, io};

pub(super) fn tools(root: &crate::grammar::Root) -> io::Result<Vec<Value>> {
    [
        (
            "search",
            "query",
            "Search BioMCP source headers. Prefix a CLI entity and its arguments, or supply article query text.",
        ),
        (
            "fetch",
            "id",
            "Fetch a search result using its exact id. Returns the CLI record as Markdown with source metadata.",
        ),
    ]
    .into_iter()
    .map(|(name, key, description)| {
        let output = super::output_schema::for_tool(root, name)?;
        Ok(json!({"name":name,
"description":description,
"inputSchema":{"$schema":super::protocol::SCHEMA_DIALECT,"type":"object",
"properties":{key:{"type":"string"}},
"required":[key],
"additionalProperties":false},
"outputSchema":output,

                "annotations":{"readOnlyHint":true,"destructiveHint":false,"openWorldHint":true}}))
    })
    .collect()
}
#[derive(Clone)]
pub(super) enum Projection {
    Cli,
    Search,
    Fetch(String),
}
pub(super) struct Call {
    pub(super) args: Vec<OsString>,
    pub(super) projection: Projection,
}
pub(super) fn parse(params: &Value) -> Option<Call> {
    let name = params.get("name")?.as_str()?;
    let object = params.get("arguments")?.as_object()?;
    if object.len() != 1 {
        return None;
    }
    if name == "biomcp" {
        let args = object
            .get("args")?
            .as_array()?
            .iter()
            .map(|a| a.as_str().map(OsString::from))
            .collect::<Option<_>>()?;
        return Some(Call {
            args,
            projection: Projection::Cli,
        });
    }
    let key = match name {
        "search" => "query",
        "fetch" => "id",
        _ => return None,
    };
    let input = object.get(key)?.as_str()?;
    let (args, projection) = if name == "search" {
        (search_args(input), Projection::Search)
    } else {
        let mut args = vec!["get".into()];
        args.extend(shlex::split(input).unwrap_or_else(|| vec![input.into()]));
        (args, Projection::Fetch(input.into()))
    };
    Some(Call {
        args: args.into_iter().map(OsString::from).collect(),
        projection,
    })
}
fn search_args(input: &str) -> Vec<String> {
    let first = input.split_whitespace().next().unwrap_or_default();
    if crate::grammar::command_paths()
        .iter()
        .any(|(verb, entity, _)| *verb == "search" && *entity == first)
    {
        let mut args = vec!["search".into()];
        args.extend(shlex::split(input).unwrap_or_else(|| vec![input.into()]));
        args
    } else {
        vec!["search".into(), "article".into(), input.into()]
    }
}
pub(super) fn execute(
    call: &Call,
    context: crate::command::Context<'_>,
    http: bool,
) -> io::Result<crate::command::Outcome> {
    let mut outcome = if http {
        crate::command::tool_http(&call.args, context)?
    } else {
        crate::command::tool(&call.args, context)?
    };
    if !matches!(
        outcome.status,
        crate::exit::Status::Success | crate::exit::Status::Absent
    ) {
        return Ok(outcome);
    }
    if matches!(&call.projection, Projection::Cli) {
        return Ok(outcome);
    }
    let original = std::mem::take(outcome.value_mut()?);
    let projected = match &call.projection {
        Projection::Cli => original,
        Projection::Search => search(context.root, call, original)?,
        Projection::Fetch(id) => {
            let entity = entity(context.root, call);
            let hit = records::fetch(entity, id, &original);
            let Some(hit) = hit else {
                outcome.status = crate::exit::Status::Failed;
                *outcome.value_mut()? = original;
                return Ok(outcome);
            };
            serde_json::to_value(biomcp_types::ResearchFetch {
                id: id.clone().into(),
                title: hit.title,
                url: hit.url,
                text: biodata::Markdown::new(
                    &String::from_utf8(crate::view::markdown(&original)?)
                        .map_err(io::Error::other)?,
                )
                .map_err(io::Error::other)?,
                metadata: original,
            })?
        }
    };
    *outcome.value_mut()? = projected;
    Ok(outcome)
}
fn search(root: &crate::grammar::Root, call: &Call, envelope: Value) -> io::Result<Value> {
    let entity = entity(root, call);
    let rows = envelope.pointer("/result/items").and_then(Value::as_array);
    let results = rows
        .into_iter()
        .flatten()
        .filter_map(|row| records::search(entity, row, &envelope))
        .map(|hit| restrictions(hit, &call.args))
        .collect::<io::Result<Vec<_>>>()?;
    let native_rows = rows.or_else(|| envelope.pointer("/result/rows").and_then(Value::as_array));
    let omitted_rows = native_rows
        .map_or(
            usize::from(!envelope.get("result").is_none_or(Value::is_null)),
            Vec::len,
        )
        .saturating_sub(results.len());
    serde_json::to_value(biomcp_types::ResearchSearch {
        results,
        metadata: biomcp_types::ResearchSearchMetadata {
            envelope,
            omitted_rows: u32::try_from(omitted_rows).unwrap_or(u32::MAX),
            note:
                "Rows without an exact citable get target remain available through the biomcp tool."
                    .parse()
                    .map_err(io::Error::other)?,
        },
    })
    .map_err(Into::into)
}

fn restrictions(
    mut hit: biomcp_types::ResearchHit,
    args: &[OsString],
) -> io::Result<biomcp_types::ResearchHit> {
    let mut words =
        shlex::split(hit.id.as_str()).ok_or_else(|| io::Error::other("invalid get projection"))?;
    let mut args = args.iter().filter_map(|a| a.to_str());
    while let Some(arg) = args.next() {
        if arg == "--disable-sources" {
            if let Some(value) = args.next() {
                words.extend([arg.to_owned(), value.to_owned()]);
            }
        } else if arg.starts_with("--disable-sources=") {
            words.push(arg.to_owned());
        }
    }
    hit.id = biodata::Text::new(
        &shlex::try_join(words.iter().map(String::as_str)).map_err(io::Error::other)?,
    )
    .map_err(io::Error::other)?;
    Ok(hit)
}

fn entity(root: &crate::grammar::Root, call: &Call) -> &'static str {
    match crate::grammar::parse(root, &call.args)
        .ok()
        .and_then(|parsed| parsed.action)
    {
        Some(biomcp_query::Action::Run { request, .. }) => crate::spell::command_entity(&request),
        _ => "",
    }
}
