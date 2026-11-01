use super::{Selection, restart};
use crate::{Ctx, Outcome, source_snapshot, status};
use biomcp_adapters::uniprot::{self, Entry};
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::{PoolInfo, Source};
pub(super) fn load(
    cx: &Ctx<'_>,
    q: &Selection,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Entry, Option<PoolInfo>)>, RequestError> {
    let leg = cx.fetch.leg(&uniprot::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::UniProt) {
        let e = RequestError::Source(SourceError::disabled(Source::UniProt));
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
    type Saved = (Entry, Vec<biomcp_types::Problem>, bool);
    let (saved, mismatch) = if cx.fresh {
        (None, false)
    } else {
        source_snapshot::find::<Saved>(cx, &key, selected, deadline)?
    };
    if let Some(((entry, problems, dropped), pool)) = saved {
        out.sources = pool.sources;
        out.problems = problems;
        out.dropped = dropped;
        return Ok(Some((entry, Some(pool.info))));
    }
    if selected.is_some() {
        restart(q, out, mismatch);
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
    let pool = crate::publish::keep(
        source_snapshot::publish(
            cx,
            &key,
            &(&entry, &out.problems, out.dropped),
            size,
            &out.sources,
            None,
            deadline,
        ),
        out,
    )?;
    Ok(Some((entry, pool.map(|p| p.info))))
}
