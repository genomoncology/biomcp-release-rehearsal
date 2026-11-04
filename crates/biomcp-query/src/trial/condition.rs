//! Trial-owned use of the accepted exact annotation resolver and shared source policy.
use crate::{Ctx, Outcome, status};
use biodata::{Curie, Text};
use biomcp_adapters::mydisease;
use biomcp_fetch::Deadline;
use biomcp_types::{ReplyData, Source, TrialConditionResolution, TrialSearch, TrialSource};

pub(super) fn resolve<'a>(
    cx: &Ctx<'a>,
    query: &TrialSearch,
    out: &mut Outcome,
    deadline: &mut Deadline<'a>,
) -> Option<Option<Text>> {
    let Some(input) = query.condition_id() else {
        return Some(None);
    };
    let target = if query.source() == TrialSource::Nci {
        Source::Nci
    } else {
        Source::ClinicalTrialsGov
    };
    // The target's ordinary transport admission returns its skipped outcome. Never resolve first.
    if cx.fetch.disabled(target)
        || (target == Source::Nci && !cx.fetch.has_key(biomcp_fetch::SecretName::NciApiKey))
    {
        return Some(None);
    }
    let mut resolution = TrialConditionResolution {
        input: input.clone(),
        resolver: None,
        concept: None,
        alias: None,
        ncit_candidates: Box::default(),
        selected_ncit: None,
        attribution: None,
        components: Box::default(),
    };
    if let Some(code) = ncit_code(input) {
        let value = Text::new(&code).ok()?;
        resolution.selected_ncit = Some(input.clone());
        assign(out, resolution);
        return Some(Some(value));
    }
    resolution.resolver = Some(Source::MyDisease);
    let leg = cx.fetch.leg(&mydisease::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let reply = mydisease::annotation(&leg, input);
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let error = reply.as_ref().err().copied();
    out.sources
        .push(status::source(Source::MyDisease, leg.finish(), error));
    out.local = error.and_then(status::local);
    let value = match reply {
        Ok(mapped) => {
            out.include_refusals(mapped.refusals);
            out.problems.extend(mapped.problems);
            resolved(query.source(), mapped.value, &mut resolution)
        }
        Err(_) => None,
    };
    if value.is_none() && resolution.concept.is_some() && query.source() == TrialSource::Nci {
        out.problems.push(biomcp_types::Problem {
            source: Source::MyDisease,
            record: Text::new(&input.to_string()).ok(),
            field: "ncit_mapping".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
    }
    assign(out, resolution);
    value.map(Some)
}
fn resolved(
    source: TrialSource,
    annotation: mydisease::Annotation,
    resolution: &mut TrialConditionResolution,
) -> Option<Text> {
    let card = annotation.card;
    let label = card.concept.label.clone();
    resolution.concept = Some(card.concept);
    resolution.alias = card.alias;
    resolution.attribution = Some(card.attribution);
    resolution.components = card.components;
    resolution.ncit_candidates = annotation
        .xrefs
        .into_iter()
        .filter(|row| row.object.id.to_string().starts_with("NCIT:"))
        .collect();
    if source == TrialSource::Ctgov {
        return label;
    }
    let choices: std::collections::BTreeSet<_> = resolution
        .ncit_candidates
        .iter()
        .map(|row| row.object.id.to_string())
        .collect();
    let unique = (choices.len() == 1)
        .then(|| choices.into_iter().next())
        .flatten()?;
    let unique: Curie = unique.parse().ok()?;
    let code = ncit_code(&unique)?;
    let value = Text::new(&code).ok()?;
    resolution.selected_ncit = Some(unique);
    Some(value)
}
fn ncit_code(id: &Curie) -> Option<String> {
    let value = id.to_string();
    let (namespace, local) = value.split_once(':')?;
    (namespace == "NCIT"
        && local
            .strip_prefix('C')
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|v| v.is_ascii_digit())))
    .then(|| local.to_owned())
}
fn assign(out: &mut Outcome, resolution: TrialConditionResolution) {
    match &mut out.body {
        ReplyData::TrialPage {
            body: Some(page), ..
        } => page.condition_resolution = Some(resolution),
        ReplyData::NciTrialPage {
            body: Some(page), ..
        } => page.condition_resolution = Some(resolution),
        _ => (),
    }
}

/// A lexical mismatch does not establish why the registry matched a record.
pub(super) fn word_notice(
    query: &biomcp_types::TrialSearch,
    fresh: bool,
    source: &mut biomcp_types::SourceOutcome,
    outcome: &mut super::Outcome,
) {
    source.notices.push(biomcp_types::SourceNotice::ConditionWordMismatch {
        message: concat!(
            "Some returned trials do not contain all searched words in their registered conditions. ",
            "ClinicalTrials.gov condition search also covers titles, keywords and indexed terms ",
            "and may expand synonyms."
        ).into(),
    });
    if let Some(query) = query.exact_condition_search() {
        outcome.next.push(crate::next::Step::new(
            biomcp_types::StepKind::Disambiguate,
            &[query.source().source()],
            super::Action::Run {
                request: biomcp_types::Request::SearchTrial(query),
                fresh,
            },
        ));
    }
}
