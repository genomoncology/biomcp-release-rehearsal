//! Hosted licensed identities follow the command's effective credential.
use crate::{Fetch, KeyUse, Leg, Secrets};
use biomcp_types::KeyTier;
use sha2::{Digest, Sha256};

impl Fetch {
    /// Scope licensed retained replies to this hosted command's effective keys.
    pub fn with_hosted_secrets(&self, secrets: Secrets) -> Self {
        let mut fetch = self.with_secrets(secrets);
        fetch.hosted = true;
        fetch
    }
}

impl Leg<'_> {
    pub(crate) fn licensed_identity(&self, identity: [u8; 32]) -> [u8; 32] {
        if self.fetch.hosted
            && let KeyUse::Required(credential) | KeyUse::Optional(credential) = self.spec.key
            && credential.tier == KeyTier::NoFallback
        {
            return self
                .fetch
                .secrets
                .fingerprint(credential.secret)
                .map_or_else(
                    || {
                        let mut hash = Sha256::new();
                        hash.update(b"biomcp/hosted-keyless/v1\0");
                        hash.update(identity);
                        hash.finalize().into()
                    },
                    |key| key.namespace(identity),
                );
        }
        identity
    }

    pub(crate) fn licensed(&self) -> bool {
        self.fetch.hosted
            && matches!(self.spec.key, KeyUse::Required(c) | KeyUse::Optional(c)
                if c.tier == KeyTier::NoFallback)
    }
}
