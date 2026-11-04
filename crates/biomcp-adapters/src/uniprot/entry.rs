//! Exact UniProt entry identity is checked before every cache admission.
use super::SPEC;
use crate::Mapped;
use biodata::{ProteinAudit, ProteinHeader, UniProtIsoform};
use biomcp_fetch::{Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use serde::{Deserialize, Serialize};
pub(super) mod map;
pub(super) mod raw;
mod sections_map;
mod sections_raw;
/// A source-native card without sequence, references or other sections.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    /// Exact returned identity and native metadata.
    pub header: ProteinHeader,
    /// Independent native entry and sequence audit.
    pub audit: ProteinAudit,
    /// Native source features, without coordinate transfer.
    pub features: Vec<biodata::ProteinFeature>,
    /// Native alternative product rows.
    pub isoforms: Vec<biodata::ProteinIsoform>,
    /// Native PDB links and properties.
    pub structures: Vec<biodata::ProteinStructureRef>,
    /// Complete source paragraphs with their own evidence.
    pub function: Vec<biodata::ProteinFunction>,
    /// Direct displayed sequence, when supplied by the entry.
    pub sequence: Option<biodata::ProteinSequence>,
}
/// Fetch exactly the requested accession, preserving an explicit isoform suffix.
pub fn entry(leg: &Leg<'_>, id: &UniProtIsoform) -> Result<Mapped<Option<Entry>>, RequestError> {
    let ep = SPEC
        .get("")
        .with_path(format!("/uniprotkb/{id}.json"))?
        .identity_encoding()
        .fresh(Freshness::Days(7));
    match leg
        .call(
            &ep,
            Input::Params(&[]),
            Read::<raw::Entry>::json().cache_if(&|raw| raw.admitted(id)),
        )
        .map(|received| received.value)
    {
        Ok(raw) => Ok(map::entry(raw, id)),
        Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::NotFound => Ok(Mapped {
            refusals: crate::Refusal::None,
            value: None,
            problems: Vec::new(),
        }),
        Err(e) => Err(e),
    }
}
