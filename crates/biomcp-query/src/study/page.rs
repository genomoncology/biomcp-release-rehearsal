use crate::{Ctx, Outcome, store::Store};
use biodata::StudyHeader;
use biomcp_fetch::{BundleInfo, CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, ReplyData, Request, StepKind, StudyGet, StudyPage, StudySearch, StudySnapshot,
};
type Retained = (
    Vec<StudyHeader>,
    StudySnapshot,
    Vec<biomcp_types::Problem>,
    bool,
);
pub(super) fn search(
    cx: &Ctx<'_>,
    q: &StudySearch,
    rows: Vec<StudyHeader>,
    info: BundleInfo,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let snapshot = biomcp_adapters::cbioportal::studies::snapshot(&info)?;
    let key = request_key(q, &snapshot)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let existing = store.find(&key, q.pool().map(|p| p.as_str()))?;
    let pool = if let Some(pool) = existing {
        pool
    } else if q.pool().is_some() {
        pool_error(cx, q, false, out);
        return Ok(());
    } else {
        let size = u32::try_from(rows.len()).map_err(|_| CacheError::Database)?;
        let payload = serde_json::to_string(&(rows, snapshot, &out.problems, out.dropped))
            .map_err(|_| CacheError::Snapshot)?;
        store.publish_source_pool(&key, &payload, size, &[])?
    };
    let Some(payload) =
        store.source_snapshot(pool.info.id.as_str(), &pool.key, |s| cx.fetch.disabled(s))?
    else {
        pool_error(cx, q, false, out);
        return Ok(());
    };
    let retained = serde_json::from_str::<Retained>(&payload).map_err(|_| CacheError::Snapshot)?;
    if pool.key != request_key(q, &retained.1)? {
        pool_error(cx, q, true, out);
        return Ok(());
    }
    render(cx, q, pool.info, retained, out)
}
fn render(
    cx: &Ctx<'_>,
    q: &StudySearch,
    pool: biomcp_types::PoolInfo,
    retained: Retained,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let (rows, snapshot, problems, dropped) = retained;
    out.dropped = dropped;
    out.problems = problems;
    let query = q.with_pool(Some(pool.id.clone()));
    let total = rows.len() as u64;
    let items: Box<[_]> = rows
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    let returned = items.len() as u32;
    let page = PageInfo {
        offset: q.offset(),
        returned,
        total,
        pool: Some(pool),
        truncated: None,
    };
    for row in &items {
        if let Some(get) = crate::next::checked(
            StudyGet::new(row.study_id.clone()),
            biomcp_types::Source::Cbioportal,
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[biomcp_types::Source::Cbioportal],
                Request::GetStudy(get),
                cx.fresh,
            ));
        }
    }
    if u64::from(q.offset()) + u64::from(returned) < total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::Cbioportal],
            Request::SearchStudy(query.at_offset(q.offset() + returned)),
            cx.fresh,
        ));
    }
    out.page = Some(page.clone());
    out.body = (StudyPage {
        items,
        page,
        snapshot,
    })
    .with_query(q.clone());
    crate::status::item(out, true, false, false);
    Ok(())
}

fn request_key(q: &StudySearch, snapshot: &StudySnapshot) -> Result<Vec<u8>, RequestError> {
    serde_json::to_vec(&(
        "study-catalog/3",
        q.text(),
        q.cancer_type(),
        &snapshot.revision,
    ))
    .map_err(|_| CacheError::Database.into())
}
fn pool_error(cx: &Ctx<'_>, q: &StudySearch, mismatch: bool, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::Cbioportal],
        Request::SearchStudy(q.with_pool(None).at_offset(0)),
        cx.fresh,
    ));
}
