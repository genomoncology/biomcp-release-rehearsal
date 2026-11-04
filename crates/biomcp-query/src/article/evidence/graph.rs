use super::{Snapshot, source};
use crate::{Ctx, Outcome, status};
use biodata::{Doi, Pmid};

use biomcp_adapters::{pubmed, semantic_scholar};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticleEvidence, Capped, CitationContexts, EvidenceContext, EvidenceNotice, Section, Source,
};
/// A target's independently checked native identities, never a rendered citation match.
#[derive(Clone)]
pub(super) struct Target {
    pub pmid: Option<Pmid>,
    pub doi: Option<Doi>,
}
pub(super) fn target<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEvidence,
    deadline: &mut Deadline<'a>,
    snapshot: &mut Snapshot,
    out: &mut Outcome,
) -> Result<Target, RequestError> {
    if let Some(target) = &snapshot.target {
        return Ok(target.clone());
    }
    if let ArticleInput::Doi(doi) = query.cited() {
        return Ok(Target {
            pmid: None,
            doi: Some(doi.clone()),
        });
    }
    let leg = cx.fetch.leg(&pubmed::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = (|| {
        let ids = match query.cited() {
            ArticleInput::Pmid(id) => vec![*id],
            input => pubmed::resolve_ids(&leg, input, cx.ncbi_email.as_ref())?,
        };
        if ids.len() != 1 {
            return Err(biomcp_fetch::bad_reply(Source::SemanticScholar, None));
        }
        let mapped = pubmed::summaries(&leg, &ids, cx.ncbi_email.as_ref())?;
        out.include_refusals(mapped.refusals);
        status::append(&mut out.problems, mapped.problems);
        let header = mapped
            .value
            .into_iter()
            .find(|h| h.pmid == ids.first().copied())
            .ok_or_else(|| biomcp_fetch::bad_reply(Source::SemanticScholar, None))?;
        Ok(Target {
            pmid: header.pmid,
            doi: header.doi,
        })
    })();
    *deadline = crate::article::consume(*deadline, started, leg.remaining());
    source(
        snapshot,
        status::source(Source::PubMed, leg.finish(), result.as_ref().err().copied()),
    );
    let result = match result {
        Err(error) if matches!(query.cited(), ArticleInput::Pmid(_)) => {
            out.local = out.local.or(status::local(error));
            let pmid = if let ArticleInput::Pmid(id) = query.cited() {
                Some(*id)
            } else {
                None
            };
            Ok(Target { pmid, doi: None })
        }
        result => result,
    };
    if let Ok(target) = &result {
        snapshot.target = Some(target.clone());
    }
    result
}
pub(super) fn read<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEvidence,
    snapshot: &mut Snapshot,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let target = match target(cx, query, deadline, snapshot, out) {
        Ok(value) => value,
        Err(error) => {
            snapshot.graph = Section::unavailable(
                Source::SemanticScholar,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            );
            out.local = status::local(error);
            return Ok(());
        }
    };
    let leg = cx
        .fetch
        .leg(&semantic_scholar::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let mut result = Ok(());
    for _ in 0..5 {
        match semantic_scholar::edges(&leg, query.citing(), true, snapshot.cursor) {
            Ok(reply) => {
                out.include_refusals(reply.refusals);
                out.problems.extend(reply.problems);
                for edge in reply.data {
                    out.problems.extend(edge.problems.clone());
                    if matches_target(&edge, &target) {
                        retain(snapshot, edge)?;
                    }
                }
                match reply.next {
                    None => {
                        snapshot.terminal = true;
                        snapshot.cursor = None;
                        break;
                    }
                    Some(next) if next > snapshot.cursor.unwrap_or(0) => {
                        snapshot.cursor = Some(next)
                    }
                    Some(_) => {
                        result = Err(biomcp_fetch::bad_reply(Source::SemanticScholar, None));
                        break;
                    }
                }
            }
            Err(error) => {
                result = Err(error);
                break;
            }
        }
    }
    *deadline = crate::article::consume(*deadline, started, leg.remaining());
    source(
        snapshot,
        status::source(
            Source::SemanticScholar,
            leg.finish(),
            result.as_ref().err().copied(),
        ),
    );
    if let Err(error) = result {
        out.local = status::local(error);
        if !matches!(snapshot.graph, Section::Data { .. }) {
            snapshot.graph = Section::unavailable(
                Source::SemanticScholar,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            );
        }
    }
    finish(snapshot);
    Ok(())
}
fn retain(snapshot: &mut Snapshot, edge: semantic_scholar::Edge) -> Result<(), RequestError> {
    if snapshot.matched {
        return Ok(());
    } // A provider graph edge is one citing/cited pair.
    snapshot.matched = true;
    let mut contexts = Vec::new();
    for (index, text) in edge.contexts.unwrap_or_default().into_iter().enumerate() {
        for (offset, text) in super::page::split(&text) {
            contexts.push(EvidenceContext {
                position: u32::try_from(index + 1)
                    .map_err(|_| biomcp_fetch::bad_reply(Source::SemanticScholar, None))?,
                character_offset: offset,
                text: text.into(),
            });
        }
    }
    snapshot.graph = Section::Data {
        source: Source::SemanticScholar,
        data: CitationContexts {
            contexts: Capped {
                total: contexts.len() as u64,
                items: contexts.into(),
            },
            complete: false,
            source_cursor: None,
            intents: edge.intents,
            influential: edge.is_influential,
            notice: None,
        },
    };
    Ok(())
}
fn finish(snapshot: &mut Snapshot) {
    if let Section::Data { data, .. } = &mut snapshot.graph {
        data.complete = snapshot.terminal;
        data.source_cursor = snapshot.cursor;
        data.notice = (snapshot.terminal && data.contexts.items.is_empty())
            .then_some(EvidenceNotice::NoContexts);
    } else if snapshot.terminal {
        snapshot.graph = Section::Data {
            source: Source::SemanticScholar,
            data: CitationContexts {
                contexts: Capped {
                    total: 0,
                    items: Box::default(),
                },
                complete: true,
                source_cursor: None,
                intents: None,
                influential: None,
                notice: Some(EvidenceNotice::NotInReferences),
            },
        };
    } else if !matches!(
        snapshot.graph,
        Section::Unavailable { .. } | Section::Skipped { .. }
    ) {
        snapshot.graph = Section::Data {
            source: Source::SemanticScholar,
            data: CitationContexts {
                contexts: Capped {
                    total: 0,
                    items: Box::default(),
                },
                complete: false,
                source_cursor: snapshot.cursor,
                intents: None,
                influential: None,
                notice: None,
            },
        };
    }
}

fn matches_target(edge: &semantic_scholar::Edge, target: &Target) -> bool {
    edge.cited_paper
        .as_ref()
        .and_then(|p| p.external_ids.as_ref())
        .is_some_and(|ids| {
            target.pmid.is_some() && ids.pubmed == target.pmid
                || ids
                    .doi
                    .as_ref()
                    .zip(target.doi.as_ref())
                    .is_some_and(|(a, b)| a == b)
        })
}
