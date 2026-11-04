//! Read-only ClinGen Allele Registry resolution, without local allele repair.
use biomcp_fetch::Input;
mod annotations;
mod map;
mod normalization;
mod raw;
use crate::quotas;
use biodata::Text;
use biomcp_fetch::{Arg, Endpoint, KeyUse, Leg, PathSegment, RequestError, SourceSpec};
use biomcp_types::{Source, VariantHeader};
use std::time::Duration;
/// Public GET access; raw output redistribution rights remain unverified.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "ClinGen Allele Registry service terms; output rights require review",
        url: "https://clinicalgenome.org/about/terms-of-use/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::AlleleRegistry,
    base: "https://reg.genome.network",
    key: KeyUse::None,
    quota: &quotas::ALLELE_REGISTRY,
    deadline: Duration::from_secs(15),
};
const ALLELE: Endpoint = SPEC
    .get("/allele")
    .json_rejections()
    .fresh(biomcp_fetch::Freshness::Days(1));
const FIELDS: &str = "none+@id+communityStandardTitle+genomicAlleles+transcriptAlleles.MANE+externalRecords.dbSNP+externalRecords.ClinVarVariations";
/// Checked service result including a source validation code.
#[derive(Debug)]
pub enum Resolution {
    /// Source-resolved canonical identity.
    Found(Box<crate::Mapped<VariantHeader>>),
    /// Source rejected the unchanged notation.
    Rejected(Text),
    /// A native record identity was refused with its scoped field problem.
    Refused(Vec<biomcp_types::Problem>),
}
/// Resolve unchanged HGVS through the declared GET endpoint.
pub fn hgvs(leg: &Leg<'_>, input: &str) -> Result<Resolution, RequestError> {
    Ok(map::allele(read(leg, input, false)?, None))
}
/// Read a canonical source record directly, without another resolver.
pub fn by_id(leg: &Leg<'_>, input: &str) -> Result<Resolution, RequestError> {
    Ok(map::allele(read(leg, input, true)?, Some(input)))
}
fn read(leg: &Leg<'_>, input: &str, by_id: bool) -> Result<raw::Allele, RequestError> {
    let endpoint = if by_id {
        ALLELE.clone().with_segments(&[PathSegment::new(input)?])
    } else {
        ALLELE.clone()
    };
    let mut params = vec![("fields", Arg::Text(FIELDS))];
    if !by_id {
        params.insert(0, ("hgvs", Arg::Text(input)));
    }
    let reply = leg
        .call(
            &endpoint,
            Input::Params(&params),
            biomcp_fetch::Read::<raw::Allele>::json().cache_if(&|r| {
                r.error_type.is_none() && r.identity(by_id.then_some(input)).is_ok()
            }),
        )?
        .value;
    Ok(reply)
}

/// Preserve the registry's own mapped forms and validation rejection.
pub fn normalize(
    leg: &Leg<'_>,
    input: &str,
    build: Option<biodata::Build>,
) -> Result<crate::Mapped<biomcp_types::NormalizationResult>, RequestError> {
    let qualified = qualify(input, build)?;
    let raw = read(leg, &qualified, false)?;
    Ok(normalization::map(raw))
}

// CAR GET accepts reference-qualified HGVS. Bind only the explicit chromosome alias
// through the shared reference table; leave the genomic description unchanged.
fn qualify(input: &str, build: Option<biodata::Build>) -> Result<String, RequestError> {
    let Some((chromosome, description)) = input
        .strip_prefix("chr")
        .and_then(|value| value.split_once(':'))
    else {
        return Ok(input.into());
    };
    let build = build.ok_or(biomcp_fetch::bad_reply(Source::AlleleRegistry, None))?;
    let chromosome = biodata::Chromosome::parse(if chromosome == "M" { "MT" } else { chromosome })
        .map_err(|_| biomcp_fetch::bad_reply(Source::AlleleRegistry, None))?;
    Ok(format!("{}:{description}", chromosome.accession(build)))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
