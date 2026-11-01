//! Resolve the source's own candidate-to-CA cross-reference before the checked GRCh38 join.
//! No expression edit, assembly conversion, padding or candidate selection occurs here.
use crate::{Ctx, status};
use biodata::VariantId;
use biomcp_adapters::allele_registry;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{Problem, Reason, Source, SourceOutcome, VariantGet, VariantHeader};

#[derive(Default)]
pub(super) struct Identity {
    pub dropped: bool,
    pub rows: Vec<VariantHeader>,
    pub sources: Vec<SourceOutcome>,
    pub problems: Vec<Problem>,
    pub local: Option<RequestError>,
    pub reason: Option<Reason>,
    pub rejection: Option<biodata::Text>,
    pub error: Option<RequestError>,
    pub elapsed: std::time::Duration,
}
pub(super) fn read(
    cx: &Ctx<'_>,
    query: &VariantGet,
    deadline: Deadline<'_>,
    population: bool,
) -> Identity {
    let mut found = Identity::default();
    let spec = super::source(query);
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = if population && spec.source == Source::MyVariant {
        super::myvariant_identity(query, &leg, &mut found.problems, &mut found.dropped, true)
    } else {
        super::resolve(query, &leg, &mut found.problems, &mut found.dropped)
    };
    found.elapsed = started.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    found.error = error;
    found
        .sources
        .push(status::source(spec.source, leg.finish(), error));
    found.local = error.and_then(status::local);
    found.reason = error.and_then(status::reason);
    if let Ok((rows, rejection)) = result {
        found.rows = rows;
        if rejection.is_some() {
            found.reason = Some(Reason::Rejected);
            if let Some(source) = found.sources.first_mut() {
                source.state = biomcp_types::SourceState::Failed;
                source.reason = Some(Reason::Rejected);
            }
        }
        found.rejection = rejection;
    }
    if population && found.rows.len() == 1 && spec.source == Source::MyVariant {
        canonical(cx, deadline.after_stage(found.elapsed), &mut found);
    }
    found
}
fn canonical(cx: &Ctx<'_>, deadline: Deadline<'_>, found: &mut Identity) {
    let [row] = found.rows.as_mut_slice() else {
        return;
    };
    let ids: Vec<_> = row
        .identity
        .ids
        .iter()
        .filter(|id| matches!(id, VariantId::Ca(_)))
        .collect();
    let [id] = ids.as_slice() else {
        return;
    };
    let leg = cx
        .fetch
        .leg(&allele_registry::SPEC, deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let result = allele_registry::by_id(&leg, &id.to_string());
    found.elapsed += started.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    found
        .sources
        .push(status::source(Source::AlleleRegistry, leg.finish(), error));
    found.local = found.local.or(error.and_then(status::local));
    found.reason = error.and_then(status::reason);
    match result {
        Ok(allele_registry::Resolution::Found(mapped)) => {
            found.dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
            found.problems.extend(mapped.problems);
            let mut representations = row.identity.representations.to_vec();
            representations.extend(mapped.value.identity.representations);
            row.identity.representations = representations.into();
        }
        Ok(allele_registry::Resolution::Rejected(_)) => found.reason = Some(Reason::Rejected),
        Ok(allele_registry::Resolution::Refused(problems)) => found.problems.extend(problems),
        Err(_) => (),
    }
}

/// Recover one candidate using its own source build or its single validated CAid.
pub(super) fn choice(
    row: &VariantHeader,
    problems: &mut Vec<Problem>,
) -> Option<(biomcp_types::Request, Source)> {
    use biomcp_types::{Request, VariantResolver, VariantSearch};
    let build = row.identity.representations.iter().find_map(|r| r.build);
    let mut input = crate::next::checked(row.record.id.as_str().parse(), row.source, problems)?;
    if build.is_none() {
        let ids: Vec<_> = row
            .identity
            .ids
            .iter()
            .filter(|id| matches!(id, VariantId::Ca(_)))
            .collect();
        if let [id] = ids.as_slice() {
            input = crate::next::checked(
                biomcp_types::VariantInput::from_identity((*id).clone()),
                row.source,
                problems,
            )?;
        } else {
            let query =
                crate::next::checked(VariantSearch::source_record(&input), row.source, problems)?;
            return Some((Request::SearchVariant(query), Source::MyVariant));
        }
    }
    let query = crate::next::checked(
        VariantGet::new(input, VariantResolver::Car, build),
        row.source,
        problems,
    )?;
    let source = super::source(&query).source;
    Some((Request::GetVariant(query), source))
}

pub(super) fn requested(query: &VariantGet) -> bool {
    query.sections().iter().any(|section| {
        matches!(
            section,
            biomcp_types::VariantSection::Population | biomcp_types::VariantSection::All
        )
    })
}
