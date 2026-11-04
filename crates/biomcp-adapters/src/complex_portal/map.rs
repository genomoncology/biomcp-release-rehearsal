use super::{Complex, raw};
use biodata::{ProteinComplexHeader, ProteinComplexMember, UniProtIsoform};
use biomcp_fetch::RequestError;
pub(super) fn members(
    rows: Vec<raw::Member>,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Result<Vec<ProteinComplexMember>, RequestError> {
    rows.into_iter()
        .map(|r| {
            let protein = if r.interactor_type == "protein" {
                problems.required(
                    "participants.protein",
                    UniProtIsoform::new(r.identifier.as_str())
                        .filter(|p| p.as_str() == r.identifier),
                )
            } else {
                None
            };
            let mut xrefs = Vec::new();
            let (field_name, references) = if r.xrefs.is_some() {
                ("participants.xrefs", r.xrefs)
            } else {
                ("participants.crossReferences", r.cross_references)
            };
            if let Some(field) = references
                && let Some(values) = problems.required(field_name, field.value())
            {
                for value in values {
                    let parsed = value
                        .value()
                        .and_then(|x| format!("{}:{}", x.database, x.identifier).parse().ok());
                    if let Some(value) = problems.required(field_name, parsed) {
                        xrefs.push(value);
                    } else {
                        *refusals = crate::Refusal::Refused;
                    }
                }
            }
            let organism = crate::mapped::field_text(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                "participants.organismName",
                r.organism_name,
            );
            Ok(ProteinComplexMember {
                id: crate::mapped::text(&r.identifier).map_err(|_| {
                    biomcp_fetch::bad_reply(biomcp_types::Source::ComplexPortal, Some(200))
                })?,
                participant_type: crate::mapped::text(&r.interactor_type).map_err(|_| {
                    biomcp_fetch::bad_reply(biomcp_types::Source::ComplexPortal, Some(200))
                })?,
                taxon: problems
                    .optional(
                        "participants.taxon",
                        organism.as_ref().map(|s| raw::taxon(s.as_str())),
                    )
                    .flatten(),
                organism,
                name: crate::mapped::field_text(
                    &mut problems.list,
                    problems.source,
                    problems.record.clone(),
                    "participants.name",
                    r.name,
                ),
                stoichiometry: crate::mapped::field_code_text(
                    &mut problems.list,
                    problems.source,
                    problems.record.clone(),
                    "participants.stochiometry",
                    r.stochiometry,
                ),
                xrefs: xrefs.into(),
                protein,
            })
        })
        .collect()
}
pub(super) fn complex(
    raw: raw::Complex,
    anchor: &UniProtIsoform,
) -> Result<Option<Complex>, RequestError> {
    let rows = match raw.interactors {
        Some(rows) => rows,
        None => return Ok(None),
    };
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::ComplexPortal,
        record: crate::mapped::record(&raw.id),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let members = members(rows, &mut problems, &mut refusals)?;
    if !members.iter().any(|m| m.protein.as_ref() == Some(anchor)) {
        return Ok(None);
    }
    let header = ProteinComplexHeader {
        id: raw.id.clone(),
        name: crate::mapped::field_text(
            &mut problems.list,
            problems.source,
            problems.record.clone(),
            "name",
            raw.name,
        ),
        taxon: 9606,
        predicted: raw
            .predicted_complex
            .and_then(|v| problems.required("predictedComplex", v.value())),
        participant_total: members.len() as u64,
        matched_participant: crate::mapped::text(anchor.as_str())
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::ComplexPortal, Some(200)))?,
        url: crate::mapped::text(&format!(
            "https://www.ebi.ac.uk/complexportal/complex/{}",
            raw.id
        ))
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::ComplexPortal, Some(200)))?,
    };
    Ok(Some(Complex {
        refusals,
        header,
        members,
        problems: Some(problems.list),
    }))
}
