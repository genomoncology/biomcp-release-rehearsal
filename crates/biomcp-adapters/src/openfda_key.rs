//! openFDA authentication applies only to adapters declaring https://api.fda.gov.
use biomcp_fetch::{Credential, KeyPlace, KeyUse, SecretName};

// https://open.fda.gov/apis/authentication/ documents the api_key query parameter.
pub(crate) const OPTIONAL: KeyUse = KeyUse::Optional(Credential {
    tier: biomcp_types::KeyTier::Fallback,
    secret: SecretName::OpenfdaApiKey,
    place: KeyPlace::Query("api_key"),
});
