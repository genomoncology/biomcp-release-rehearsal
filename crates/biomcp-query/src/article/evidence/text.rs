use super::{Snapshot, source};
use crate::{Ctx, Outcome, status};

use biomcp_adapters::pmc_oa;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    ArticleEvidence, Capped, CitationParagraphs, EvidenceParagraph, Section, Source,
};
pub(super) fn read<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEvidence,
    snapshot: &mut Snapshot,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    // The text source remains independent even if the graph leg failed.
    let Some((id, pmid)) =
        (match super::locator::resolve(cx, query.citing(), snapshot, deadline, out) {
            Ok(id) => id,
            Err(error) => {
                snapshot.fulltext = Some(Section::unavailable(
                    Source::PmcOa,
                    crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
                ));
                return Ok(());
            }
        })
    else {
        snapshot.fulltext = Some(Section::NotApplicable);
        return Ok(());
    };
    let target = match super::graph::target(cx, query, deadline, snapshot, out) {
        Ok(value) => value,
        Err(error) => {
            snapshot.fulltext = Some(Section::unavailable(
                Source::PmcOa,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            ));
            return Ok(());
        }
    };
    let leg = cx.fetch.leg(&pmc_oa::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = pmc_oa::read(&leg, id, pmid).map(|mapped| {
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        mapped.value
    });
    *deadline = crate::article::consume(*deadline, started, leg.remaining());
    source(
        snapshot,
        status::source(Source::PmcOa, leg.finish(), result.as_ref().err().copied()),
    );
    snapshot.fulltext = Some(match result {
        Err(error) => {
            out.local = out.local.or(status::local(error));
            Section::unavailable(
                Source::PmcOa,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            )
        }
        Ok(None) => Section::unavailable(Source::PmcOa, biomcp_types::Reason::NotOpen),
        Ok(Some(document)) => {
            out.dropped |= document.body.refusals == biomcp_adapters::Refusal::Refused;
            out.problems.extend(document.problems.clone());
            paragraphs(&document, &target)?
        }
    });
    Ok(())
}

fn paragraphs(
    document: &pmc_oa::Document,
    target: &super::graph::Target,
) -> Result<Section<CitationParagraphs>, RequestError> {
    let refs: Vec<_> = document
        .body
        .references
        .iter()
        .filter(|r| {
            target.pmid.is_some() && r.pmid == target.pmid
                || r.doi
                    .as_ref()
                    .zip(target.doi.as_ref())
                    .is_some_and(|(a, b)| a == b)
        })
        .map(|r| &r.id)
        .collect();
    let mut paragraphs = Vec::new();
    let mut position = 0;
    for (index, row) in document.body.rows.iter().enumerate() {
        for paragraph in &row.paragraphs {
            position += 1;
            if !paragraph.references.iter().any(|id| refs.contains(&id)) {
                continue;
            }
            let Some(part) = document
                .body
                .parts
                .iter()
                .find(|p| p.rows.iter().any(|r| r.source_row == index))
            else {
                continue;
            };
            for (offset, text) in super::page::split(&paragraph.markdown) {
                paragraphs.push(EvidenceParagraph {
                    position,
                    part: part.meta.number,
                    character_offset: offset,
                    markdown: text.into(),
                });
            }
        }
    }
    Ok(Section::Data {
        source: Source::PmcOa,
        data: CitationParagraphs {
            license: document.license,
            paragraphs: Capped {
                total: paragraphs.len() as u64,
                items: paragraphs.into(),
            },
        },
    })
}
