//! Explicit VariantValidator GET reads; warning envelopes never resolve identity.
use crate::quotas;
use biodata::Build;
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{Arg, Endpoint, KeyUse, Leg, PathSegment, RequestError, SourceSpec};
use biomcp_types::{Source, VariantHeader};
mod map;
mod normalization;
mod raw;
use raw::Response;
use std::time::Duration;
/// Service terms are independent of the implementation's software license.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "Citation requested; commercial use of results not addressed",
        url: "https://variantvalidator.org/help/about/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::VariantValidator,
    base: "https://rest.variantvalidator.org/VariantValidator",
    key: KeyUse::None,
    quota: &quotas::VARIANT_VALIDATOR,
    deadline: Duration::from_secs(15),
};
const VALIDATE: Endpoint = SPEC
    .get("/variantvalidator")
    .json_rejections()
    .fresh(biomcp_fetch::Freshness::Days(1));
/// Explicit service response, without a local consensus or preferred transcript.
#[derive(Debug)]
pub struct Resolution {
    /// Actual native transcript records refused at their identity boundary.
    pub refusals: crate::Refusal,
    /// Warning flag refuses resolution even on HTTP 200.
    pub rejected: bool,
    /// Every source identity returned by the selected service.
    pub candidates: Vec<VariantHeader>,
    /// Rejected native fields and record identities.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Preserve caller assembly, unchanged input and every service-returned identity.
fn fetch(leg: &Leg<'_>, input: &str, build: Build) -> Result<Response, RequestError> {
    let build = build.to_string();
    let endpoint = VALIDATE.clone().with_segments(&[
        PathSegment::new(&build)?,
        PathSegment::new(input)?,
        PathSegment::new("all")?,
    ]);
    let raw = leg
        .call(
            &endpoint,
            Input::Params(&[("content-type", Arg::Text("application/json"))]),
            Read::<Response>::json()
                .check(&|_| Ok(()))
                .cache_if(&|reply| reply.flag.as_str() != "warning" && reply.eligible(input)),
        )?
        .value;
    Ok(raw)
}
/// Preserve caller assembly, unchanged input and every service-returned identity.
pub fn resolve(leg: &Leg<'_>, input: &str, build: Build) -> Result<Resolution, RequestError> {
    Ok(map::resolve(fetch(leg, input, build)?, input))
}

/// Native service descriptions, warnings and version metadata.
pub use normalization::normalize;

/// Complete source policy and command declaration.
pub(crate) mod decl;
