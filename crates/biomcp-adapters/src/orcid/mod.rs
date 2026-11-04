//! Exact public registry records with checked returned paths before cache admission.
use biomcp_fetch::Input;
mod raw;
mod works;
use biodata::{AuthorCard, OrcidId, OrcidWorkGroup};
use biomcp_fetch::{
    BadReply, Credential, Freshness, KeyPlace, KeyUse, Leg, RequestError, SecretName, SourceSpec,
};
use biomcp_types::Source;
use std::time::Duration;
/// Shared public person and works declaration.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Orcid,
    terms: biomcp_fetch::Terms {
        name: "ORCID Public API Terms of Service",
        url: "https://info.orcid.org/public-client-terms-of-service/",
        usage: biomcp_types::Use::Noncommercial,
    },
    base: "https://pub.orcid.org/v3.0",
    key: KeyUse::Optional(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::OrcidAccessToken,
        place: KeyPlace::Bearer,
    }),
    quota: &crate::quotas::ORCID,
    deadline: Duration::from_secs(25),
};
// The accepted whole-record capture recipe bound, also applied to cached decoding.
const MAX_BODY_BYTES: u64 = 8 * 1024 * 1024;
fn endpoint(id: &OrcidId, section: &str) -> Result<biomcp_fetch::Endpoint, RequestError> {
    Ok(SPEC
        .get("")
        .with_path(format!("/{id}/{section}"))?
        .bytes(MAX_BODY_BYTES)
        .fresh(Freshness::Days(1)))
}
/// Read only the public professional name from the requested person record.
pub fn card(
    leg: &Leg<'_>,
    id: &OrcidId,
) -> Result<crate::Mapped<Option<AuthorCard>>, RequestError> {
    let (record, _) = leg
        .call(
            &endpoint(id, "person")?,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Person>::json().check(&|r| {
                if r.path == format!("/{id}/person")
                    && r.name.as_ref().is_none_or(|n| n.path == id.to_string())
                {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    record
        .card(id)
        .map_err(|_| biomcp_fetch::bad_reply(Source::Orcid, None))
}
/// Read one whole bounded works reply and preserve groups and assertion order.
pub fn works(
    leg: &Leg<'_>,
    id: &OrcidId,
) -> Result<crate::Mapped<Box<[OrcidWorkGroup]>>, RequestError> {
    let (record, _) = leg
        .call(
            &endpoint(id, "works")?,
            Input::Params(&[]),
            biomcp_fetch::Read::<works::Works>::json().check(&|r| {
                if r.identity(id) {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(record.groups())
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
