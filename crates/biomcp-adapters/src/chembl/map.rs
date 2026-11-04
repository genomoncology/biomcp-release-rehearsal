//! Map each native relation independently from its descriptive metadata.
use super::raw::{Indication as RawIndication, Mechanism as RawMechanism, Reference};
use crate::mapped::Problems;
use biodata::{
    Attribution, DrugDiseaseReference, DrugIdentity, DrugIndicationClaim, DrugNativeReference,
    DrugNativeTarget, DrugStage, DrugTargetRelation, Error, Text,
};
/// Keep native target identities and disclose rejected relation metadata.
pub fn targets(
    drug: &DrugIdentity,
    rows: Vec<Mechanism>,
) -> Result<crate::Mapped<Box<[DrugTargetRelation]>>, Error> {
    let mut records = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mut rejected = Vec::new();
    for r in rows {
        let r = r.0;
        refusals.include(r.refusals);
        let target_identity = r.target_identity();
        let mut problems = Problems {
            source: biomcp_types::Source::Chembl,
            record: crate::mapped::record(r.mec_id),
            list: r.problems,
        };
        let source = Text::new("chembl")?;
        let target = problems
            .optional("target_chembl_id", Some(target_identity))
            .flatten()
            .map(|id| DrugNativeTarget {
                namespace: source,
                id,
                label: None,
                url: None,
            });
        if problems
            .required("target_chembl_id", target.as_ref())
            .is_none()
        {
            rejected.extend(problems.list);
            refusals = crate::Refusal::Refused;
            continue;
        }
        let relation = match r.action_type.as_deref() {
            Some(value) => problems.checked_text("action_type", Some(value)),
            None if !problems
                .list
                .iter()
                .any(|p| p.field.as_ref() == "action_type") =>
            {
                Some(Text::new("mechanism")?)
            }
            None => None,
        };
        let mechanism = problems.text("mechanism_of_action", r.mechanism_of_action.as_deref());
        let references = references(
            r.mechanism_refs,
            "mechanism_refs",
            &mut problems,
            &mut refusals,
        );
        records.push(DrugTargetRelation::new(
            drug.clone(),
            None,
            target,
            relation,
            mechanism,
            references,
            attribution(r.mec_id.to_string())?,
        )?);
        rejected.extend(problems.list);
    }
    Ok(crate::Mapped {
        refusals,
        value: records.into(),
        problems: rejected,
    })
}
/// Keep disease identities, labels and native stage strings independently.
pub fn indications(
    drug: &DrugIdentity,
    rows: Vec<Indication>,
) -> Result<crate::Mapped<Box<[DrugIndicationClaim]>>, Error> {
    let mut records = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mut rejected = Vec::new();
    for r in rows {
        let r = r.0;
        refusals.include(r.refusals);
        let mut problems = Problems {
            source: biomcp_types::Source::Chembl,
            record: crate::mapped::record(r.drugind_id),
            list: r.problems,
        };
        let mut diseases = Vec::new();
        let indication = if r.efo_term.is_some() {
            problems.text("efo_term", r.efo_term.as_deref())
        } else {
            problems.text("mesh_heading", r.mesh_heading.as_deref())
        };
        for (namespace, id_field, id, label_field, label) in [
            ("EFO", "efo_id", r.efo_id, "efo_term", r.efo_term),
            ("MeSH", "mesh_id", r.mesh_id, "mesh_heading", r.mesh_heading),
        ] {
            let label = problems.text(label_field, label.as_deref());
            if let Some(id) = problems.checked_text(id_field, id.as_deref()) {
                diseases.push(DrugDiseaseReference {
                    namespace: Text::new(namespace)?,
                    id,
                    label,
                });
            }
        }
        let references = references(
            r.indication_refs,
            "indication_refs",
            &mut problems,
            &mut refusals,
        );
        records.push(DrugIndicationClaim {
            drug: drug.clone(),
            indication,
            disease_references: diseases.into(),
            stage: r.max_phase_for_ind.map(|value| DrugStage {
                field: "max_phase_for_ind".into(),
                value: value.into(),
            }),
            references,
            population: None,
            combination: Box::default(),
            attribution: attribution(r.drugind_id.to_string())?,
        });
        rejected.extend(problems.list);
    }
    Ok(crate::Mapped {
        refusals,
        value: records.into(),
        problems: rejected,
    })
}
fn references(
    rows: Vec<Reference>,
    field: &str,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Box<[DrugNativeReference]> {
    let mut references = Vec::new();
    for r in rows {
        for problem in r.problems {
            crate::mapped::push_problem(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                &format!("{field}.{}", problem.field),
            );
        }
        let ref_type = problems.checked_text(&format!("{field}.ref_type"), r.ref_type.as_deref());
        if let Some(ref_type) = ref_type {
            references.push(DrugNativeReference {
                ref_type,
                ref_id: problems.checked_text(&format!("{field}.ref_id"), r.ref_id.as_deref()),
                ref_url: problems.checked_text(&format!("{field}.ref_url"), r.ref_url.as_deref()),
            });
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    references.into()
}
fn attribution(record: String) -> Result<Attribution, Error> {
    Ok(Attribution {
        approved: None,
        published: None,
        source: Text::new("chembl")?,
        record: Some(Text::new(&record)?),
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
        url: None,
    })
}

/// Accepted molecule storage with private provider fields.
#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct Molecule(pub(super) super::raw::Molecule);
/// Accepted mechanism storage with private provider fields.
#[derive(Debug)]
pub struct Mechanism(pub(super) RawMechanism);
/// Accepted indication storage with private provider fields.
#[derive(Debug)]
pub struct Indication(pub(super) RawIndication);
impl Molecule {
    /// Match complete native preferred or synonym names, retaining native trade-name classification.
    pub fn name_match(&self, name: &str) -> Option<biomcp_types::DrugNameMatch> {
        let exact = |value: &str| biomcp_types::DrugNameMatch::same_name(value, name);
        let brand = self.0.molecule_synonyms.iter().any(|s| {
            s.molecule_synonym.as_deref().is_some_and(exact)
                && s.syn_type.as_deref() == Some("TRADE_NAME")
                && !self.0.pref_name.as_deref().is_some_and(exact)
        });
        let generic = self
            .0
            .pref_name
            .as_deref()
            .is_some_and(|value| biomcp_types::DrugNameMatch::generic_name(value, name))
            || self.0.molecule_synonyms.iter().any(|s| {
                s.molecule_synonym
                    .as_deref()
                    .is_some_and(|value| biomcp_types::DrugNameMatch::generic_name(value, name))
                    && s.syn_type.as_deref() != Some("TRADE_NAME")
            });
        biomcp_types::DrugNameMatch::from_fields(brand, generic)
    }
    /// Actual refused native child identities.
    pub fn refusals(&self) -> crate::Refusal {
        self.0.refusals
    }
    /// Rejected native optional metadata.
    pub fn problems(&self) -> &[biomcp_types::Problem] {
        &self.0.problems
    }
}
impl Mechanism {
    /// Rejected native optional metadata.
    pub fn problems(&self) -> &[biomcp_types::Problem] {
        &self.0.problems
    }
    pub(super) fn target_identity(&self) -> Result<Option<biodata::Text>, Error> {
        self.0.target_identity()
    }
}
impl Indication {
    /// Rejected native optional metadata.
    pub fn problems(&self) -> &[biomcp_types::Problem] {
        &self.0.problems
    }
}
/// Map an admitted molecule identity and optional metadata.
pub fn molecule_header(row: &Molecule) -> Result<crate::Mapped<biomcp_types::DrugHeader>, Error> {
    super::molecule::header(&row.0)
}
/// Map an admitted molecule card and its original retrieval date.
pub fn molecule_card(
    row: Molecule,
    fetched: Option<i64>,
) -> Result<crate::Mapped<biomcp_types::DrugCard>, Error> {
    super::molecule::card(row.0, fetched)
}
