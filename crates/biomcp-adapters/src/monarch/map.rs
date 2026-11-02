use super::{components, raw};
use biodata::{
    Attribution, Curie, DiseaseAssociation, DiseaseAssociationQualifier,
    DiseaseAssociationReference, DiseaseEndpoint, Text,
};
use biomcp_fetch::{InputError, RequestError};
use biomcp_types::DiseaseAssociationItem;
pub(super) fn row(
    mut raw: raw::Row,
    requested: &Curie,
) -> Result<crate::Mapped<DiseaseAssociationItem>, RequestError> {
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Monarch,
        record: raw.id.clone(),
        list: std::mem::take(&mut raw.problems),
    };
    let qualifiers = qualifiers(&mut raw)?;
    let value = DiseaseAssociationItem {
        association: DiseaseAssociation {
            subject_taxon: raw.subject_taxon,
            requested_id: requested.clone(),
            subject: DiseaseEndpoint {
                id: raw.subject.ok_or(InputError::Serialization)?,
                label: raw.subject_label,
            },
            object: DiseaseEndpoint {
                id: raw.object.ok_or(InputError::Serialization)?,
                label: raw.object_label,
            },
            predicate: raw.predicate.ok_or(InputError::Serialization)?,
            original_subject: raw.original_subject,
            original_object: raw.original_object,
            attribution: Attribution {
                approved: None,
                published: None,
                source: Text::new("monarch").map_err(|_| InputError::Serialization)?,
                record: raw.id,
                url: raw
                    .provided_by_link
                    .and_then(|link| problems.field("provided_by_link.url", link.url)),
                version: None,
                release: None,
                submitter: None,
                evaluated: None,
            },
            primary_knowledge_source: raw.primary_knowledge_source,
            aggregator_knowledge_source: raw.aggregators.0,
            provided_by: raw.provided_by,
            evidence: if raw.has_evidence.is_some() || raw.has_evidence_links.is_some() {
                Some(references(
                    raw.has_evidence,
                    raw.has_evidence_links,
                    "has_evidence_links",
                    &mut problems,
                ))
            } else {
                None
            },
            publications: references(
                raw.publications,
                raw.publications_links,
                "publications_links",
                &mut problems,
            ),
            qualifiers: qualifiers.into(),
            original_predicate: raw.original_predicate,
            category: raw.category,
            agent_type: raw.agent_type,
            knowledge_level: raw.knowledge_level,
            evidence_count: raw.evidence_count,
            negated: raw.negated,
            has_count: raw.has_count,
            has_total: raw.has_total,
            has_percentage: raw.has_percentage,
            has_quotient: raw.has_quotient,
            subject_closure: raw.subject_closure.unwrap_or_default(),
            object_closure: raw.object_closure.unwrap_or_default(),
        },
        components: components()?,
    };
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
fn qualifier(
    field: &str,
    id: Curie,
    label: Option<Text>,
) -> Result<DiseaseAssociationQualifier, RequestError> {
    Ok(DiseaseAssociationQualifier {
        field: Text::new(field).map_err(|_| InputError::Serialization)?,
        id,
        label,
    })
}
fn references(
    ids: Option<Box<[Curie]>>,
    links: Option<Box<[raw::Reference]>>,
    field: &str,
    problems: &mut crate::mapped::Problems,
) -> Box<[DiseaseAssociationReference]> {
    let mut links = links.unwrap_or_default().into_vec();
    let mut references = Vec::new();
    for id in ids.unwrap_or_default() {
        let link = links
            .iter()
            .position(|link| link.id == id)
            .map(|index| links.remove(index));
        references.push(reference(id, link, field, problems));
    }
    references.extend(
        links
            .into_iter()
            .map(|link| reference(link.id.clone(), Some(link), field, problems)),
    );
    references.into()
}

fn reference(
    id: Curie,
    link: Option<raw::Reference>,
    field: &str,
    problems: &mut crate::mapped::Problems,
) -> DiseaseAssociationReference {
    let (label, url) = match link {
        Some(link) => (
            problems
                .field(&format!("{field}.label"), link.label)
                .map(Into::into),
            problems.field(&format!("{field}.url"), link.url),
        ),
        None => (None, None),
    };
    DiseaseAssociationReference { id, label, url }
}

fn qualifiers(raw: &mut raw::Row) -> Result<Vec<DiseaseAssociationQualifier>, RequestError> {
    let mut qualifiers = raw
        .qualifiers
        .take()
        .unwrap_or_default()
        .into_vec()
        .into_iter()
        .map(|id| qualifier("qualifiers", id, None))
        .collect::<Result<Vec<_>, _>>()?;
    for (field, id, label) in [
        (
            "frequency_qualifier",
            raw.frequency_qualifier.take(),
            raw.frequency_qualifier_label.take(),
        ),
        (
            "onset_qualifier",
            raw.onset_qualifier.take(),
            raw.onset_qualifier_label.take(),
        ),
        (
            "sex_qualifier",
            raw.sex_qualifier.take(),
            raw.sex_qualifier_label.take(),
        ),
        (
            "disease_context_qualifier",
            raw.disease_context_qualifier.take(),
            raw.disease_context_qualifier_label.take(),
        ),
        (
            "species_context_qualifier",
            raw.species_context_qualifier.take(),
            raw.species_context_qualifier_label.take(),
        ),
        (
            "stage_qualifier",
            raw.stage_qualifier.take(),
            raw.stage_qualifier_label.take(),
        ),
    ] {
        if let Some(id) = id {
            qualifiers.push(qualifier(field, id, label.map(Into::into))?);
        }
    }
    Ok(qualifiers)
}
