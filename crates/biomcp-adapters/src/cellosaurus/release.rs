use super::{Arg, Freshness, Input, Leg, Read, RequestError, SPEC, Source, raw};

/// Independent cacheable release provenance; no database counts become search totals.
pub fn release(
    leg: &Leg<'_>,
) -> Result<crate::Mapped<biomcp_types::CellLineRelease>, RequestError> {
    let endpoint = SPEC.get("/release-info").fresh(Freshness::Days(1));
    let reply: raw::ReleaseReply = leg
        .call(
            &endpoint,
            Input::Params(&[("format", Arg::Text("json"))]),
            Read::json(),
        )?
        .value;
    let release = reply.data.header.release;
    let mut problems = crate::mapped::Problems {
        source: Source::Cellosaurus,
        record: None,
        list: Vec::new(),
    };
    let value = biomcp_types::CellLineRelease {
        version: problems.required_field("release.version", release.version),
        updated: problems.required_field("release.updated", release.updated),
    };
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
