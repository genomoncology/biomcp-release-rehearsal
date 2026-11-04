use super::{SPEC, map, raw};
use crate::Mapped;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{DiseaseCard, DiseaseGet};
/// The exact card and complete native section rows, sliced by the query layer.
#[derive(Debug)]
pub struct Record {
    /// Rejected optional card fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// Exact identity and native metadata.
    pub card: DiseaseCard,
    /// Every native synonym, with scope absent when the publisher omits it.
    pub synonyms: Mapped<Vec<biodata::DiseaseSynonym>>,
    /// Every cross-reference, without inferred equivalence.
    pub xrefs: Mapped<Vec<biodata::DiseaseRelation>>,
}
/// Request the exact CURIE and refuse wrong returned identities before cache admission.
pub fn get(leg: &Leg<'_>, query: &DiseaseGet) -> Result<Option<Record>, RequestError> {
    let ontology = crate::disease_sources::components::ontology(
        leg,
        query
            .ontology()
            .ok_or(biomcp_fetch::InputError::Serialization)?,
    )?;
    let endpoint = SPEC
        .get("/api/ontologies")
        .with_path(format!("/api/ontologies/{}/terms", ontology.as_str()))?
        .fresh(Freshness::Days(1));
    let (reply, _) = leg
        .call(
            &endpoint,
            Input::Params(&[("obo_id", Arg::Text(query.id().as_str()))]),
            Read::<raw::Exact>::json().check(&|reply| {
                if reply.embedded.terms.iter().any(|t| {
                    t.obo_id.to_string() != query.id().as_str()
                        || t.ontology_name.as_str() != ontology.as_str()
                }) {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )
        .map(|received| (received.value, received.meta))?;
    reply
        .embedded
        .terms
        .into_iter()
        .next()
        .map(|term| map::record(term, ontology))
        .transpose()
}
