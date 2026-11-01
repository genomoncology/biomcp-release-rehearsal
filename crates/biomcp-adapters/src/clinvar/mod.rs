//! ClinVar VCV XML by a checked Variation ID, without text-search identity.
use biomcp_fetch::{Input, Read};
mod detail;
mod map;
mod raw;
mod search;
mod subject;
use crate::quotas;
use biomcp_fetch::{
    Arg, Credential, Endpoint, Freshness, KeyPlace, KeyUse, Leg, RequestError, SecretName,
    SourceSpec,
};
use biomcp_types::{Source, VariantAssertion};
pub use search::{SearchPage, search};
use std::time::Duration;
/// ClinVar shares the existing NCBI quota and optional user key.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "NCBI policy; submission rights remain source-specific",
        url: "https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/",
        usage: biomcp_types::Use::Restricted,
    },
    source: Source::ClinVar,
    base: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils",
    key: KeyUse::Optional(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NcbiApiKey,
        place: KeyPlace::Query("api_key"),
    }),
    quota: &quotas::NCBI,
    deadline: Duration::from_secs(15),
};
const FETCH: Endpoint = SPEC.get("/efetch.fcgi").fresh(Freshness::Days(1));
/// Fetch exactly one Variation ID and validate the XML identity before cache admission.
pub fn assertions(
    leg: &Leg<'_>,
    id: &str,
    selection: Option<(&biomcp_types::AssertionId, Option<&str>)>,
) -> Result<crate::Mapped<Vec<VariantAssertion>>, RequestError> {
    let vcv = selection
        .filter(|(id, _)| matches!(id, biomcp_types::AssertionId::Vcv(_)))
        .map(|(id, version)| (id.as_str(), version));
    let selected = vcv.map(|(record, version)| {
        version.map_or_else(|| record.to_owned(), |v| format!("{record}.{v}"))
    });
    let mut params = vec![
        ("db", Arg::Text("clinvar")),
        ("id", Arg::Text(selected.as_deref().unwrap_or(id))),
        ("rettype", Arg::Text("vcv")),
    ];
    if vcv.is_none() {
        params.push(("is_variationid", Arg::Text("true")));
    }
    params.extend([("retmode", Arg::Text("xml")), ("tool", Arg::Text("biomcp"))]);
    let (mut reply, _) = leg
        .call(
            &FETCH,
            Input::Params(&params),
            Read::<raw::Records>::xml()
                .cache_if(&|reply| reply.identity_problem(id, vcv).is_none()),
        )
        .map(|received| (received.value, received.meta))?;
    if let Some(field) = reply.identity_problem(id, vcv) {
        reply.refusals = crate::Refusal::Refused;
        reply.rows.clear();
        if !reply
            .problems
            .iter()
            .any(|problem| problem.field.as_ref() == field)
        {
            reply.problems.push(crate::mapped::problem(
                Source::ClinVar,
                crate::mapped::record(id),
                field,
            ));
        }
    }
    Ok(crate::Mapped {
        refusals: reply.refusals,
        value: reply.rows,
        problems: reply.problems,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
