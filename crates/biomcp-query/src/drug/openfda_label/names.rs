//! Retain the authorized spelling union and the provider tier selected on its first page.
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::Text;
use biomcp_adapters::openfda_label::NameTier;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{DrugSearch, Paged, ReplyData, Request, SourceNotice, StepKind};
use serde::{Deserialize, Serialize};
mod resolve;
#[derive(Serialize, Deserialize)]
pub(super) struct Saved {
    names: Vec<Text>,
    sources: Vec<biomcp_types::SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
    incomplete: bool,
    omitted: usize,
    tier: Option<NameTier>,
}
pub(super) fn prepare<'a>(
    cx: &Ctx<'a>,
    query: &DrugSearch,
    deadline: &mut Deadline<'a>,
) -> Result<Option<Saved>, RequestError> {
    if let Some(token) = query.name_snapshot() {
        let key = source_snapshot::key("label-drug-names", &query.restart())?;
        let saved: Option<Saved> = source_snapshot::read(cx, &key, token.as_str(), *deadline)?;
        return Ok(saved.filter(|saved| {
            saved.tier.is_some() && query.label_name_selector(&saved.names, true).is_ok()
        }));
    }
    Ok(Some(resolve::resolve(cx, query, deadline)))
}
pub(super) fn search(
    leg: &biomcp_fetch::Leg<'_>,
    query: &DrugSearch,
    saved: &mut Saved,
) -> biomcp_adapters::openfda_label::Search {
    let (result, tier) =
        biomcp_adapters::openfda_label::search_expanded(leg, query, &saved.names, saved.tier);
    saved.tier = Some(tier);
    result
}
pub(super) fn retain(
    cx: &Ctx<'_>,
    query: &DrugSearch,
    saved: Option<&Saved>,
    deadline: Deadline<'_>,
    success: bool,
    out: &mut Outcome,
) -> DrugSearch {
    let Some(saved) = saved.filter(|_| success && query.name_snapshot().is_none()) else {
        return query.clone();
    };
    let result = source_snapshot::key("label-drug-names", &query.restart()).and_then(|key| {
        let mut admitted: Vec<_> = saved
            .sources
            .iter()
            .filter(|source| source.state != biomcp_types::SourceState::Skipped)
            .cloned()
            .collect();
        admitted.extend(out.sources.iter().cloned());
        source_snapshot::write(cx, &key, saved, &admitted, deadline)
    });
    match crate::publish::keep(result, out) {
        Ok(Some(token)) => query.with_name_snapshot(token),
        Ok(None) => query.clone(),
        Err(error) => {
            out.local = Some(error);
            query.clone()
        }
    }
}
pub(super) fn missing(query: &DrugSearch, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch: false };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[],
        Request::SearchDrug(query.restart()),
        false,
    ));
}
pub(super) fn finish(query: &DrugSearch, saved: Option<Saved>, out: &mut Outcome) {
    let Some(saved) = saved else { return };
    if let Some(source) = out.sources.first_mut() {
        source.term = match saved.tier {
            Some(tier) => match query.label_name_selector(&saved.names, tier.exact()) {
                Ok(selector) => Some(selector),
                Err(_) => {
                    out.local = Some(biomcp_fetch::CacheError::Snapshot.into());
                    None
                }
            },
            None => None,
        };
        if let Some(input) = query.query() {
            source.notices.push(SourceNotice::DrugNameSearch {
                input: input.clone(), names: saved.names.clone().into(),
                message: format!(
                    "Searched these literal spellings in one native label name union; counts are native label records, not sums. \
                     Name discovery is bounded and does not establish drug-wide completeness. \
                     Left out {} additional names at the whole-query bound.", saved.omitted
                ).into(),
            });
        }
    }
    // Classify the primary provider before adding discovery evidence, which can name this provider too.
    if out.status() == biomcp_types::Status::Complete && saved.incomplete {
        status::item(out, true, false, true);
    }
    out.sources.extend(saved.sources);
    out.problems.extend(saved.problems);
}
