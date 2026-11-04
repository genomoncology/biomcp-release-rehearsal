//! Admit association metadata independently from row identities.
use super::raw::{Link, Reference};
use crate::native_text::NativeText;
use biodata::{Curie, FiniteNumber, Text};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct ReceivedRow {
    pub id: Option<crate::raw::Field<Text>>,
    pub subject: Option<crate::raw::Field<Curie>>,
    pub subject_taxon: Option<crate::raw::Field<Curie>>,
    pub subject_label: Option<crate::raw::Field<NativeText>>,
    pub predicate: Option<crate::raw::Field<Curie>>,
    pub object: Option<crate::raw::Field<Curie>>,
    pub object_label: Option<crate::raw::Field<NativeText>>,
    pub original_subject: Option<crate::raw::Field<Curie>>,
    pub original_object: Option<crate::raw::Field<Curie>>,
    pub primary_knowledge_source: Option<crate::raw::Field<Curie>>,
    #[serde(flatten)]
    pub aggregators: super::presence::ReceivedAggregators,
    pub provided_by: Option<crate::raw::Field<Text>>,
    pub provided_by_link: Option<crate::raw::Field<Link>>,
    pub has_evidence: Option<crate::raw::Field<Vec<crate::raw::Field<Curie>>>>,
    pub has_evidence_links: Option<crate::raw::Field<Vec<crate::raw::Field<Reference>>>>,
    pub publications: Option<crate::raw::Field<Vec<crate::raw::Field<Curie>>>>,
    pub publications_links: Option<crate::raw::Field<Vec<crate::raw::Field<Reference>>>>,
    pub original_predicate: Option<crate::raw::Field<Text>>,
    pub category: Option<crate::raw::Field<Curie>>,
    pub agent_type: Option<crate::raw::Field<Text>>,
    pub knowledge_level: Option<crate::raw::Field<Text>>,
    pub evidence_count: Option<crate::raw::Field<u64>>,
    pub qualifiers: Option<crate::raw::Field<Vec<crate::raw::Field<Curie>>>>,
    pub frequency_qualifier: Option<crate::raw::Field<Curie>>,
    pub frequency_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub onset_qualifier: Option<crate::raw::Field<Curie>>,
    pub onset_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub sex_qualifier: Option<crate::raw::Field<Curie>>,
    pub sex_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub disease_context_qualifier: Option<crate::raw::Field<Curie>>,
    pub disease_context_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub species_context_qualifier: Option<crate::raw::Field<Curie>>,
    pub species_context_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub stage_qualifier: Option<crate::raw::Field<Curie>>,
    pub stage_qualifier_label: Option<crate::raw::Field<NativeText>>,
    pub negated: Option<crate::raw::Field<bool>>,
    pub has_count: Option<crate::raw::Field<u64>>,
    pub has_total: Option<crate::raw::Field<u64>>,
    pub has_percentage: Option<crate::raw::Field<FiniteNumber>>,
    pub has_quotient: Option<crate::raw::Field<FiniteNumber>>,
    pub subject_closure: Option<crate::raw::Field<Vec<crate::raw::Field<Curie>>>>,
    pub object_closure: Option<crate::raw::Field<Vec<crate::raw::Field<Curie>>>>,
}
impl From<ReceivedRow> for super::raw::Row {
    fn from(row: ReceivedRow) -> Self {
        let mut problems = fields(&row);
        let aggregators = row
            .aggregators
            .0
            .map(|value| value.map(|v| problems.fields("aggregator_knowledge_source", Some(v))));
        Self {
            aggregators: super::presence::Aggregators(aggregators),
            id: problems.required_field("id", row.id),
            subject: problems.required_field("subject", row.subject),
            subject_taxon: problems.field("subject_taxon", row.subject_taxon),
            subject_label: problems
                .required_field("subject_label", row.subject_label)
                .map(Into::into),
            predicate: problems.required_field("predicate", row.predicate),
            object: problems.required_field("object", row.object),
            object_label: problems
                .required_field("object_label", row.object_label)
                .map(Into::into),
            original_subject: problems.field("original_subject", row.original_subject),
            original_object: problems.field("original_object", row.original_object),
            primary_knowledge_source: problems
                .field("primary_knowledge_source", row.primary_knowledge_source),
            provided_by: problems.field("provided_by", row.provided_by),
            provided_by_link: problems.field("provided_by_link", row.provided_by_link),
            has_evidence: row
                .has_evidence
                .map(|v| problems.fields("has_evidence", Some(v))),
            has_evidence_links: row
                .has_evidence_links
                .map(|v| problems.fields("has_evidence_links", Some(v))),
            publications: row
                .publications
                .map(|v| problems.fields("publications", Some(v))),
            publications_links: row
                .publications_links
                .map(|v| problems.fields("publications_links", Some(v))),
            original_predicate: problems.field("original_predicate", row.original_predicate),
            category: problems.field("category", row.category),
            agent_type: problems.field("agent_type", row.agent_type),
            knowledge_level: problems.field("knowledge_level", row.knowledge_level),
            evidence_count: problems.field("evidence_count", row.evidence_count),
            qualifiers: row
                .qualifiers
                .map(|v| problems.fields("qualifiers", Some(v))),
            frequency_qualifier: problems.field("frequency_qualifier", row.frequency_qualifier),
            frequency_qualifier_label: problems
                .field("frequency_qualifier_label", row.frequency_qualifier_label),
            onset_qualifier: problems.field("onset_qualifier", row.onset_qualifier),
            onset_qualifier_label: problems
                .field("onset_qualifier_label", row.onset_qualifier_label),
            sex_qualifier: problems.field("sex_qualifier", row.sex_qualifier),
            sex_qualifier_label: problems.field("sex_qualifier_label", row.sex_qualifier_label),
            disease_context_qualifier: problems
                .field("disease_context_qualifier", row.disease_context_qualifier),
            disease_context_qualifier_label: problems.field(
                "disease_context_qualifier_label",
                row.disease_context_qualifier_label,
            ),
            species_context_qualifier: problems
                .field("species_context_qualifier", row.species_context_qualifier),
            species_context_qualifier_label: problems.field(
                "species_context_qualifier_label",
                row.species_context_qualifier_label,
            ),
            stage_qualifier: problems.field("stage_qualifier", row.stage_qualifier),
            stage_qualifier_label: problems
                .field("stage_qualifier_label", row.stage_qualifier_label),
            negated: problems.field("negated", row.negated),
            has_count: problems.field("has_count", row.has_count),
            has_total: problems.field("has_total", row.has_total),
            has_percentage: problems.field("has_percentage", row.has_percentage),
            has_quotient: problems.field("has_quotient", row.has_quotient),
            subject_closure: row
                .subject_closure
                .map(|v| problems.fields("subject_closure", Some(v))),
            object_closure: row
                .object_closure
                .map(|v| problems.fields("object_closure", Some(v))),
            problems: problems.list,
        }
    }
}

fn fields(row: &ReceivedRow) -> crate::mapped::Problems {
    crate::mapped::Problems {
        source: biomcp_types::Source::Monarch,
        record: row.id.as_ref().and_then(crate::raw::Field::as_ref).cloned(),
        list: Vec::new(),
    }
}
