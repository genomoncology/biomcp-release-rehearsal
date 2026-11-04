//! Explicit Mutalyzer GET resolution; no inferred transcript or local correction.
use crate::{quotas, raw::Field};
use biodata::{Representation, Text, Variant};
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{BadReply, Endpoint, KeyUse, Leg, PathSegment, Reply, RequestError, SourceSpec};
use biomcp_types::{Source, SourceRecord, VariantHeader};
use serde::Deserialize;
mod normalization;
use std::time::Duration;
/// Service rights remain distinct from the software license.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "Mutalyzer service output terms require review",
        url: "https://mutalyzer.nl/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Mutalyzer,
    base: "https://mutalyzer.nl/api",
    key: KeyUse::None,
    quota: &quotas::MUTALYZER,
    deadline: Duration::from_secs(15),
};
const NORMALIZE: Endpoint = SPEC
    .get("/normalize")
    .json_rejections()
    .fresh(biomcp_fetch::Freshness::Days(1));
/// A source's exact outcome, never a local normalization.
#[derive(Debug)]
pub enum Resolution {
    /// Source supplied a normalized identity.
    Found(VariantHeader),
    /// Source validation codes reject the supplied input.
    Rejected(Vec<Text>),
    /// Native identity refused without rejecting a well-formed service reply.
    Refused(Vec<biomcp_types::Problem>),
}
type Descriptions = Field<Vec<Field<Description>>>;
#[derive(Deserialize)]
struct Normalized {
    custom: Option<Box<Normalized>>,
    corrected_description: Option<Field<biodata::SourceText>>,
    protein: Option<Field<Description>>,
    rna: Option<Field<Description>>,
    equivalent_descriptions: Option<Field<std::collections::BTreeMap<String, Descriptions>>>,
    back_translated_descriptions: Option<Field<Vec<Field<biodata::SourceText>>>>,
    infos: Option<Field<Vec<Field<Message>>>>,
    warnings: Option<Field<Vec<Field<Message>>>>,
    version: Option<Field<biodata::SourceText>>,
    input_description: Option<Field<Text>>,
    normalized_description: Option<Field<Text>>,
    errors: Option<Vec<Message>>,
}
#[derive(Deserialize)]
struct Message {
    code: Text,
    details: Option<Field<biodata::SourceText>>,
}
#[derive(Deserialize)]
struct Description {
    description: Field<biodata::SourceText>,
}
impl Normalized {
    fn payload(&self) -> &Self {
        self.custom.as_deref().unwrap_or(self)
    }
    fn errors(&self) -> &[Message] {
        self.payload().errors.as_deref().unwrap_or(&[])
    }
    fn identity(&self, input: &str) -> Result<Option<Text>, &'static str> {
        let raw = self.payload();
        if raw.input_description.is_some() || raw.errors().is_empty() {
            let given = raw
                .input_description
                .as_ref()
                .and_then(Field::as_ref)
                .ok_or("input_description")?;
            if given.as_str() != input {
                return Err("input_description");
            }
        }
        if !raw.errors().is_empty() {
            return Ok(None);
        }
        let normalized = raw
            .normalized_description
            .as_ref()
            .and_then(Field::as_ref)
            .ok_or("normalized_description")?;
        biodata::Expression::hgvs(normalized.clone()).map_err(|_| "normalized_description")?;
        Ok(Some(normalized.clone()))
    }
}
impl Reply for Normalized {
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if (200..300).contains(&status) || matches!(status, 400 | 422) && !self.errors().is_empty()
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
    fn check(&self) -> Result<(), BadReply> {
        if self.errors().len() > 1000
            || self.errors().iter().any(|error| {
                let code = error.code.as_str();
                code.len() > 128 || !code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            })
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// Route only the explicitly selected service, preserving the supplied description.
fn fetch(leg: &Leg<'_>, input: &str) -> Result<Normalized, RequestError> {
    let endpoint = NORMALIZE.clone().with_segments(&[PathSegment::new(input)?]);
    let mut raw = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<Normalized>::json()
                .cache_if(&|reply| reply.errors().is_empty() && reply.identity(input).is_ok()),
        )?
        .value;
    Ok(if let Some(custom) = raw.custom.take() {
        *custom
    } else {
        raw
    })
}
/// Route only the explicitly selected service, preserving the supplied description.
pub fn resolve(leg: &Leg<'_>, input: &str) -> Result<Resolution, RequestError> {
    let raw = fetch(leg, input)?;
    let identity = match raw.identity(input) {
        Ok(value) => value,
        Err(field) => {
            let mut problems = crate::mapped::Problems {
                source: Source::Mutalyzer,
                record: crate::mapped::record(input),
                list: Vec::new(),
            };
            problems.required::<()>(field, None);
            return Ok(Resolution::Refused(problems.list));
        }
    };
    if !raw.errors().is_empty() {
        return Ok(Resolution::Rejected(
            raw.errors.into_iter().flatten().map(|e| e.code).collect(),
        ));
    }
    let notation = identity.ok_or_else(|| biomcp_fetch::bad_reply(Source::Mutalyzer, None))?;
    let expression = biodata::Expression::hgvs(notation.clone())
        .map_err(|_| biomcp_fetch::bad_reply(Source::Mutalyzer, None))?;
    Ok(Resolution::Found(VariantHeader {
        identity: Variant {
            ids: Box::default(),
            representations: vec![Representation {
                source: Source::Mutalyzer.text(),
                expression,
                build: None,
                allele: None,
            }]
            .into(),
            genes: Box::default(),
        },
        source: Source::Mutalyzer,
        record: SourceRecord {
            namespace: biodata::IdentifierNamespace::Unknown,
            id: notation,
            url: None,
        },
        transcripts: Box::default(),
        myvariant: None,
        requested: None,
        sections: None,
        component_terms: Box::default(),
    }))
}

/// Source normalization descriptions, mappings and native messages.
pub use normalization::normalize;

/// Complete source policy and command declaration.
pub(crate) mod decl;
