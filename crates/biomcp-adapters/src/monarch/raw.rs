use crate::native_text::NativeText;
use biodata::{Curie, FiniteNumber, Text};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Page {
    pub limit: u32,
    pub offset: u32,
    pub total: u64,
    pub items: Vec<Row>,
}
impl Page {
    pub(super) fn identities_admitted(&self) -> bool {
        let accepted = self
            .items
            .iter()
            .filter(|row| row.identity_admitted())
            .count();
        let refusals = if accepted < self.items.len() {
            crate::Refusal::Refused
        } else {
            crate::Refusal::None
        };
        refusals.identities_admitted(accepted, self.items.len())
    }
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.items.len() > self.limit as usize
            || (!self.items.is_empty()
                && u64::from(self.offset) + self.items.len() as u64 > self.total)
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedRow")]
pub(super) struct Row {
    pub problems: Vec<biomcp_types::Problem>,
    pub id: Option<Text>,
    pub subject: Option<Curie>,
    pub subject_taxon: Option<Curie>,
    pub subject_label: Option<biodata::Text>,
    pub predicate: Option<Curie>,
    pub object: Option<Curie>,
    pub object_label: Option<biodata::Text>,
    pub original_subject: Option<Curie>,
    pub original_object: Option<Curie>,
    pub primary_knowledge_source: Option<Curie>,
    pub aggregators: super::presence::Aggregators,
    pub provided_by: Option<Text>,
    pub provided_by_link: Option<Link>,
    pub has_evidence: Option<Box<[Curie]>>,
    pub has_evidence_links: Option<Box<[Reference]>>,
    pub publications: Option<Box<[Curie]>>,
    pub publications_links: Option<Box<[Reference]>>,
    pub original_predicate: Option<Text>,
    pub category: Option<Curie>,
    pub agent_type: Option<Text>,
    pub knowledge_level: Option<Text>,
    pub evidence_count: Option<u64>,
    pub qualifiers: Option<Box<[Curie]>>,
    pub frequency_qualifier: Option<Curie>,
    pub frequency_qualifier_label: Option<NativeText>,
    pub onset_qualifier: Option<Curie>,
    pub onset_qualifier_label: Option<NativeText>,
    pub sex_qualifier: Option<Curie>,
    pub sex_qualifier_label: Option<NativeText>,
    pub disease_context_qualifier: Option<Curie>,
    pub disease_context_qualifier_label: Option<NativeText>,
    pub species_context_qualifier: Option<Curie>,
    pub species_context_qualifier_label: Option<NativeText>,
    pub stage_qualifier: Option<Curie>,
    pub stage_qualifier_label: Option<NativeText>,
    pub negated: Option<bool>,
    pub has_count: Option<u64>,
    pub has_total: Option<u64>,
    pub has_percentage: Option<FiniteNumber>,
    pub has_quotient: Option<FiniteNumber>,
    pub subject_closure: Option<Box<[Curie]>>,
    pub object_closure: Option<Box<[Curie]>>,
}
impl Row {
    pub(super) fn identity_admitted(&self) -> bool {
        self.id.is_some()
            && self.subject.is_some()
            && self.object.is_some()
            && self.predicate.is_some()
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Link {
    pub url: Option<crate::raw::Field<Text>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Reference {
    pub id: Curie,
    pub label: Option<crate::raw::Field<NativeText>>,
    pub url: Option<crate::raw::Field<Text>>,
}
