use super::Snapshot;
use crate::Outcome;
use biodata::Text;
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::{ArticleEvidence, ArticleEvidenceResult, ReplyData, Request, Section, StepKind};
/// Exact Unicode fragments leave room for both independent evidence sections and envelope.
pub(super) fn split(text: &str) -> Vec<(u32, String)> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut bytes = 0;
    let mut offset = 0;
    for (chars, ch) in text.chars().enumerate() {
        let cost = biomcp_adapters::jats::escaped_bytes(&ch.to_string());
        if bytes + cost > 3500 {
            out.push((offset, std::mem::take(&mut buffer)));
            bytes = 0;
            offset = chars as u32;
        }
        buffer.push(ch);
        bytes += cost;
    }
    if !buffer.is_empty() {
        out.push((offset, buffer))
    }
    out
}

pub(super) fn render(
    query: &ArticleEvidence,
    mut snapshot: Snapshot,
    id: Option<Text>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    out.dropped |= snapshot.dropped;
    let mut context_end = query.offset();
    let mut paragraph_end = query.paragraph_offset();
    let mut more = false;
    if let Section::Data { data, .. } = &mut snapshot.graph {
        let total = data.contexts.items.len();
        let start = query.offset() as usize;
        if start > total {
            out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
            return Ok(());
        }
        let items = page(&data.contexts.items, start, 4500, |item| {
            serde_json::to_vec(item).map(|bytes| bytes.len())
        })?;
        context_end += items.len() as u32;
        more = (context_end as usize) < total;
        data.contexts.items = items.into();
    }
    if let Some(Section::Data { data, .. }) = &mut snapshot.fulltext {
        let total = data.paragraphs.items.len();
        let start = query.paragraph_offset() as usize;
        if start > total {
            out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
            return Ok(());
        }
        let items = page(&data.paragraphs.items, start, 8500, |item| {
            serde_json::to_vec(item).map(|bytes| bytes.len())
        })?;
        paragraph_end += items.len() as u32;
        more |= (paragraph_end as usize) < total;
        data.paragraphs.items = items.into();
    }
    let unavailable = matches!(&snapshot.graph, Section::Unavailable { .. })
        || matches!(&snapshot.fulltext, Some(Section::Unavailable { .. }));
    let answered = matches!(&snapshot.graph, Section::Data { .. })
        || matches!(&snapshot.fulltext, Some(Section::Data { .. }));
    sources(out, std::mem::take(&mut snapshot.sources));
    crate::status::item(
        out,
        true,
        unavailable && !answered,
        unavailable || !snapshot.terminal,
    );
    if let Some(id) = id.filter(|_| more || !snapshot.terminal) {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &super::super::next::evidence_sources(query),
            Request::ArticleEvidence(query.page(id, context_end, paragraph_end)),
            false,
        ));
    }
    out.body = (ArticleEvidenceResult {
        semantic_scholar: snapshot.graph,
        fulltext: snapshot.fulltext,
    })
    .with_query(query.clone());
    Ok(())
}
fn page<T: Clone>(
    items: &[T],
    offset: usize,
    budget: usize,
    size: impl Fn(&T) -> Result<usize, serde_json::Error>,
) -> Result<Vec<T>, RequestError> {
    let mut out = Vec::new();
    let mut bytes = 0;
    for item in items.iter().skip(offset) {
        let size = size(item).map_err(|_| biomcp_fetch::CacheError::Database)? + 1;
        if !out.is_empty() && bytes + size > budget {
            break;
        }
        bytes += size;
        out.push(item.clone());
    }
    Ok(out)
}

fn sources(out: &mut Outcome, sources: Vec<biomcp_types::SourceOutcome>) {
    for incoming in sources {
        if let Some(original) = out.sources.iter_mut().find(|s| s.source == incoming.source) {
            crate::status::retain_source_state(original, &incoming);
        } else {
            out.sources.push(incoming)
        }
    }
}
