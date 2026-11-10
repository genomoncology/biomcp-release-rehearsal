//! Source-stated aliases may recover an empty source lookup, never substitute its records.
use super::resolve::within;
use crate::{Ctx, Outcome, status};
use biodata::Text;
use biomcp_types::{
    DrugCard, DrugGet, DrugNameSelector, DrugSearch, DrugSource, ReplyBody, ReplyData, Request,
    SourceNotice, SourceOutcome, SourceState, StepKind,
};

pub(crate) fn search(cx: &Ctx<'_>, query: &DrugSearch) -> Outcome {
    if query.expand_drug_names() {
        return super::search(cx, query);
    }
    let deadline = cx.fetch.deadline(cx.budget);
    let mut out = search_spelling(cx, query);
    if query.offset() != 0 || !zero(&out, query.source()) {
        return out;
    }
    let Some(input) = query.query() else {
        return out;
    };
    let (found, card, evidence) = with_aliases(
        &within(cx, deadline.remaining()),
        input,
        query.source(),
        |card, name, _| {
            if query.explicit_name_selector() {
                return None;
            }
            let retry = query.with_name(name.clone());
            let mut found = search_spelling(&within(cx, deadline.remaining()), &retry);
            if zero(&found, query.source()) {
                return None;
            }
            notice(
                &mut found,
                query.source(),
                format!(
                    "The search for {input:?} returned no rows. {} record {} supplies the name {name:?}; \
                     tried the selected source with that name. \
                     Returned rows and totals belong to the selected source.",
                    card.header.source, card.header.id
                ),
            );
            Some(found)
        },
    );
    if let Some(found) = found {
        return found;
    }
    evidence.append(&mut out, query.source());
    if evidence.local.is_some() {
        // A local failure while looking up names leaves the search incomplete, as its exit code says.
        status::search(&mut out);
    }
    if let Some(card) = card {
        hint(&mut out, &card, input, query.source(), cx.fresh);
    }
    out
}

pub(super) fn get(cx: &Ctx<'_>, query: &DrugGet) -> Outcome {
    let deadline = cx.fetch.deadline(cx.budget);
    let mut out = super::resolve::get(cx, query);
    if !matches!(out.body, ReplyData::Usage(_)) || !usable(&out, query.source()) {
        return out;
    }
    let input = query.input();
    let (found, card, evidence) = with_aliases(
        &within(cx, deadline.remaining()),
        input,
        query.source(),
        |card, name, evidence| {
            let retry = crate::next::checked(
                query.with_name(name.clone()),
                card.header.source.source(),
                &mut evidence.problems,
            )?;
            let mut found = super::resolve::get(&within(cx, deadline.remaining()), &retry);
            if matches!(found.body, ReplyData::Usage(_)) {
                return None;
            }
            notice(
                &mut found,
                query.source(),
                format!(
                    "The source did not resolve {input:?}. {} record {} supplies the name {name:?}; \
                     tried resolving that name in the selected source. \
                     Only that source can supply its own record.",
                    card.header.source, card.header.id
                ),
            );
            Some(found)
        },
    );
    if let Some(found) = found {
        return found;
    }
    evidence.append(&mut out, query.source());
    if let Some(card) = card {
        // A Usage reply discards source metadata, including the other-source hint.
        out.body = DrugCard::into_reply(query.clone(), None);
        out.page = None;
        hint(&mut out, &card, input, query.source(), cx.fresh);
        status::item(&mut out, false, false, false);
    }
    out
}

#[derive(Default)]
struct Evidence {
    sources: Vec<SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
    local: Option<biomcp_fetch::RequestError>,
}
impl Evidence {
    fn append(&self, out: &mut Outcome, selected: DrugSource) {
        for source in &self.sources {
            // MyChem component metadata cannot replace the selected source's native outcome.
            if source.source == selected.source() {
                continue;
            }
            if let Some(existing) = out.sources.iter_mut().find(|s| s.source == source.source) {
                status::retain_source_state(existing, source);
            } else {
                out.sources.push(source.clone());
            }
        }
        for problem in &self.problems {
            if !out.problems.contains(problem) {
                out.problems.push(problem.clone())
            }
        }
        out.local = out.local.or(self.local);
    }
}

