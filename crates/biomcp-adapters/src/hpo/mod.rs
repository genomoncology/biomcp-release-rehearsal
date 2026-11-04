//! Direct exact-term HPO service policy; no discovery or bundle endpoint.
use crate::disease_sources::components::HPO;
use biomcp_fetch::Input;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// The same publisher declaration governs direct terms and aggregator components.
/// Reply retrieval time never establishes an ontology version or release date.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Hpo,
    base: "https://ontology.jax.org",
    terms: Terms {
        name: HPO.name,
        url: HPO.url,
        usage: HPO.usage,
    },
    key: KeyUse::None,
    quota: &crate::quotas::HPO,
    deadline: Duration::from_secs(15),
};

mod raw;
use biodata::{Attribution, PhenotypeCard, PhenotypeSynonym, PhenotypeXref, Text};
use biomcp_fetch::{BadReply, Freshness, InputError, Leg, ReplyMetadata, RequestError};
use biomcp_types::{PhenotypeGet, PhenotypeSection, PhenotypeTermCard};
/// Exact native term and its original retrieval metadata.
#[derive(Debug)]
pub struct Record {
    /// Rejected native fields attributed to the exact term.
    pub problems: Vec<biomcp_types::Problem>,
    /// Complete native source card.
    pub card: PhenotypeTermCard,
    /// Checked retrieval metadata.
    pub metadata: ReplyMetadata,
    /// Unclassified source synonyms; missing remains unavailable.
    pub synonyms: Option<Box<[PhenotypeSynonym]>>,
    /// Source references, without an inferred predicate.
    pub xrefs: Option<Box<[PhenotypeXref]>>,
}
/// Read exactly one HPO identity without fallback or local term selection.
pub fn get(leg: &Leg<'_>, query: &PhenotypeGet) -> Result<Record, RequestError> {
    let endpoint = SPEC
        .get("/api/hp/terms")
        .with_segments(&[biomcp_fetch::PathSegment::new(query.id().as_str())?])
        .fresh(Freshness::Days(1));
    let (term, metadata) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Term>::json().check(&|term| {
                if &term.id == query.id() {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    map(term, metadata, query)
}
fn map(
    term: raw::Term,
    metadata: ReplyMetadata,
    query: &PhenotypeGet,
) -> Result<Record, RequestError> {
    let mut problems = crate::mapped::Problems {
        source: Source::Hpo,
        record: crate::mapped::record(&term.id),
        list: Vec::new(),
    };
    let label = problems.required_field("name", term.name);
    let definition = problems.field("definition", term.definition);
    let comment = problems.field("comment", term.comment);
    let publications = term
        .publications
        .map(|v| problems.fields("publicationReferences", Some(v)));
    let descendants = problems.field("descendantCount", term.descendants);
    let synonyms = term.synonyms.map(|v| problems.fields("synonyms", Some(v)));
    let xrefs = term.xrefs.map(|v| problems.fields("xrefs", Some(v)));
    let mut sections = Vec::new();
    if synonyms.is_some() {
        sections.push(PhenotypeSection::Synonyms);
    }
    if xrefs.is_some() {
        sections.push(PhenotypeSection::Xrefs);
    }
    let attribution = attribution(&term.id)?;
    let synonyms = synonyms.map(|rows| {
        rows.into_vec()
            .into_iter()
            .map(|value| PhenotypeSynonym {
                value,
                relationship: None,
                attribution: attribution.clone(),
            })
            .collect()
    });
    let xrefs = xrefs.map(|rows| {
        rows.into_vec()
            .into_iter()
            .map(|id| PhenotypeXref {
                id,
                predicate: None,
                attribution: attribution.clone(),
            })
            .collect()
    });
    Ok(Record {
        problems: problems.list,
        synonyms,
        xrefs,
        card: PhenotypeTermCard {
            term: PhenotypeCard::Hpo {
                requested_id: query.id().clone(),
                returned_id: term.id,
                label,
                definition,
                comment,
                publication_references: publications,
                hpo_descendant_count: descendants,
                attribution,
            },
            sections: sections.into(),
            components: Box::default(),
        },
        metadata,
    })
}
fn attribution(id: &biodata::HpId) -> Result<Attribution, RequestError> {
    let text = |s: &str| Text::new(s).map_err(|_| InputError::Serialization);
    Ok(Attribution {
        approved: None,
        published: None,
        source: text("hpo")?,
        record: Some(text(id.as_str())?),
        url: Some(text(&format!("https://hpo.jax.org/browse/term/{id}"))?),
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
