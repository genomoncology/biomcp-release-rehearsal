//! Canonical mydisease publisher declaration.
use biomcp_fetch::Input;
mod map;
mod native;
mod raw;
use biodata::Curie;
use biomcp_fetch::{Arg, BadReply, Freshness, InputError, Leg, PathSegment, RequestError};
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
pub use native::{Component, Record, Reference, Value};
/// Complete exact annotation before independently bounded response pages.
#[derive(Debug)]
pub struct Annotation {
    /// Exact identity with optional native alias proof.
    pub card: biomcp_types::DiseaseCard,
    /// Component-attributed native synonyms.
    pub synonyms: Vec<biodata::DiseaseSynonym>,
    /// Every native cross-reference with its original source field predicate.
    pub xrefs: Vec<biodata::DiseaseRelation>,
    /// Direct component parents, without ancestors or alternatives.
    pub parents: Vec<biodata::DiseaseRelation>,
}
/// Read and map one exact annotation without additional source requests.
pub fn annotation(leg: &Leg<'_>, id: &Curie) -> Result<crate::Mapped<Annotation>, RequestError> {
    map::annotation(get(leg, id)?)
}
use std::time::Duration;
/// Publisher policy shared by health, admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::MyDisease,
    base: "https://mydisease.info/v1",
    terms: Terms {
        name: "MyDisease source-specific terms",
        url: "https://docs.mydisease.info/en/latest/doc/data.html",
        usage: Use::Attribution,
    },
    quota: &crate::quotas::BIOTHINGS,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};
const GET: biomcp_fetch::Endpoint = SPEC.get("/disease").fresh(Freshness::Days(7));
const FIELDS: &str = "mondo,disease_ontology";
/// Whether an identifier belongs to the publisher's implemented exact namespaces.
pub fn supported_id(id: &str) -> bool {
    id.split_once(':').is_some_and(|(namespace, local)| {
        matches!(namespace, "MONDO" | "DOID")
            && !local.is_empty()
            && local.bytes().all(|v| v.is_ascii_digit())
    })
}

/// Read one exact MONDO or DOID annotation, with no search, fallback or fanout.
pub fn get(leg: &Leg<'_>, id: &Curie) -> Result<Record, RequestError> {
    let identifier = id.to_string();
    if !supported_id(&identifier) {
        return Err(InputError::Path.into());
    }
    let fields = crate::disease_sources::components::allowed_fields(leg, FIELDS)?;
    let endpoint = GET.clone().with_segments(&[PathSegment::new(&identifier)?]);
    let (raw, metadata) = leg
        .call(
            &endpoint,
            Input::Params(&[("fields", Arg::Text(&fields))]),
            biomcp_fetch::Read::<raw::Document>::json().check(&|record| {
                if record.matches(id, &fields) {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    // Ignore any unrequested component even when an old replay contains it.
    let mut components = Vec::new();
    let mut problems = crate::mapped::Problems {
        source: Source::MyDisease,
        record: crate::mapped::record(&raw.id),
        list: Vec::new(),
    };
    for (field, name, value, prefix) in [
        ("mondo", "mondo", raw.mondo, "MONDO:"),
        ("disease_ontology", "doid", raw.disease_ontology, "DOID:"),
    ] {
        if !fields.split(',').any(|f| f == field) {
            continue;
        }
        if let Some(value) = problems.field(field, value) {
            if value.id.to_string().starts_with(prefix) {
                components.push(native::component(name, value));
            } else {
                problems.required::<()>(&format!("{field}.id"), None);
            }
        }
    }
    let version = problems.field("_version", raw.version);
    Ok(Record {
        problems: problems.list,
        requested_id: id.clone(),
        returned_id: raw.id,
        version,
        components: components.into(),
        metadata,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
