//! Independent NCI source: never a registry fallback or local disease matcher.
use biomcp_types::ReplyBody;
mod get;
mod page;
use crate::{Action, Ctx, Outcome, status};
use biodata::Text;
use biomcp_adapters::nci;
use biomcp_types::{NciTrialPage, Reason, ReplyData, Request, Source, StepKind, TrialSearch};
pub(super) use get::run as get;
const ATTRIBUTION: &str =
    "NCI Clinical Trials Search; source-native fields, no eligibility assessment";
pub(super) fn search(cx: &Ctx<'_>, q: &TrialSearch) -> Outcome {
    let mut out = empty(
        (NciTrialPage {
            items: Box::default(),
            request: Box::default(),
            condition_resolution: None,
            attribution: ATTRIBUTION.into(),
        })
        .with_query(q.clone()),
    );
    if q.nci_disease_invalid() {
        out.body = ReplyData::Usage(biomcp_types::UsageError::NciDiseaseError);
        return out;
    }
    if let Some(option) = q.unsupported() {
        out.body = ReplyData::Usage(biomcp_types::UsageError::UnsupportedTrialFilter {
            source: Source::Nci,
            option,
        });
        return out;
    }
    if q.limit() > nci::PAGE_LIMIT
        || q.next_page()
            .is_some_and(|p| p.as_str().parse::<u32>().is_err())
    {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return out;
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let Some(condition) = super::condition::resolve(cx, q, &mut out, &mut deadline) else {
        command.finish(deadline.remaining());
        return out;
    };
    let leg = cx.fetch.leg(&nci::SPEC, deadline).fresh(cx.fresh);
    let mut params = nci::parameters(q);
    if let Some(condition) = condition {
        params.insert(
            2,
            (
                "diseases.nci_thesaurus_concept_id".into(),
                condition.to_string(),
            ),
        );
    }
    if let ReplyData::NciTrialPage {
        body: Some(body), ..
    } = &mut out.body
    {
        body.request = params
            .iter()
            .map(|(name, value)| biomcp_types::TrialParameter {
                name: name.clone(),
                value: value.clone(),
            })
            .collect();
    }
    let result = nci::search(&leg, &params);
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::Nci, leg.finish(), error);
    if let Ok(page) = result {
        apply(q, cx.fresh, &mut out, &mut source, page);
    }
    source.notices.extend(super::drug_names::literal_notices(q));
    finish(&mut out, source, error);
    status::search_for(&mut out, &[Source::Nci]);
    command.finish(deadline.remaining());
    out
}

fn finish(
    out: &mut Outcome,
    source: biomcp_types::SourceOutcome,
    error: Option<biomcp_fetch::RequestError>,
) {
    if source.reason == Some(Reason::NoKey) {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Key,
                &[Source::Nci],
                Action::KeyHelp(biomcp_fetch::SecretName::NciApiKey),
            ),
        );
    }
    out.local = error.and_then(status::local);
    out.sources.push(source);
}
pub(super) use crate::status::empty;

fn apply(
    q: &TrialSearch,
    fresh: bool,
    out: &mut Outcome,
    source: &mut biomcp_types::SourceOutcome,
    page: nci::Page,
) {
    out.include_refusals(page.refusals);
    source.count = Some(page.total);
    source.retrieved_count = Some(page.data.len() as u32);
    {
        let unavailable = source.state == biomcp_types::SourceState::Stale
            || out
                .sources
                .iter()
                .any(|s| s.state != biomcp_types::SourceState::Ok);
        crate::status::item(out, true, false, unavailable);
    };
    let offset = q
        .next_page()
        .and_then(|v| v.as_str().parse::<u32>().ok())
        .unwrap_or_default();
    let returned = if q.count_only() {
        0
    } else {
        page.data.len() as u32
    };
    out.page = Some(biomcp_types::PageInfo {
        offset,
        returned,
        total: page.total,
        pool: None,
        truncated: None,
    });
    if !q.count_only() && returned > 0 && u64::from(offset.saturating_add(returned)) < page.total {
        let token = Text::new(&offset.saturating_add(returned).to_string());
        if let Some(token) = crate::next::checked(token, Source::Nci, &mut out.problems) {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[Source::Nci],
                Request::SearchTrial(q.at_page(token, offset.saturating_add(returned))),
                fresh,
            ));
        }
    }
    if let ReplyData::NciTrialPage {
        body: Some(body), ..
    } = &mut out.body
        && !q.count_only()
    {
        body.items = page
            .data
            .into_iter()
            .map(|mut r| {
                out.problems.extend(r.problems().iter().cloned());
                let card = if let Some(name) = q.facility() {
                    let mapped = r.facility_card(name);
                    out.dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    out.problems.extend(mapped.problems);
                    mapped.value
                } else {
                    r.card(false)
                };
                if let Some(code) = q.biomarker_code() {
                    let mapped = r.biomarker_card(code, q.biomarker_use(), card);
                    out.dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    out.problems.extend(mapped.problems);
                    mapped.value
                } else {
                    card
                }
            })
            .collect();
        let problems: Vec<_> = body.items.iter().filter_map(phase_problem).collect();
        out.problems.extend(problems);
    }
}

fn phase_problem(card: &biomcp_types::NciTrial) -> Option<biomcp_types::Problem> {
    card.phase
        .as_ref()
        .filter(|phase| phase.is_unknown())
        .map(|_| biomcp_types::Problem {
            source: Source::Nci,
            record: Some(card.nci_id.clone()),
            field: "phase".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        })
}
