use biomcp_adapters::ctgov::detail::Detail;
use biomcp_fetch::CacheError;
use biomcp_types::SourceOutcome;
use sha2::{Digest, Sha256};
pub(super) struct Snapshot {
    pub(super) detail: Detail,
    pub(super) fetched: Option<u64>,
    pub(super) version: Option<biomcp_types::SourceTimestamp>,
    pub(super) processing_problems: Vec<biomcp_types::Problem>,
    pub(super) sources: Vec<SourceOutcome>,
}
// Detail owns mandatory named fields; fixed query metadata follows it. The
// The key separates snapshots that retain native record refusal facts.
type Saved = (
    Detail,
    Option<u64>,
    Option<biomcp_types::SourceTimestamp>,
    Vec<SourceOutcome>,
    Vec<biomcp_types::Problem>,
);
impl Snapshot {
    pub(super) fn key(id: biodata::NctId) -> Vec<u8> {
        Sha256::digest(format!("biomcp.trial_snapshot/6\0{id}").as_bytes()).to_vec()
    }
    pub(super) fn encode(&self) -> Result<String, CacheError> {
        serde_json::to_string(&(
            &self.detail,
            self.fetched,
            &self.version,
            &self.sources,
            &self.processing_problems,
        ))
        .map_err(|_| CacheError::Snapshot)
    }
    pub(super) fn decode(payload: &str) -> Result<Self, CacheError> {
        let (detail, fetched, version, sources, processing_problems): Saved =
            serde_json::from_str(payload).map_err(|_| CacheError::Snapshot)?;
        Ok(Self {
            detail,
            fetched,
            version,
            sources,
            processing_problems,
        })
    }
}
