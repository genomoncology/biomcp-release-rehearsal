//! Human Protein Atlas gene-level RNA expression and subcellular annotations.
use biomcp_fetch::{Absent, Input, Read};
mod map;
mod raw;
use crate::{Mapped, quotas};
use biodata::EnsemblGeneId;
use biomcp_fetch::{Freshness, KeyUse, Leg, RequestError, SourceError, SourceSpec};
use biomcp_types::{GeneExpression, Reason, Source};
use std::time::Duration;
/// HPA source policy with a conservative local quota.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC BY 4.0; third-party conditions",
        url: "https://www.proteinatlas.org/about/licence",
        usage: biomcp_types::Use::Attribution,
    },
    source: Source::Hpa,
    base: "https://www.proteinatlas.org",
    key: KeyUse::None,
    quota: &quotas::HPA,
    deadline: Duration::from_secs(8),
};
/// Fetch one HGNC Ensembl identity, preserving the source's own miss.
pub fn expression(
    leg: &Leg<'_>,
    id: &EnsemblGeneId,
) -> Result<Option<Mapped<GeneExpression>>, RequestError> {
    let endpoint = SPEC
        .get("/")
        .fresh(Freshness::Days(7))
        .with_path(format!("/{}.json", id.base()))?;
    let raw = leg
        .call(&endpoint, Input::Params(&[]), Read::<raw::Gene>::json())
        .map(|received| received.value)
        .absent()?;
    raw.map(|raw| {
        if raw.ensembl != id.base() {
            return Err(SourceError {
                source: Source::Hpa,
                reason: Reason::BadReply,
                status: None,
                tls_cause: None,
                retry_after: None,
            }
            .into());
        }
        Ok(map::expression(raw, id))
    })
    .transpose()
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
