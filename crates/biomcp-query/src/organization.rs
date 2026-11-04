//! Organization discovery pages one complete NCI observation locally.
use crate::source_snapshot as snapshot;
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::nci;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{OrganizationPage, OrganizationSearch, Request, StepKind};
/// Find published organizations without assigning identities to names.
pub(crate) fn search(cx: &Ctx<'_>, q: &OrganizationSearch) -> Outcome {
    crate::nci_search::search(
        cx,
        q,
        nci::organization::search,
        "biomcp.organization-snapshot/2",
        biomcp_types::UsageError::OrganizationTooBroad,
        empty(q),
        page,
    )
}
impl crate::nci_search::Query for OrganizationSearch {
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
    q: &OrganizationSearch,
    saved: snapshot::Saved<biomcp_types::Organization>,
    pool: Option<biodata::Text>,
    out: &mut Outcome,
) {
    let total = saved.items.len() as u64;
    let items: Box<[_]> = saved
        .items
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    let returned = items.len() as u32;
    let next = q.offset().saturating_add(returned);
    if let Some(pool) = pool.filter(|_| returned > 0 && u64::from(next) < total) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Nci],
            run(q.at_page(next, pool)),
        ));
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
    out.body = (OrganizationPage { items }).with_query(q.clone());
}
fn empty(q: &OrganizationSearch) -> Outcome {
    crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (OrganizationPage {
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
fn run(q: OrganizationSearch) -> Action {
    Action::Run {
        request: Request::SearchOrganization(q),
        fresh: false,
    }
}
