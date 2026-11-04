//! A complete GPML relation is checked once and retained before any page is selected.
use super::{relation::Saved, relation::Selection};
use crate::{Ctx, Outcome, status};
use biodata::{PathwayId, PathwaySpecies};
use biomcp_adapters::wikipathways;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::{
    PathwayGenes, PathwayHeader, PathwayPageInfo, PathwayTotalKind, Reason, Source,
};

pub(super) fn fetch(
    cx: &Ctx<'_>,
    q: &Selection<'_>,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Saved, RequestError> {
    let PathwayId::Wikipathways(id) = q.id else {
        return Err(biomcp_fetch::InputError::Serialization.into());
    };
    let leg = cx
        .fetch
        .leg(
            &wikipathways::SPEC,
            deadline.child(std::time::Duration::from_secs(8)),
        )
        .fresh(q.fresh);
    let reply = (|| {
        // The stored human catalog admits the identity; its date is never a GPML revision.
        let card = wikipathways::card(&leg, id, false)?;
        if card.value.is_none() {
            return Err(SourceError {
                source: Source::WikiPathways,
                reason: Reason::NotFound,
                status: None,
                tls_cause: None,
                retry_after: None,
            }
            .into());
        }
        let mapped = wikipathways::members(&leg, id)?;
        let rows = mapped.value;
        let count = rows.items.len();
        Ok::<_, RequestError>(Saved {
            dropped: card.refusals == biomcp_adapters::Refusal::Refused
                || mapped.refusals == biomcp_adapters::Refusal::Refused,
            genes: Some(PathwayGenes {
                header: PathwayHeader {
                    id: q.id.clone(),
                    name: rows.name,
                    species: PathwaySpecies::Human,
                    version: None,
                    revision: Some(rows.version),
                },
                items: rows.items.into(),
                page: PathwayPageInfo {
                    offset: 0,
                    returned: count as u32,
                    total: count as u64,
                    total_kind: PathwayTotalKind::DataNodes,
                    next_offset: None,
                    pool: None,
                },
                reference_occurrences: count as u64,
                physical_entities: rows.data_nodes,
            }),
            events: None,
            sources: Vec::new(),
            problems: card.problems.into_iter().chain(mapped.problems).collect(),
        })
    })();
    out.sources.push(status::source(
        Source::WikiPathways,
        leg.finish(),
        reply.as_ref().err().copied(),
    ));
    let mut saved = reply?;
    saved.sources = out.sources.clone();
    Ok(saved)
}
