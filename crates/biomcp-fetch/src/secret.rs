use crate::SecretName;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt};

/// Opaque identity of an effective credential, with no credential disclosure.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyFingerprint([u8; 32]);

impl KeyFingerprint {
    /// Scope an existing identity without exposing the credential fingerprint.
    pub fn namespace(self, identity: [u8; 32]) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"biomcp/key-namespace/v1\0");
        hash.update(self.0);
        hash.update(identity);
        hash.finalize().into()
    }
}

impl fmt::Debug for KeyFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KeyFingerprint(***)")
    }
}

/// A credential that never formats its value.
pub struct Secret(Box<str>);

impl Secret {
    /// Store an explicitly supplied credential.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    pub(crate) fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Explicitly supplied secrets; never reads environment or files.
#[derive(Debug, Default)]
pub struct Secrets(BTreeMap<SecretName, SecretValue>);
#[derive(Debug)]
enum SecretValue {
    Ready(Secret),
    Deferred(DeferredSecret),
}
#[expect(
    clippy::disallowed_types,
    reason = "Resolve one deferred credential once per configured secret"
)]
// A caller-supplied credential resolves once per Secrets owner, never globally.
struct DeferredSecret {
    resolved: std::sync::OnceLock<Result<Option<Secret>, crate::InputError>>,
    load: Box<dyn Fn() -> Result<Option<Secret>, crate::InputError> + Send + Sync>,
}
impl fmt::Debug for DeferredSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeferredSecret(***)")
    }
}
impl SecretValue {
    fn get(&self) -> Option<&str> {
        match self {
            Self::Ready(secret) => Some(secret.value()),
            Self::Deferred(secret) => secret
                .resolved
                .get_or_init(|| (secret.load)())
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(Secret::value),
        }
        .filter(|v| !v.is_empty())
    }
}

impl Secrets {
    /// Identify the resolved nonempty effective credential without exposing it.
    pub fn fingerprint(&self, name: SecretName) -> Option<KeyFingerprint> {
        self.get(name).map(|value| {
            let mut hash = Sha256::new();
            hash.update(b"biomcp/key-fingerprint/v1\0");
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(value.as_bytes());
            KeyFingerprint(hash.finalize().into())
        })
    }

    /// Insert or replace a configured secret.
    pub fn insert(&mut self, name: SecretName, value: Secret) {
        self.0.insert(name, SecretValue::Ready(value));
    }
    #[expect(
        clippy::disallowed_types,
        reason = "Resolve one deferred credential once per configured secret"
    )]
    /// Resolve a caller-owned credential only when an admitted request needs it.
    /// The loader runs once for this owner, including a missing credential.
    pub fn defer(
        &mut self,
        name: SecretName,
        load: impl Fn() -> Result<Option<Secret>, crate::InputError> + Send + Sync + 'static,
    ) {
        self.0.insert(
            name,
            SecretValue::Deferred(DeferredSecret {
                resolved: std::sync::OnceLock::new(),
                load: Box::new(load),
            }),
        );
    }
    /// Resolve this command's hosted credentials from caller keys and declared tiers.
    pub fn hosted(
        mut self,
        request: Option<&Self>,
        no_fallback: impl IntoIterator<Item = SecretName>,
    ) -> Self {
        for name in no_fallback {
            self.0.remove(&name);
        }
        if let Some(request) = request {
            for (&name, value) in &request.0 {
                if let Some(value) = value.get() {
                    self.insert(name, Secret::new(value));
                }
            }
        }
        self
    }
    pub(crate) fn invalid(&self, name: SecretName) -> bool {
        self.0.get(&name).is_some_and(|value| match value {
            SecretValue::Ready(_) => false,
            SecretValue::Deferred(secret) => {
                secret.resolved.get_or_init(|| (secret.load)()).is_err()
            }
        })
    }
    pub(crate) fn get(&self, name: SecretName) -> Option<&str> {
        self.0.get(&name).and_then(SecretValue::get)
    }
}

impl SecretName {
    /// Field within the configuration's keys object.
    pub const fn config(self) -> &'static str {
        match self {
            Self::OpenfdaApiKey => "openfda_api_key",
            Self::NcbiApiKey => "ncbi_api_key",
            Self::SemanticScholarApiKey => "semantic_scholar_api_key",
            Self::OrcidAccessToken => "orcid_access_token",
            Self::OncokbApiKey => "oncokb_api_key",
            Self::AlphaGenomeApiKey => "alphagenome_api_key",
            Self::NciApiKey => "nci_api_key",
        }
    }
    /// HTTP header for a hosted request credential, derived from its configuration name.
    pub fn header(self) -> String {
        format!("x-biomcp-key-{}", self.config().replace('_', "-"))
    }
    /// Optional environment fallback.
    pub const fn environment(self) -> &'static str {
        match self {
            Self::OpenfdaApiKey => "OPENFDA_API_KEY",
            Self::NcbiApiKey => "NCBI_API_KEY",
            Self::SemanticScholarApiKey => "S2_API_KEY",
            Self::OrcidAccessToken => "ORCID_ACCESS_TOKEN",
            Self::OncokbApiKey => "ONCOKB_TOKEN",
            Self::AlphaGenomeApiKey => "ALPHAGENOME_API_KEY",
            Self::NciApiKey => "NCI_API_KEY",
        }
    }
}
