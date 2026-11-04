use super::{SPEC, map, raw};
use crate::Mapped;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{DiseaseHeader, DiseaseSearch};
/// A single native page without local ranking or identity selection.
#[derive(Debug)]
pub struct SearchPage {
    /// Native window size before identity refusals.
    pub received: u32,
    /// Publisher total for the literal query and one ontology.
    pub total: u64,
    /// Headers in publisher order and field-level problems.
    pub rows: Mapped<Vec<DiseaseHeader>>,
}
/// Fetch exactly one page from exactly one admitted ontology.
pub fn search(leg: &Leg<'_>, query: &DiseaseSearch) -> Result<SearchPage, RequestError> {
    let ontology = crate::disease_sources::components::ontology(
        leg,
        query
            .ontology()
            .ok_or(biomcp_fetch::InputError::Serialization)?,
    )?;
    let text = query
        .text()
        .ok_or(biomcp_fetch::InputError::Serialization)?;
    let endpoint = SPEC.get("/api/search").fresh(Freshness::Days(1));
    let (page, _) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("q", Arg::Text(text)),
                ("rows", Arg::Num(u64::from(query.limit()))),
                ("start", Arg::Num(u64::from(query.offset()))),
                ("ontology", Arg::Text(ontology.as_str())),
                ("groupField", Arg::Text("iri")),
            ]),
            Read::<raw::Search>::json().check(&|page| {
                let response = &page.response;
                if response.start != query.offset()
                    || response.docs.len() > query.limit() as usize
                    || response
                        .docs
                        .iter()
                        .any(|d| d.ontology_name.as_str() != ontology.as_str())
                {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Ols,
        record: None,
        list: Vec::new(),
    };
    let received = page.response.docs.len() as u32;
    let mut value = Vec::new();
    let mut refusals = crate::Refusal::None;
    for hit in page.response.docs {
        problems.record = None;
        let Some(id) = problems.required_field("obo_id", hit.obo_id) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        problems.record = crate::mapped::record(&id);
        let iri = problems.optional("iri", hit.iri.map(|v| v.value().ok_or(())));
        let label = problems.optional("label", hit.label.map(|v| v.value().ok_or(())));
        value.push(DiseaseHeader {
            attribution: map::attribution(&id, iri, ontology)?,
            concept: biomcp_types::DiseaseConcept {
                id: Some(id),
                label: label.map(Into::into),
            },
            ontology,
            obsolete: problems.field("is_obsolete", hit.is_obsolete),
        });
    }
    Ok(SearchPage {
        received,
        total: page.response.total,
        rows: Mapped {
            refusals,
            value,
            problems: problems.list,
        },
    })
}
