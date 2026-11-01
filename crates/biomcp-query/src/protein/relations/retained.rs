use super::{Selection, restart};
use crate::{Ctx, Outcome, status, store::Store};
use biodata::Text;
use biomcp_adapters::uniprot::{self, Entry};
use biomcp_fetch::{CacheError, Deadline, RequestError, SourceError};
use biomcp_types::{PoolInfo, Reason, Source};
pub(super) fn load(
    cx: &Ctx<'_>,
    q: &Selection,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Entry, PoolInfo)>, RequestError> {
    let leg = cx.fetch.leg(&uniprot::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::UniProt) {
        let e = RequestError::Source(SourceError {
            source: Source::UniProt,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        });
        out.sources
            .push(status::source(Source::UniProt, leg.finish(), Some(e)));
        return Err(e);
    }
    let (_, offset, selected) = q.window();
    if selected.is_none() && offset != 0 || selected.is_some() && cx.fresh {
        restart(q, out, false);
        return Ok(None);
    }
    let key = q.key()?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let pool = if cx.fresh {
        None
    } else {
        store.find(&key, selected.map(Text::as_str))?
    };
    if let Some(pool) = pool {
        if pool.key != key {
            restart(q, out, true);
            return Ok(None);
        }
        let Some(payload) =
            store.source_snapshot(pool.info.id.as_str(), &key, |s| cx.fetch.disabled(s))?
        else {
            restart(q, out, false);
            return Ok(None);
        };
        let (entry, problems, dropped): (Entry, Vec<biomcp_types::Problem>, bool) =
            serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        out.sources = pool.sources;
        out.problems = problems;
        out.dropped = dropped;
        return Ok(Some((entry, pool.info)));
    }
    if selected.is_some() {
        restart(q, out, false);
        return Ok(None);
    }
    let result = uniprot::entry(&leg, q.accession());
    let error = result.as_ref().err().copied();
    out.sources
        .push(status::source(Source::UniProt, leg.finish(), error));
    let mapped = result?;
    out.include_refusals(mapped.refusals);
    out.problems = mapped.problems;
    let Some(entry) = mapped.value else {
        if !out.dropped {
            crate::status::item(out, false, false, false);
        }
        return Ok(None);
    };
    let size = q.total(&entry) as u32;
    let payload = serde_json::to_string(&(&entry, &out.problems, out.dropped))
        .map_err(|_| CacheError::Snapshot)?;
    let pool = store.publish_source_pool(&key, &payload, size, &out.sources)?;
    Ok(Some((entry, pool.info)))
}
