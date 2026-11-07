//! Posted native values from the already fetched study; no statistical calculations.
use crate::{mapped::Problems, raw::Field};
use biomcp_types::{
    TrialAdverseEvents, TrialEvent, TrialEventGroup, TrialPostedOutcome, TrialResultRecord,
};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Results {
    outcome_measures_module: Option<Field<Plain<Outcomes>>>,
    adverse_events_module: Option<Field<Plain<Events>>>,
}
/// A module whose every string has lost the source's Markdown escapes before it
/// becomes a public value, so a field added to the public types cannot be missed.
struct Plain<T>(T);
impl<'de, T: DeserializeOwned> Deserialize<'de> for Plain<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut value = Value::deserialize(d)?;
        unescape(&mut value);
        T::deserialize(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
fn unescape(value: &mut Value) {
    match value {
        Value::String(text) => {
            if let std::borrow::Cow::Owned(plain) = crate::source_text::unescape_markdown(text) {
                *text = plain;
            }
        }
        Value::Array(rows) => rows.iter_mut().for_each(unescape),
        Value::Object(fields) => fields.values_mut().for_each(unescape),
        _ => {}
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Outcomes {
    outcome_measures: Option<Field<Vec<Field<TrialPostedOutcome>>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Events {
    #[serde(flatten)]
    scope: TrialAdverseEvents,
    event_groups: Option<Field<Vec<Field<TrialEventGroup>>>>,
    serious_events: Option<Field<Vec<Field<TrialEvent>>>>,
    other_events: Option<Field<Vec<Field<TrialEvent>>>>,
}
impl Results {
    pub(super) fn rows(
        self,
        problems: &mut Problems,
        refusals: &mut crate::Refusal,
    ) -> Box<[TrialResultRecord]> {
        let mut rows = Vec::new();
        if let Some(Plain(module)) = problems.field(
            "resultsSection.outcomeMeasuresModule",
            self.outcome_measures_module,
        ) {
            rows.extend(
                super::super::refusals::records(
                    problems,
                    refusals,
                    "outcomeMeasures",
                    module.outcome_measures,
                )
                .into_vec()
                .into_iter()
                .map(TrialResultRecord::Outcome),
            );
        }
        if let Some(Plain(module)) = problems.field(
            "resultsSection.adverseEventsModule",
            self.adverse_events_module,
        ) {
            rows.push(TrialResultRecord::AdverseEvents(module.scope));
            rows.extend(
                super::super::refusals::records(
                    problems,
                    refusals,
                    "eventGroups",
                    module.event_groups,
                )
                .into_vec()
                .into_iter()
                .map(TrialResultRecord::AdverseEventGroup),
            );
            for (name, events, record) in [
                (
                    "seriousEvents",
                    module.serious_events,
                    TrialResultRecord::SeriousEvent as fn(TrialEvent) -> TrialResultRecord,
                ),
                (
                    "otherEvents",
                    module.other_events,
                    TrialResultRecord::OtherEvent as fn(TrialEvent) -> TrialResultRecord,
                ),
            ] {
                rows.extend(
                    super::super::refusals::records(problems, refusals, name, events)
                        .into_vec()
                        .into_iter()
                        .map(record),
                );
            }
        }
        rows.into()
    }
}
