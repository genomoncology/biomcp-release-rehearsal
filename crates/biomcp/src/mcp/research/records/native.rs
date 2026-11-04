//! Source-owned links and labels for native records.
use super::{hit, next_args, paths, text};
use biomcp_query::sources::{declaration, record_url};
use biomcp_types::{ResearchHit, Source};
use serde_json::Value;

fn source(row: &Value, envelope: &Value) -> Option<Source> {
    text(
        row,
        &[
            "/source",
            "/id/namespace",
            "/pathway/id/namespace",
            "/attribution/source",
            "/term/source",
        ],
    )
    .or_else(|| text(envelope, &["/query/source"]))
    .or_else(|| {
        let sources = envelope.get("sources")?.as_array()?;
        (sources.len() == 1)
            .then(|| sources.first())
            .flatten()
            .and_then(|s| text(s, &["/source"]))
    })?
    .parse()
    .ok()
}
fn identity(row: &Value) -> Option<String> {
    text(row, paths::IDS).map(str::to_owned).or_else(|| {
        paths::IDS
            .iter()
            .find_map(|path| row.pointer(path)?.as_u64().map(|id| id.to_string()))
    })
}
pub(super) fn project(
    entity: &str,
    row: &Value,
    envelope: &Value,
    args: Vec<String>,
    card: Option<&Value>,
) -> Option<ResearchHit> {
    let identity = identity(row)?;
    let id = if entity == "author" {
        identity
            .strip_prefix("semantic-scholar:")
            .unwrap_or(&identity)
    } else {
        &identity
    };
    let source = source(row, envelope);
    let declared = source.zip(paths::kind(entity));
    let template = declared.and_then(|(source, kind)| declaration(source, kind));
    let url = text(row, paths::URLS)
        .or_else(|| card.and_then(|v| text(v, &["/report_url", "/source_url"])))
        .map(str::to_owned)
        .or_else(|| {
            let (source, kind) = declared?;
            record_url(source, kind, id)
        })?;
    let fallback;
    let title = if let Some(title) = text(row, paths::TITLES) {
        title
    } else {
        let label = template
            .map(|record| record.display_name)
            .or_else(|| source.map(biomcp_query::sources::display_name))?;
        fallback = format!("{label} {id}");
        &fallback
    };
    hit(args, title, &url)
}
pub(super) fn search(entity: &str, row: &Value, envelope: &Value) -> Option<ResearchHit> {
    let nci = (entity == "trial" && source(row, envelope) == Some(Source::Nci))
        .then(|| {
            Some(vec![
                entity.into(),
                text(row, &["/nct_id"])?.into(),
                "--source".into(),
                "nci".into(),
            ])
        })
        .flatten();
    let args = nci.or_else(|| {
        paths::IDS
            .iter()
            .filter_map(|path| row.pointer(path).and_then(Value::as_str))
            .find_map(|id| next_args(entity, id, envelope))
            .or_else(|| next_args(entity, &identity(row)?, envelope))
    })?;
    project(entity, row, envelope, args, None)
}

pub(super) fn fetch(
    entity: &str,
    result: &Value,
    first: Option<&Value>,
    envelope: &Value,
) -> Option<ResearchHit> {
    let row = first
        .and_then(|v| v.get("header"))
        .or_else(|| result.get("header"))
        .unwrap_or(result);
    project(entity, row, envelope, vec![entity.into()], Some(result))
}
