//! Shared fixture content hashing.
use sha2::{Digest, Sha256};
/// Hash fixture bytes with SHA-256.
pub fn hash(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
