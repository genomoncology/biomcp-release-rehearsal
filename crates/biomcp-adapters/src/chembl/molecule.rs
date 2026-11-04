//! Map molecule metadata without changing native identities or stage strings.
use super::raw::Molecule;
use crate::mapped::Problems;
use biodata::{Attribution, DrugIdentity, DrugIdentityLevel, DrugIdentityRelation, Error, Text};
use biomcp_types::{DrugCard, DrugHeader, DrugSection, DrugSource};
/// Map one checked native molecule identity with independent metadata problems.
pub(super) fn header(row: &Molecule) -> Result<crate::Mapped<DrugHeader>, Error> {
    let mut problems = Problems {
        source: biomcp_types::Source::Chembl,
        record: crate::mapped::record(&row.molecule_chembl_id),
        list: row.problems.clone(),
    };
    let value = DrugHeader {
        id: row.molecule_chembl_id.parse()?,
        source: DrugSource::Chembl,
        name_resolution: None,
        matched_name: None,
        name: problems.text("pref_name", row.pref_name.as_deref()),
        r#type: problems.checked_text("molecule_type", row.molecule_type.as_deref()),
        version: None,
        identity_level: DrugIdentityLevel::Unknown,
        url: Some(Text::new(&format!(
            "https://www.ebi.ac.uk/chembl/explore/compound/{}",
            row.molecule_chembl_id
        ))?),
    };
    Ok(crate::Mapped {
        refusals: row.refusals,
        value,
        problems: problems.list,
    })
}
/// Map synonyms and source relations without failing unrelated molecule fields.
pub(super) fn card(row: Molecule, fetched: Option<i64>) -> Result<crate::Mapped<DrugCard>, Error> {
    let header = header(&row)?;
    let mut problems = Problems {
        source: biomcp_types::Source::Chembl,
        record: crate::mapped::record(&row.molecule_chembl_id),
        list: header.problems,
    };
    let attribution = Attribution {
        approved: None,
        published: None,
        source: Text::new("chembl")?,
        record: problems.record.clone(),
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
        url: header.value.url.clone(),
    };
    let mut refusals = header.refusals;
    let names = names(row.molecule_synonyms, &mut problems, &mut refusals);
    let references = cross_references(
        row.cross_references,
        &attribution,
        &mut problems,
        &mut refusals,
    )?;
    let mut parents = Vec::new();
    if let Some(parent) = row.molecule_hierarchy {
        if let Some(id) = problems.optional(
            "molecule_hierarchy.parent_chembl_id",
            Some(parent.parent_chembl_id.parse()),
        ) {
            parents.push(DrugIdentityRelation {
                identity: DrugIdentity {
                    id,
                    namespace: Text::new("chembl")?,
                    name: None,
                    identity_level: DrugIdentityLevel::Unknown,
                    r#type: None,
                    url: None,
                },
                relation: Text::new("molecule_hierarchy.parent_chembl_id")?,
                attribution,
            });
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    let mut sections = vec![DrugSection::Targets, DrugSection::Indications];
    if !references.is_empty() {
        sections.push(DrugSection::Xrefs);
    }
    let value = DrugCard {
        header: header.value,
        alternate_names: names.into(),
        cross_references: references.into(),
        parents: parents.into(),
        products: Box::default(),
        formulations: Box::default(),
        version: None,
        updated: None,
        retrieved: crate::drug_sources::received_date(fetched),
        available_sections: sections.into(),
        next: Box::default(),
    };
    Ok(crate::Mapped {
        refusals,
        value,
        problems: problems.list,
    })
}

fn cross_references(
    rows: Vec<super::raw::CrossReference>,
    attribution: &Attribution,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Result<Vec<DrugIdentityRelation>, Error> {
    let mut references = Vec::new();
    for r in rows {
        for issue in r.problems {
            crate::mapped::push_problem(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                &format!("cross_references.{}", issue.field),
            );
        }
        let id = problems.optional("cross_references.xref_id", Some(r.xref_id.parse()));
        let source = problems.checked_text("cross_references.xref_src", Some(&r.xref_src));
        let name = problems.text("cross_references.xref_name", r.xref_name.as_deref());
        if let (Some(id), Some(source)) = (id, source) {
            references.push(DrugIdentityRelation {
                identity: DrugIdentity {
                    id,
                    namespace: source,
                    name,
                    identity_level: DrugIdentityLevel::Unknown,
                    r#type: None,
                    url: None,
                },
                relation: Text::new("cross_references")?,
                attribution: attribution.clone(),
            });
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    Ok(references)
}

fn names(
    rows: Vec<super::raw::Synonym>,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Vec<Text> {
    let mut names = Vec::new();
    for synonym in rows {
        for issue in synonym.problems {
            crate::mapped::push_problem(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                &format!("molecule_synonyms.{}", issue.field),
            );
        }
        if let Some(name) = problems.text(
            "molecule_synonyms.molecule_synonym",
            synonym.molecule_synonym.as_deref(),
        ) {
            if !names.contains(&name) {
                names.push(name);
            }
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    names
}
