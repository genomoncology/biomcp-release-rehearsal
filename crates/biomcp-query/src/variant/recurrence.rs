//! Page-local exact sample observations with explicit cohort and source provenance.
use biomcp_types::ReplyBody;
mod matching;
use crate::{Ctx, Outcome, status};

use biomcp_adapters::{cbioportal, hgnc};
use biomcp_fetch::{Deadline, Find, KeyKind, RequestError};
use biomcp_types::{Reason, Request, Source, StepKind, VariantRecurrence, VariantRecurrenceQuery};
/// Read one native page without inventing an assayed denominator or selecting an ambiguous allele.
pub(crate) fn run(cx: &Ctx<'_>, query: &VariantRecurrenceQuery) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = crate::status::empty((empty()).with_query(query.clone()));
    if let Err(error) = execute(cx, query, deadline, &mut out) {
        out.local = status::local(error);
    }
    command.finish(deadline.remaining());
    out
}
fn empty() -> VariantRecurrence {
    VariantRecurrence {
        identity: None,
        matching_granularity: biomcp_types::RecurrenceGranularity::ExactGenomicAllele,
        entrez_gene_id: None,
        study: None,
        selected_sample_count: None,
        observed_sample_count: None,
        examined_rows: 0,
        all_pages_read: false,
        more: false,
        terms: biomcp_types::RecurrenceTerms {
            unverified: true,
            notice: "Individual studies may impose commercial restrictions; public access does not verify study-specific reuse rights".into(),
            url: "https://docs.cbioportal.org/user-guide/faq/".into(),
        },
    }
}
fn execute(
    cx: &Ctx<'_>,
    query: &VariantRecurrenceQuery,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if cx.fetch.disabled(Source::Cbioportal) {
        let leg = cx.fetch.leg(&cbioportal::SPEC, deadline);
        let error = rejected(Source::Cbioportal, Reason::Disabled);
        out.sources.push(status::source(
            Source::Cbioportal,
            leg.finish(),
            Some(error),
        ));
        return Err(error);
    }
    let identity = identity(cx, query, deadline, out)?;
    let gene = gene(cx, query, deadline, out)?;
    let leg = cx.fetch.leg(&cbioportal::SPEC, deadline).fresh(cx.fresh);
    let result = cbioportal::cohort(&leg, query)
        .and_then(|cohort| cbioportal::mutations(&leg, query, gene).map(|rows| (cohort, rows)));
    out.sources.push(status::source(
        Source::Cbioportal,
        leg.finish(),
        result.as_ref().err().copied(),
    ));
    let (cohort, rows) = result?;
    out.problems.extend(rows.problems);
    let window = rows.window;
    out.include_refusals(window.refusals());
    metadata(query, &cohort, out);
    let (count, problems) = matching::count(&identity, &window.rows, query.study());
    out.problems.extend(problems);
    let more = window.has_more();
    let mut body = empty();
    body.identity = Some(identity);
    body.entrez_gene_id = Some(gene);
    body.study = Some(cohort.study);
    body.selected_sample_count = cohort.selected_sample_count;
    body.observed_sample_count = Some(count);
    body.examined_rows = window.received();
    body.all_pages_read = query.page() == 0 && !more;
    body.more = more;
    out.body = body.with_query(query.clone());
    {
        let unavailable = !(out
            .sources
            .iter()
            .all(|s| s.state == biomcp_types::SourceState::Ok));
        crate::status::item(out, true, false, unavailable);
    };
    if more && let Some(next) = query.next_page() {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::CancerHotspots],
            Request::VariantRecurrence(next),
            false,
        ));
    }
    Ok(())
}
fn metadata(query: &VariantRecurrenceQuery, cohort: &cbioportal::Cohort, out: &mut Outcome) {
    out.problems.extend(cohort.problems.iter().cloned());
    if let Some(error) = cohort.count_error {
        if let Some(source) = out.sources.last_mut() {
            source.state = biomcp_types::SourceState::Failed;
            if let Some(reason) = status::reason(error) {
                source.reason = Some(reason);
            }
        }
        out.local = status::local(error);
        out.problems.push(matching::problem(
            Source::Cbioportal,
            query.sample_set().as_str(),
            "sample_set.metadata",
        ));
    }
    if cohort.selected_sample_count.is_none() {
        out.problems.push(matching::problem(
            Source::Cbioportal,
            query.sample_set().as_str(),
            "sample_set.selected_sample_count",
        ));
    }
}

fn identity(
    cx: &Ctx<'_>,
    query: &VariantRecurrenceQuery,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<biodata::Variant, RequestError> {
    let q = biomcp_types::VariantGet::new(
        query.input().clone(),
        biomcp_types::VariantResolver::Car,
        query.build(),
    )
    .map_err(|_| rejected(Source::AlleleRegistry, Reason::Rejected))?;
    let spec = super::source(&q);
    let found = super::identity::read(cx, &q, deadline);
    out.sources.extend(found.sources);
    out.problems.extend(found.problems);
    out.dropped |= found.dropped;
    out.local = out.local.or(found.local);
    if let Some(error) = found.error {
        return Err(error);
    }
    let mut candidates = found.rows;
    let rejection = found.rejection;
    if candidates.len() != 1
        || rejection.is_some()
        || !candidates
            .first()
            .is_some_and(|c| c.identity.representations.iter().any(matching::supported))
    {
        out.problems
            .push(super::identity::missing(Source::Cbioportal));
        out.problems.push(matching::problem(
            spec.source,
            query.input().as_str(),
            "identity.exact_allele",
        ));
        if let Some(source) = out.sources.last_mut() {
            source.state = biomcp_types::SourceState::Failed;
            source.reason = Some(Reason::Rejected);
        }
        return Err(rejected(spec.source, Reason::Rejected));
    }
    Ok(candidates.remove(0).identity)
}
fn gene(
    cx: &Ctx<'_>,
    query: &VariantRecurrenceQuery,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<biodata::NcbiGeneId, RequestError> {
    let leg = cx.fetch.leg(&hgnc::SPEC, deadline).fresh(cx.fresh);
    let result = hgnc::lookup(&leg, Find::Exact(KeyKind::Symbol, query.gene().as_str()));
    out.sources.push(status::source(
        Source::Hgnc,
        leg.finish(),
        result.as_ref().err().copied(),
    ));
    let mapped = result?;
    out.include_refusals(mapped.refusals);
    out.include_refusals(mapped.refusals);
    out.problems.extend(mapped.problems);
    if mapped.value.len() == 1
        && let Some(id) = mapped.value.first().and_then(|v| v.entrez_id)
    {
        return Ok(id);
    }
    out.problems.push(matching::problem(
        Source::Hgnc,
        query.gene().as_str(),
        "gene.entrez_id",
    ));
    Err(rejected(Source::Hgnc, Reason::Rejected))
}
fn rejected(source: Source, reason: Reason) -> RequestError {
    biomcp_fetch::SourceError {
        source,
        reason,
        status: None,
        retry_after: None,
    }
    .into()
}
