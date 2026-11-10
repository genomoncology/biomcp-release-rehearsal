//! Protect the opaque registry continuation shared by hosted tasks.
use crate::{CacheError, Fetch};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};
use std::sync::Arc;

const MAX_PAYLOAD_BYTES: usize = 8 * 1024;
const MAX_ENVELOPE_CHARS: usize =
    ((MAX_PAYLOAD_BYTES + aead::NONCE_LEN + aead::MAX_TAG_LEN) * 4).div_ceil(3);

/// Dedicated process key for portable hosted trial continuations.
pub struct TrialCursorKey(aead::LessSafeKey);

impl std::fmt::Debug for TrialCursorKey {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.write_str("TrialCursorKey(***)")
    }
}

impl TrialCursorKey {
    /// Read a canonical URL-safe unpadded encoding of a 32-byte key.
    pub fn from_encoded(encoded: &str) -> Option<Self> {
        if encoded.len() != 43 {
            return None;
        }
        let key = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        if URL_SAFE_NO_PAD.encode(&key) != encoded {
            return None;
        }
        let key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key).ok()?;
        Some(Self(aead::LessSafeKey::new(key)))
    }

    /// Encrypt and authenticate bounded continuation bytes and their query binding.
    pub fn seal(&self, mut payload: Vec<u8>, binding: &[u8]) -> Result<String, CacheError> {
        if payload.len() > MAX_PAYLOAD_BYTES {
            return Err(CacheError::Limit);
        }
        let mut nonce = [0; aead::NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| CacheError::Encoding)?;
        self.0
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::from(binding),
                &mut payload,
            )
            .map_err(|_| CacheError::Encoding)?;
        let mut envelope = nonce.to_vec();
        envelope.extend(payload);
        Ok(URL_SAFE_NO_PAD.encode(envelope))
    }

    /// Authenticate before the caller decodes any continuation fields.
    pub fn open(&self, envelope: &str, binding: &[u8]) -> Option<Vec<u8>> {
        if envelope.len() > MAX_ENVELOPE_CHARS {
            return None;
        }
        let mut bytes = URL_SAFE_NO_PAD.decode(envelope).ok()?;
        let (nonce, ciphertext) = bytes.split_at_mut_checked(aead::NONCE_LEN)?;
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce).ok()?;
        let opened = self
            .0
            .open_in_place(nonce, aead::Aad::from(binding), ciphertext)
            .ok()?;
        (opened.len() <= MAX_PAYLOAD_BYTES).then(|| opened.to_vec())
    }
}

impl Fetch {
    /// Attach the process key before cloning transports for individual callers.
    pub fn with_trial_cursor_key(mut self, key: Option<TrialCursorKey>) -> Self {
        self.trial_cursor_key = key.map(Arc::new);
        self
    }

    /// Expose the codec only for a hosted request with a configured process key.
    pub fn trial_cursor_key(&self) -> Option<&TrialCursorKey> {
        self.hosted.then_some(())?;
        self.trial_cursor_key.as_deref()
    }
}
