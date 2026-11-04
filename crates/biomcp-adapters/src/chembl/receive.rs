//! Admit ChEMBL metadata without discarding the owning native record.
use super::raw::{CrossReference, Hierarchy, Indication, Mechanism, Molecule, Reference, Synonym};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct ReceivedMolecule {
    /// Exact published molecule ID.
    pub molecule_chembl_id: String,
    /// Native preferred name.
    pub pref_name: Option<crate::raw::Field<String>>,
    /// Native record type, including nonchemical therapies.
    pub molecule_type: Option<crate::raw::Field<String>>,
    /// Explicit source synonyms.
    pub molecule_synonyms: Option<crate::raw::Field<Vec<crate::raw::Field<Synonym>>>>,
    /// Explicit native source cross-references.
    pub cross_references: Option<crate::raw::Field<Vec<crate::raw::Field<CrossReference>>>>,
    /// Explicit native hierarchy.
    pub molecule_hierarchy: Option<crate::raw::Field<Hierarchy>>,
}
impl From<ReceivedMolecule> for Molecule {
    fn from(row: ReceivedMolecule) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: crate::mapped::record(&row.molecule_chembl_id),
            list: Vec::new(),
        };
        let pref_name = problems.field("pref_name", row.pref_name);
        let molecule_type = problems.required_field("molecule_type", row.molecule_type);
        let mut refusals = crate::Refusal::None;
        let molecule_synonyms = problems
            .records("molecule_synonyms", row.molecule_synonyms, &mut refusals)
            .into_vec();
        let cross_references = problems
            .records("cross_references", row.cross_references, &mut refusals)
            .into_vec();
        Self {
            refusals,
            molecule_chembl_id: row.molecule_chembl_id,
            pref_name,
            molecule_type,
            molecule_synonyms,
            cross_references,
            molecule_hierarchy: problems.field("molecule_hierarchy", row.molecule_hierarchy),
            problems: problems.list,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ReceivedSynonym {
    /// Original name.
    pub molecule_synonym: Option<crate::raw::Field<String>>,
    /// Native name kind.
    pub syn_type: Option<crate::raw::Field<String>>,
}
impl From<ReceivedSynonym> for Synonym {
    fn from(row: ReceivedSynonym) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        let molecule_synonym = problems.required_field("molecule_synonym", row.molecule_synonym);
        let syn_type = problems.required_field("syn_type", row.syn_type);
        Self {
            syn_type,
            molecule_synonym,
            problems: problems.list,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ReceivedMechanism {
    /// Source relation ID.
    pub mec_id: u64,
    /// Native target record, without an inferred gene mapping.
    pub target_chembl_id: Option<crate::raw::Field<String>>,
    /// Native action type.
    pub action_type: Option<crate::raw::Field<String>>,
    /// Complete mechanism wording.
    pub mechanism_of_action: Option<crate::raw::Field<String>>,
    /// Native source links.
    pub mechanism_refs: Option<crate::raw::Field<Vec<crate::raw::Field<Reference>>>>,
}
impl From<ReceivedMechanism> for Mechanism {
    fn from(row: ReceivedMechanism) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: crate::mapped::record(row.mec_id),
            list: Vec::new(),
        };
        let target_chembl_id = problems.field("target_chembl_id", row.target_chembl_id);
        let action_type = problems.field("action_type", row.action_type);
        let mechanism_of_action =
            problems.required_field("mechanism_of_action", row.mechanism_of_action);
        let mut refusals = crate::Refusal::None;
        let mechanism_refs = problems
            .records("mechanism_refs", row.mechanism_refs, &mut refusals)
            .into_vec();
        Self {
            refusals,
            mec_id: row.mec_id,
            target_chembl_id,
            action_type,
            mechanism_of_action,
            mechanism_refs,
            problems: problems.list,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ReceivedIndication {
    /// Native indication ID.
    pub drugind_id: u64,
    /// Native disease ID.
    pub efo_id: Option<crate::raw::Field<String>>,
    /// Native disease name.
    pub efo_term: Option<crate::raw::Field<String>>,
    /// Native MeSH identity.
    pub mesh_id: Option<crate::raw::Field<String>>,
    /// Native MeSH wording.
    pub mesh_heading: Option<crate::raw::Field<String>>,
    /// Native clinical phase string, never an authorization.
    pub max_phase_for_ind: Option<crate::raw::Field<String>>,
    /// Native source links.
    pub indication_refs: Option<crate::raw::Field<Vec<crate::raw::Field<Reference>>>>,
}
impl From<ReceivedIndication> for Indication {
    fn from(row: ReceivedIndication) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: crate::mapped::record(row.drugind_id),
            list: Vec::new(),
        };
        let efo_id = problems.field("efo_id", row.efo_id);
        let efo_term = problems.field("efo_term", row.efo_term);
        let mesh_id = problems.field("mesh_id", row.mesh_id);
        let mesh_heading = problems.field("mesh_heading", row.mesh_heading);
        let max_phase_for_ind = problems.field("max_phase_for_ind", row.max_phase_for_ind);
        let mut refusals = crate::Refusal::None;
        let indication_refs = problems
            .records("indication_refs", row.indication_refs, &mut refusals)
            .into_vec();
        Self {
            refusals,
            drugind_id: row.drugind_id,
            efo_id,
            efo_term,
            mesh_id,
            mesh_heading,
            max_phase_for_ind,
            indication_refs,
            problems: problems.list,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ReceivedReference {
    /// Original reference identity, including source-provided lists.
    pub ref_id: Option<crate::raw::Field<String>>,
    /// Original reference type.
    pub ref_type: Option<crate::raw::Field<String>>,
    /// Original source URL.
    pub ref_url: Option<crate::raw::Field<String>>,
}
impl From<ReceivedReference> for Reference {
    fn from(row: ReceivedReference) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        Self {
            ref_id: problems.required_field("ref_id", row.ref_id),
            ref_type: problems.required_field("ref_type", row.ref_type),
            ref_url: problems.required_field("ref_url", row.ref_url),
            problems: problems.list,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ReceivedCrossReference {
    /// Native identifier or website path.
    pub xref_id: String,
    /// Native label when provided.
    pub xref_name: Option<crate::raw::Field<String>>,
    /// Native source namespace.
    pub xref_src: String,
}
impl From<ReceivedCrossReference> for CrossReference {
    fn from(row: ReceivedCrossReference) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        Self {
            xref_id: row.xref_id,
            xref_name: problems.field("xref_name", row.xref_name),
            xref_src: row.xref_src,
            problems: problems.list,
        }
    }
}