fn with_aliases(
    cx: &Ctx<'_>,
    input: &Text,
    selected: DrugSource,
    mut attempt: impl FnMut(&DrugCard, &Text, &mut Evidence) -> Option<Outcome>,
) -> (Option<Outcome>, Option<DrugCard>, Evidence) {
    let deadline = cx.fetch.deadline(cx.budget);
    let mut evidence = Evidence::default();
    let mut alternatives = None;
    let mut tried = std::collections::BTreeSet::from([input.as_str().to_lowercase()]);
    for source in [
        DrugSource::Mychem,
        DrugSource::OpenfdaLabel,
        DrugSource::Chembl,
    ] {
        if source == selected {
            continue;
        }
        if deadline.remaining().is_zero() {
            break;
        }
        let Some(query) = crate::next::checked(
            DrugGet::named(input.clone(), source),
            source.source(),
            &mut evidence.problems,
        ) else {
            continue;
        };
        let out = super::resolve::get(&within(cx, deadline.remaining()), &query);
        evidence.sources.extend(out.sources.iter().cloned());
        evidence.problems.extend(out.problems.iter().cloned());
        evidence.local = evidence.local.or(out.local);
        if !usable(&out, source) || out.dropped {
            continue;
        }
        if let ReplyData::DrugCard {
            body: Some(card), ..
        } = out.body
        {
            let exact = !ambiguous(&card) && literal_name(&card, input);
            if exact {
                for name in names(&card) {
                    if deadline.remaining().is_zero() {
                        break;
                    }
                    if !tried.insert(name.as_str().to_lowercase()) {
                        continue;
                    }
                    if let Some(mut found) = attempt(&card, name, &mut evidence) {
                        evidence.append(&mut found, selected);
                        return (Some(found), None, evidence);
                    }
                }
            }
            if alternatives.as_ref().is_none_or(|previous: &DrugCard| {
                exact && (ambiguous(previous) || !literal_name(previous, input))
            }) {
                alternatives = Some(card);
            }
        }
    }
    (None, alternatives, evidence)
}

pub(super) fn ambiguous(card: &DrugCard) -> bool {
    card.header
        .name_resolution
        .as_ref()
        .is_none_or(|r| r.total != 1)
}

pub(super) fn literal_name(card: &DrugCard, input: &Text) -> bool {
    names(card)
        .iter()
        .any(|name| biomcp_types::DrugNameMatch::same_name(name.as_str(), input.as_str()))
}

pub(super) fn names(card: &DrugCard) -> Vec<&Text> {
    let mut seen = std::collections::BTreeSet::new();
    card.header
        .name
        .iter()
        .chain(card.alternate_names.iter())
        .filter(|name| seen.insert(name.as_str().to_lowercase()))
        .collect()
}

fn usable(out: &Outcome, source: DrugSource) -> bool {
    out.local.is_none()
        && out
            .sources
            .iter()
            .any(|s| s.source == source.source() && s.state == SourceState::Ok)
}

fn zero(out: &Outcome, source: DrugSource) -> bool {
    usable(out, source)
        && !out.dropped
        && out
            .page
            .as_ref()
            .is_some_and(|page| page.total == 0 && page.returned == 0)
}

fn notice(out: &mut Outcome, selected: DrugSource, message: String) {
    if let Some(source) = out
        .sources
        .iter_mut()
        .find(|s| s.source == selected.source())
    {
        source.notices.push(SourceNotice::RequestGuidance {
            data_date: None,
            message: message.into(),
            retry_after_seconds: None,
        });
    }
}

fn hint(out: &mut Outcome, card: &DrugCard, input: &Text, selected: DrugSource, fresh: bool) {
    if ambiguous(card) || !literal_name(card, input) {
        notice(
            out,
            selected,
            format!(
                "{} has candidate records for {input:?}, but no unique literal name match. \
                 No cross-source alias was selected. Search that source to inspect its candidates.",
                card.header.source
            ),
        );
        if let Some(query) = crate::next::checked(
            DrugSearch::named(input.clone(), card.header.source, 10),
            card.header.source.source(),
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[card.header.source.source()],
                Request::SearchDrug(query),
                fresh,
            ));
        }
        return;
    }
    notice(
        out,
        selected,
        format!(
            "The completed queries in {selected} found no matching record for {input:?}. \
             {} record {} matches the input; its presence does not establish a record in {selected}.",
            card.header.source, card.header.id
        ),
    );
    if let Some(query) = crate::next::checked(
        DrugGet::new(card.header.id.clone(), card.header.source),
        card.header.source.source(),
        &mut out.problems,
    ) {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &[card.header.source.source()],
            Request::GetDrug(query),
            fresh,
        ));
    }
    status::search_for(out, &[selected.source()]);
}

fn search_spelling(cx: &Ctx<'_>, query: &DrugSearch) -> Outcome {
    let deadline = cx.fetch.deadline(cx.budget);
    let mut out = super::search(cx, query);
    if query.source() != DrugSource::Chembl
        || query.explicit_name_selector()
        || query.offset() != 0
        || !zero(&out, query.source())
    {
        return out;
    }
    let Some(synonym) = crate::next::checked(
        query.with_name_selector(DrugNameSelector::Synonym),
        query.source().source(),
        &mut out.problems,
    ) else {
        return out;
    };
    let mut found = super::search(&within(cx, deadline.remaining()), &synonym);
    notice(
        &mut found,
        query.source(),
        format!(
            "The preferred-name search for {:?} returned no rows; tried the native exact synonym filter.",
            query.query()
        ),
    );
    found
}
