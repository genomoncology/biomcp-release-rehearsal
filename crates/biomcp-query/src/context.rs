use biomcp_fetch::Fetch;
use std::time::Duration;
/// Shared transport, command budget and outgoing-only source contact.
#[derive(Debug)]
pub struct Ctx<'a> {
    /// Optional outgoing-only NCBI contact.
    pub ncbi_email: Option<biomcp_fetch::Secret>,
    /// Shared checked transport.
    pub fetch: &'a Fetch,
    /// Absolute command budget shared by source legs.
    pub budget: Duration,
    /// Bypass cached reads on this request only.
    pub fresh: bool,
    /// Most names one search may send for a drug.
    pub drug_name_limit: usize,
}
/// Default for [`Ctx::drug_name_limit`].
pub const DRUG_NAME_LIMIT: usize = 30;
