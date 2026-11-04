//! Preserve native page counts before refusing row identities.
use super::raw::PageMeta;
use super::{Indication, Mechanism, Molecule};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
/// Native source page with independent row diagnostics.
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedMolecules")]
pub struct Molecules {
    /// Accepted native rows.
    pub window: crate::NativeWindow<Molecule>,
    /// Rejected row identities.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Deserialize)]
struct ReceivedMolecules {
    molecules: Vec<crate::raw::Field<super::raw::Molecule>>,
    page_meta: PageMeta,
}
impl From<ReceivedMolecules> for Molecules {
    fn from(page: ReceivedMolecules) -> Self {
        let received = page.molecules.len() as u32;
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        let mut rows = Vec::new();
        let mut refusals = crate::Refusal::None;
        for row in page.molecules {
            problems.record = None;
            if let Some(mut row) = problems.field("molecule_chembl_id", Some(row)) {
                if row.molecule_chembl_id.starts_with("CHEMBL")
                    && row.molecule_chembl_id.parse::<biodata::DrugId>().is_ok()
                {
                    refusals.include(row.refusals);
                    row.refusals = crate::Refusal::None;
                    rows.push(Molecule(row));
                } else {
                    refusals = crate::Refusal::Refused;
                    problems.record = crate::mapped::record(&row.molecule_chembl_id);
                    problems.required::<()>("molecule_chembl_id", None);
                }
            } else {
                refusals = crate::Refusal::Refused;
            }
        }
        Self {
            window: crate::NativeWindow::new(
                rows,
                page.page_meta.offset,
                page.page_meta.limit,
                Some(page.page_meta.total_count),
                received,
                refusals,
            ),
            problems: problems.list,
        }
    }
}
impl Reply for Molecules {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Native source page with independent row diagnostics.
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedMechanisms")]
pub struct Mechanisms {
    pub(super) molecules: Vec<String>,
    /// Accepted native rows.
    pub window: crate::NativeWindow<Mechanism>,
    /// Rejected row identities.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Deserialize)]
struct ReceivedMechanisms {
    mechanisms: Vec<super::scope::Row<super::raw::Mechanism>>,
    page_meta: PageMeta,
}
impl From<ReceivedMechanisms> for Mechanisms {
    fn from(page: ReceivedMechanisms) -> Self {
        let received = page.mechanisms.len() as u32;
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        let mut rows = Vec::new();
        let mut refusals = crate::Refusal::None;
        let mut molecules = Vec::new();
        for row in page.mechanisms {
            molecules.push(row.scope.molecule_chembl_id);
            problems.record = None;
            if let Some(mut row) = problems.field("mec_id", Some(row.value)) {
                if row.mec_id > 0 {
                    refusals.include(row.refusals);
                    row.refusals = crate::Refusal::None;
                    rows.push(Mechanism(row));
                } else {
                    refusals = crate::Refusal::Refused;
                    problems.record = crate::mapped::record(row.mec_id);
                    problems.required::<()>("mec_id", None);
                }
            } else {
                refusals = crate::Refusal::Refused;
            }
        }
        Self {
            molecules,
            window: crate::NativeWindow::new(
                rows,
                page.page_meta.offset,
                page.page_meta.limit,
                Some(page.page_meta.total_count),
                received,
                refusals,
            ),
            problems: problems.list,
        }
    }
}
impl Mechanisms {
    pub(super) fn identities_admitted(&self) -> bool {
        let accepted = self
            .window
            .rows
            .iter()
            .filter(|row| matches!(row.target_identity(), Ok(Some(_))))
            .count();
        let mut refusals = self.window.refusals();
        if accepted < self.window.rows.len() {
            refusals.include(crate::Refusal::Refused);
        }
        refusals.identities_admitted(accepted, self.window.received() as usize)
    }
}
impl Reply for Mechanisms {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Native source page with independent row diagnostics.
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedIndications")]
pub struct Indications {
    pub(super) molecules: Vec<String>,
    /// Accepted native rows.
    pub window: crate::NativeWindow<Indication>,
    /// Rejected row identities.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Deserialize)]
struct ReceivedIndications {
    drug_indications: Vec<super::scope::Row<super::raw::Indication>>,
    page_meta: PageMeta,
}
impl From<ReceivedIndications> for Indications {
    fn from(page: ReceivedIndications) -> Self {
        let received = page.drug_indications.len() as u32;
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Chembl,
            record: None,
            list: Vec::new(),
        };
        let mut rows = Vec::new();
        let mut refusals = crate::Refusal::None;
        let mut molecules = Vec::new();
        for row in page.drug_indications {
            molecules.push(row.scope.molecule_chembl_id);
            problems.record = None;
            if let Some(mut row) = problems.field("drugind_id", Some(row.value)) {
                if row.drugind_id > 0 {
                    refusals.include(row.refusals);
                    row.refusals = crate::Refusal::None;
                    rows.push(Indication(row));
                } else {
                    refusals = crate::Refusal::Refused;
                    problems.record = crate::mapped::record(row.drugind_id);
                    problems.required::<()>("drugind_id", None);
                }
            } else {
                refusals = crate::Refusal::Refused;
            }
        }
        Self {
            molecules,
            window: crate::NativeWindow::new(
                rows,
                page.page_meta.offset,
                page.page_meta.limit,
                Some(page.page_meta.total_count),
                received,
                refusals,
            ),
            problems: problems.list,
        }
    }
}
impl Reply for Indications {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
