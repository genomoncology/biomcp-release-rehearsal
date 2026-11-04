//! Native ChEMBL records; source fields never become chemical or regulatory inference.
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
/// Native molecule identity returned by the molecule endpoint.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedMolecule")]
pub(super) struct Molecule {
    /// Actual refused native child records.
    pub refusals: crate::Refusal,
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Exact published molecule ID.
    pub molecule_chembl_id: String,
    /// Native preferred name.
    pub pref_name: Option<String>,
    /// Native record type, including nonchemical therapies.
    pub molecule_type: Option<String>,
    /// Explicit source synonyms.
    pub molecule_synonyms: Vec<Synonym>,
    /// Explicit native source cross-references.
    pub cross_references: Vec<CrossReference>,
    /// Explicit native hierarchy.
    pub molecule_hierarchy: Option<Hierarchy>,
}
/// One source alternate name.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedSynonym")]
pub(super) struct Synonym {
    pub syn_type: Option<String>,
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Original name.
    pub molecule_synonym: Option<String>,
}
/// Source molecule parent relationship.
#[derive(Debug, Deserialize)]
pub(super) struct Hierarchy {
    /// Source parent ID, possibly the molecule itself.
    pub parent_chembl_id: String,
}
/// Source page metadata, independent of local row counts.
#[derive(Debug, Deserialize)]
pub(super) struct PageMeta {
    /// Native total.
    pub total_count: u64,
    /// Native offset.
    pub offset: u32,
    /// Native page size.
    pub limit: u32,
}

/// Native target mechanism, which need not identify a gene.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedMechanism")]
pub(super) struct Mechanism {
    /// Actual refused native child records.
    pub refusals: crate::Refusal,
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Source relation ID.
    pub mec_id: u64,
    /// Native target record, without an inferred gene mapping.
    pub target_chembl_id: Option<String>,
    /// Native action type.
    pub action_type: Option<String>,
    /// Complete mechanism wording.
    pub mechanism_of_action: Option<String>,
    /// Native source links.
    pub mechanism_refs: Vec<Reference>,
}
impl Mechanism {
    pub(super) fn target_identity(&self) -> Result<Option<biodata::Text>, biodata::Error> {
        self.target_chembl_id
            .as_deref()
            .map(biodata::Text::new)
            .transpose()
    }
}
/// Native indication, independent of a regulatory authorization.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedIndication")]
pub(super) struct Indication {
    /// Actual refused native child records.
    pub refusals: crate::Refusal,
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native indication ID.
    pub drugind_id: u64,
    /// Native disease ID.
    pub efo_id: Option<String>,
    /// Native disease name.
    pub efo_term: Option<String>,
    /// Native MeSH identity.
    pub mesh_id: Option<String>,
    /// Native MeSH wording.
    pub mesh_heading: Option<String>,
    /// Native clinical phase string, never an authorization.
    pub max_phase_for_ind: Option<String>,
    /// Native source links.
    pub indication_refs: Vec<Reference>,
}
/// Native source reference, retaining its original identifiers and URL.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedReference")]
pub(super) struct Reference {
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Original reference identity, including source-provided lists.
    pub ref_id: Option<String>,
    /// Original reference type.
    pub ref_type: Option<String>,
    /// Original source URL.
    pub ref_url: Option<String>,
}

impl Reply for Molecule {
    fn check(&self) -> Result<(), BadReply> {
        if self.molecule_chembl_id.starts_with("CHEMBL")
            && self.molecule_chembl_id.parse::<biodata::DrugId>().is_ok()
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}

/// A source-stated cross-reference, preserving its original source namespace and path.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedCrossReference")]
pub(super) struct CrossReference {
    /// Rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native identifier or website path.
    pub xref_id: String,
    /// Native label when provided.
    pub xref_name: Option<String>,
    /// Native source namespace.
    pub xref_src: String,
}
