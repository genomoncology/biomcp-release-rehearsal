//! Complete entry-backed pages bind the section and every selector to one snapshot.
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::{Text, UniProtIsoform};
use biomcp_adapters::uniprot;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, ProteinFeatures, ProteinFeaturesQuery, ProteinIsoforms, ProteinRelationQuery,
    ProteinStructures, ReplyData, Request, Source, StepKind,
};
mod retained;
#[derive(Clone)]
pub(super) enum Selection {
    Features(ProteinFeaturesQuery),
    Isoforms(ProteinRelationQuery),
    Structures(ProteinRelationQuery),
}
impl Selection {
    fn accession(&self) -> &UniProtIsoform {
        match self {
            Self::Features(q) => q.accession(),
            Self::Isoforms(q) | Self::Structures(q) => q.accession(),
        }
    }
    fn window(&self) -> (u32, u32, Option<&Text>) {
        match self {
            Self::Features(q) => (q.limit(), q.offset(), q.pool()),
            Self::Isoforms(q) | Self::Structures(q) => (q.limit(), q.offset(), q.pool()),
        }
    }
    fn request(&self, page: Option<(Text, u32)>) -> Request {
        match self {
            Self::Features(q) => {
                Request::ProteinFeatures(page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)))
            }
            Self::Isoforms(q) => {
                Request::ProteinIsoforms(page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)))
            }
            Self::Structures(q) => Request::ProteinStructures(
                page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)),
            ),
        }
    }
    fn key(&self) -> Result<Vec<u8>, RequestError> {
        source_snapshot::key("protein-sections", &self.request(None))
    }

    fn total(&self, entry: &uniprot::Entry) -> usize {
        match self {
            Self::Features(q) => entry.features.iter().filter(|r| accepts(q, r)).count(),
            Self::Isoforms(_) => entry.isoforms.len(),
            Self::Structures(_) => entry.structures.len(),
        }
    }
    fn render(&self, entry: uniprot::Entry, out: &mut Outcome) -> usize {
        let (limit, offset, _) = self.window();
        match self {
            Self::Features(q) => {
                let rows: Vec<_> = entry
                    .features
                    .into_iter()
                    .filter(|r| accepts(q, r))
                    .collect();
                let total = rows.len();
                out.body = (ProteinFeatures {
                    items: rows
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect(),
                })
                .with_query(q.clone());
                total
            }
            Self::Isoforms(q) => {
                let total = entry.isoforms.len();
                out.body = (ProteinIsoforms {
                    items: entry
                        .isoforms
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect(),
                })
                .with_query(q.clone());
                total
            }
            Self::Structures(q) => {
                let total = entry.structures.len();
                out.body = (ProteinStructures {
                    items: entry
                        .structures
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect(),
                })
                .with_query(q.clone());
                total
            }
        }
    }
}
fn accepts(q: &ProteinFeaturesQuery, row: &biodata::ProteinFeature) -> bool {
    q.feature_type()
        .is_none_or(|t| row.feature.kind.label == *t)
        && q.position().is_none_or(|p| {
            let point = p.get() - 1;
            row.feature.protein == *q.accession()
                && row.feature.span.start().max().is_some_and(|n| n <= point)
                && row.feature.span.end().min().is_some_and(|n| n > point)
        })
}
fn restart(q: &Selection, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::UniProt],
        q.request(None),
        false,
    ));
}
pub(super) fn run(cx: &Ctx<'_>, q: Selection) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let body = match &q {
        Selection::Features(query) => {
            biomcp_types::ProteinFeatures::into_reply(query.clone(), None)
        }
        Selection::Isoforms(query) => {
            biomcp_types::ProteinIsoforms::into_reply(query.clone(), None)
        }
        Selection::Structures(query) => {
            biomcp_types::ProteinStructures::into_reply(query.clone(), None)
        }
    };
    let mut out = super::empty(body);
    if let Err(e) = read(cx, &q, deadline, &mut out) {
        out.local = status::local(e);
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[biomcp_types::Source::UniProt],
            q.request(None),
            true,
        ));
    }
    command.finish(deadline.remaining());
    out
}
fn read(
    cx: &Ctx<'_>,
    q: &Selection,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let Some((entry, pool)) = retained::load(cx, q, deadline, out)? else {
        return Ok(());
    };
    if let Selection::Features(query) = q
        && query.position().is_some_and(|p| {
            entry
                .header
                .protein
                .length
                .is_some_and(|length| p.get() > length)
        })
    {
        out.problems.push(biomcp_types::Problem {
            source: Source::UniProt,
            record: Text::new(q.accession().as_str()).ok(),
            field: "position".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
        return Ok(());
    }
    let total = q.render(entry, out);
    let (limit, offset, _) = q.window();
    if offset as usize > total {
        restart(q, out, false);
        return Ok(());
    }
    let returned = limit.min(total.saturating_sub(offset as usize) as u32);
    out.page = Some(PageInfo {
        offset,
        returned,
        total: total as u64,
        pool: pool.clone(),
        truncated: None,
    });
    {
        let unavailable = false;
        crate::status::item(out, true, false, unavailable);
    };
    if out
        .sources
        .iter()
        .any(|s| s.state == biomcp_types::SourceState::Stale)
    {
        crate::status::item(out, true, false, true);
    }
    if let Some(pool) = pool.filter(|_| (offset + returned) < total as u32) {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::UniProt],
            q.request(Some((pool.id, offset + returned))),
            false,
        ));
    }
    Ok(())
}
