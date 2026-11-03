//! Keyed gene annotations, joined only by checked HGNC Entrez identity.
use biomcp_fetch::{Absent, Input};
mod narrative;
use crate::mapped::push_problem;
use crate::mapped::record as record_text;
mod raw;
pub mod variant;
mod variant_groups;
pub mod variant_map;
mod variant_map_support;
use crate::{Mapped, quotas};
use biodata::{GeneSymbol, NcbiGeneId, Text};

use biomcp_fetch::{
    Arg, Credential, Endpoint, Freshness, KeyPlace, KeyUse, Leg, RequestError, SecretName,
    SourceSpec,
};
use biomcp_types::{Oncokb, OncokbCard, Source};
use std::time::Duration;
/// Required-key source policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "OncoKB license",
        url: "https://faq.oncokb.org/licensing",
        usage: biomcp_types::Use::Licensed,
    },
    source: Source::Oncokb,
    base: "https://www.oncokb.org",
    key: KeyUse::Required(Credential {
        tier: biomcp_types::KeyTier::NoFallback,
        secret: SecretName::OncokbApiKey,
        place: KeyPlace::Bearer,
    }),
    quota: &quotas::ONCOKB,
    deadline: Duration::from_secs(8),
};
const GENES: Endpoint = SPEC
    .get("/api/v1/utils/allCuratedGenes")
    .fresh(Freshness::Days(1));
pub(crate) const INFO: Endpoint = SPEC.get("/api/v1/info").fresh(Freshness::Days(1));
/// One accepted gene reply, shared by its card and section.
#[derive(Debug)]
pub struct GeneData {
    /// Short card annotation.
    pub card: OncokbCard,
    /// Whole source section.
    pub section: Oncokb,
}
/// Fetch one case-sensitive HGNC symbol and accept only its checked Entrez identity.
pub fn gene(
    leg: &Leg<'_>,
    symbol: &GeneSymbol,
    id: NcbiGeneId,
) -> Result<Mapped<Option<GeneData>>, RequestError> {
    let reply = leg
        .call(
            &GENES,
            Input::Params(&[
                ("hugoSymbol", Arg::Text(symbol.as_str())),
                ("includeEvidence", Arg::Text("true")),
            ]),
            biomcp_fetch::Read::<raw::Genes>::json(),
        )
        .map(|received| received.value)
        .absent()?;
    let rows = reply.map(|reply| reply.0).unwrap_or_default();
    let mut problems = Vec::new();
    let mut accepted = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in rows {
        if let Some(value) = row.mapped(symbol, id, &mut problems) {
            accepted.push(value);
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    let value = if accepted.len() == 1 {
        accepted.pop()
    } else {
        if accepted.len() > 1 {
            refusals = crate::Refusal::Refused;
            push_problem(
                &mut problems,
                Source::Oncokb,
                record_text(id),
                "entrezGeneId",
            );
        }
        None
    };
    Ok(Mapped {
        refusals,
        value,
        problems,
    })
}
/// Read the data version once for the command's accepted gene data.
pub fn version(leg: &Leg<'_>) -> Result<Mapped<Option<Text>>, RequestError> {
    let info = leg
        .call(
            &INFO,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Info>::json(),
        )?
        .value;
    let mut cx = crate::mapped::Problems {
        source: Source::Oncokb,
        record: None,
        list: Vec::new(),
    };
    let value = cx
        .field("dataVersion", info.data_version)
        .and_then(|value| cx.field("dataVersion.version", value.version));
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: cx.list,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
