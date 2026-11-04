use super::relation::{self, Echo, Selection};
use crate::{Ctx, Outcome, status};
use biomcp_fetch::Deadline;
use biomcp_types::Paged;
use biomcp_types::{
    PathwayCard, PathwayGet, PathwayHeader, PathwaySection, Reason, ReplyData, Section, Source,
};
pub(super) fn attach(
    cx: &Ctx<'_>,
    q: &PathwayGet,
    card: &mut PathwayCard,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) {
    let source = if matches!(q.id(), biodata::PathwayId::Wikipathways(_)) {
        Source::WikiPathways
    } else {
        Source::Reactome
    };
    for section in [PathwaySection::Genes, PathwaySection::Events] {
        if source == Source::WikiPathways && section == PathwaySection::Events {
            continue;
        }
        if !matches!(q.section(), Some(PathwaySection::All)) && q.section() != Some(section) {
            continue;
        }
        let Some(echo) = relation_echo(q, section, source, out) else {
            continue;
        };
        let selected = Selection {
            echo,
            id: q.id(),
            genes: section == PathwaySection::Genes,
            details: false,
            limit: q.section_limit(),
            offset: q.section_offset(),
            pool: if q.section() == Some(PathwaySection::All) {
                None
            } else {
                q.pool()
            },
            fresh: cx.fresh || q.fresh(),
            header: Some(PathwayHeader {
                id: card.pathway.id.clone(),
                name: card.pathway.name.clone(),
                species: card.pathway.species,
                version: card.pathway.version.clone(),
                revision: card.pathway.revision.clone(),
            }),
        };
        let result = relation::read(cx, selected, deadline);
        let reason = result
            .sources
            .iter()
            .find_map(|s| s.reason)
            .unwrap_or(Reason::BadReply);
        match result.body {
            ReplyData::PathwayGenes { body, .. } => {
                card.genes = Some(body.map_or_else(
                    || Section::unavailable(source, reason),
                    |data| Section::Data { source, data },
                ))
            }
            ReplyData::PathwayEvents { body, .. } => {
                card.events = Some(body.map_or_else(
                    || Section::unavailable(source, reason),
                    |data| Section::Data { source, data },
                ))
            }
            ReplyData::PoolError { mismatch } => {
                out.body = ReplyData::PoolError { mismatch };
                out.next.clear();
            }
            _ => {}
        }
        for incoming in result.sources {
            if let Some(existing) = out.sources.iter_mut().find(|s| s.source == incoming.source) {
                status::retain_source_state(existing, &incoming);
            } else {
                out.sources.push(incoming);
            }
        }
        out.dropped |= result.dropped;
        out.problems.extend(result.problems);
        out.next.extend(result.next);
        out.local = out.local.or(result.local);
    }
}

fn relation_echo(
    q: &PathwayGet,
    section: PathwaySection,
    source: Source,
    out: &mut Outcome,
) -> Option<Echo> {
    let pool = if q.section() == Some(PathwaySection::All) {
        None
    } else {
        q.pool()
    };
    if section == PathwaySection::Genes {
        crate::next::checked(
            biomcp_types::PathwayGenesQuery::new(q.id().clone(), q.section_limit()),
            source,
            &mut out.problems,
        )
        .map(|query| {
            Echo::Genes(pool.map_or_else(
                || query.at_offset(q.section_offset()),
                |pool| query.at_offset(q.section_offset()).with_pool(pool.clone()),
            ))
        })
    } else {
        crate::next::checked(
            biomcp_types::PathwayEventsQuery::new(q.id().clone(), q.section_limit()),
            source,
            &mut out.problems,
        )
        .map(|query| {
            Echo::Events(pool.map_or_else(
                || query.at_offset(q.section_offset()),
                |pool| query.at_offset(q.section_offset()).with_pool(pool.clone()),
            ))
        })
    }
}
