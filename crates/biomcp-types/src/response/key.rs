use serde::{Deserialize, Serialize};
/// Configuration names for one keyed source; never includes credential values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KeyHelp {
    /// Optional source-specific key guidance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<crate::Text>,
    /// Source name.
    pub source: crate::Source,
    /// Configuration file key path.
    pub config: crate::Text,
    /// Environment variable name.
    pub environment: crate::Text,
}
impl super::ResponseBody for KeyHelp {
    const SCHEMA: &'static str = "biomcp.key/5";
    type Query = super::EmptyQuery;
}

biodata::wire_enum! {
    /// Where the command will obtain a credential; never its value.
    KeyState {
        /// Configuration file credential.
        Stored => "stored",
        /// Environment fallback credential.
        Environment => "environment",
        /// No credential configured.
        None => "none"
    }
}
biodata::wire_enum! {
    /// Whether hosted calls may use the server's configured source credential.
    KeyTier {
        /// Only the caller may supply this source credential.
        NoFallback => "no-fallback",
        /// Use the server credential when the caller supplies none.
        Fallback => "fallback"
    }
}
/// Credential identity and redacted presence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RegisteredKey {
    /// Source accepted by key set.
    pub source: crate::Text,
    /// Configuration name.
    pub config: crate::Text,
    /// Environment fallback name.
    pub environment: crate::Text,
    /// Header accepted by the hosted HTTP transport.
    pub header: crate::Text,
    /// Declared server credential fallback policy.
    pub tier: KeyTier,
    /// Publisher use label for the whole source.
    pub usage: crate::Use,
    /// Current credential origin.
    pub state: KeyState,
}
/// Results of human credential registration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RegisteredKeys {
    /// Redacted credential identities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<RegisteredKey>,
}
impl super::ResponseBody for RegisteredKeys {
    const SCHEMA: &'static str = "biomcp.registered-keys/5";
    type Query = super::EmptyQuery;
}
/// One source name a hosted request may leave out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SwitchableSource {
    /// Source name accepted by the hosted header.
    pub source: crate::Source,
    /// Publisher display name.
    pub name: crate::Text,
    /// Publisher use label for the whole source.
    pub usage: crate::Use,
}
/// Source names a hosted request may leave out, and the header that carries them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SwitchableSources {
    /// Comma-separated hosted HTTP request header.
    pub header: crate::Text,
    /// Accepted source names in public order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<SwitchableSource>,
}
impl super::ResponseBody for SwitchableSources {
    const SCHEMA: &'static str = "biomcp.switchable-sources/1";
    type Query = super::EmptyQuery;
}
