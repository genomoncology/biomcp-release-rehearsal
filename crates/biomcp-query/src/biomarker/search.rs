//! Biomarker discovery pages one complete NCI observation locally.
use crate::source_snapshot as snapshot;
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::nci;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{BiomarkerPage, BiomarkerSearch, Request, StepKind};
/// Find native biomarker concept headers without interpretation.
pub(crate) fn search(cx: &Ctx<'_>, q: &BiomarkerSearch) -> Outcome {
    crate::nci_search::search(
        cx,
        q,
        nci::biomarker::search,
        "biomcp.biomarker-snapshot/2",
        biomcp_types::UsageError::BiomarkerTooBroad,
        empty(q),
        page,
    )
}
impl crate::nci_search::Query for BiomarkerSearch {
    fn snapshot_query(&self) -> Self {
        self.snapshot_query()
    }
    fn pool(&self) -> Option<&biodata::Text> {
        self.pool()
    }
    fn restart_action(&self) -> Action {
        run(self.restart())
    }
}
fn page(
    q: &BiomarkerSearch,
    saved: snapshot::Saved<biomcp_types::BiomarkerCard>,
    pool: Option<biodata::Text>,
    out: &mut Outcome,
) {
    let total = saved.items.len() as u64;
    let items: Box<[_]> = saved
        .items
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .map(|card| card.header)
        .collect();
    for header in &items {
        for id in &header.codes {
            out.next.push(crate::next::Step::run(
                StepKind::Pivot,
                &[biomcp_types::Source::Nci],
                Request::SearchTrial(biomcp_types::TrialSearch::for_nci_biomarker(id.clone())),
                false,
            ));
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[biomcp_types::Source::Nci],
                Request::GetBiomarker(biomcp_types::BiomarkerGet::one(id.clone())),
                false,
            ));
        }
    }
    let returned = items.len() as u32;
    let next = q.offset().saturating_add(returned);
    if let Some(pool) = pool.filter(|_| returned > 0 && u64::from(next) < total) {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::Nci],
                run(q.at_page(next, pool)),
            ),
        );
    }
    out.page = Some(biomcp_types::PageInfo {
        offset: q.offset(),
        returned,
        total,
        pool: None,
        truncated: None,
    });
    out.sources = saved.sources;
    out.problems = saved.problems;
    out.dropped = saved.dropped;
    status::search(out);
    out.body = (BiomarkerPage { items }).with_query(q.clone());
}
fn empty(q: &BiomarkerSearch) -> Outcome {
    crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (BiomarkerPage {
                items: Box::default(),
            })
            .with_query(q.clone()),
            page: None,
            sources: Vec::new(),
            problems: Vec::new(),
            next: Vec::new(),
            local: None,
        },
    )
}
fn run(q: BiomarkerSearch) -> Action {
    Action::Run {
        request: Request::SearchBiomarker(q),
        fresh: false,
    }
}
