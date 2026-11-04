//! Admit native gene–disease–MOI identities before optional curation fields.
//! Publisher SOP v11 pp. 10–11: https://clinicalgenome.org/site/assets/files/9851/gene-disease_validity_standard_operating_procedures-_version_11_docx.pdf
//! Export field names: https://search.clinicalgenome.org/kb/gene-validity/download
use crate::native_text::NativeText;
use biodata::{GeneSymbol, HgncId, Text};

use crate::{mapped::Problems, raw::Field};
use biomcp_fetch::{KeyKind, Keys, Row};
use biomcp_types::ClinGenValidity;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Validity {
    #[serde(rename = "GENE ID (HGNC)")]
    id: Option<Field<HgncId>>,
    #[serde(rename = "GENE SYMBOL")]
    symbol: Option<Field<GeneSymbol>>,
    #[serde(rename = "DISEASE LABEL")]
    disease: Option<Field<NativeText>>,
    #[serde(rename = "DISEASE ID (MONDO)")]
    mondo_id: Option<Field<biodata::MondoId>>,
    #[serde(rename = "MOI")]
    mode_of_inheritance: Option<Field<Text>>,
    #[serde(rename = "CLASSIFICATION")]
    classification: Option<Field<Text>>,
    #[serde(rename = "CLASSIFICATION DATE")]
    classification_date: Option<Field<Text>>,
    #[serde(rename = "GCEP")]
    expert_panel: Option<Field<NativeText>>,
    #[serde(rename = "ONLINE REPORT")]
    report_url: Option<Field<Text>>,
}
impl Validity {
    pub(super) fn checked(self, problems: &mut Problems) -> Option<ClinGenValidity> {
        problems.record = None;
        let id = problems.required_field("GENE ID (HGNC)", self.id)?;
        problems.record = crate::mapped::record(id);
        Some(ClinGenValidity {
            mondo_id: problems.required_field("DISEASE ID (MONDO)", self.mondo_id)?,
            mode_of_inheritance: problems.required_field("MOI", self.mode_of_inheritance)?,
            disease: problems.source_field("DISEASE LABEL", self.disease),
            classification: problems.field("CLASSIFICATION", self.classification),
            classification_date: problems.field("CLASSIFICATION DATE", self.classification_date),
            expert_panel: problems.source_field("GCEP", self.expert_panel),
            report_url: problems.url("ONLINE REPORT", self.report_url),
        })
    }
}
impl Row for Validity {
    fn keys(&self) -> Keys {
        keys(self.id.as_ref(), self.symbol.as_ref())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Dosage {
    #[serde(rename = "HGNC ID")]
    id: Option<Field<HgncId>>,
    #[serde(rename = "GENE SYMBOL")]
    symbol: Option<Field<GeneSymbol>>,
    #[serde(rename = "HAPLOINSUFFICIENCY")]
    pub haploinsufficiency: Option<Field<Text>>,
    #[serde(rename = "TRIPLOSENSITIVITY")]
    pub triplosensitivity: Option<Field<Text>>,
    #[serde(rename = "DATE")]
    pub date: Option<Field<Text>>,
}
impl Row for Dosage {
    fn keys(&self) -> Keys {
        keys(self.id.as_ref(), self.symbol.as_ref())
    }
}
fn keys(id: Option<&Field<HgncId>>, symbol: Option<&Field<GeneSymbol>>) -> Keys {
    let key = match id.and_then(Field::as_ref) {
        Some(id) => Some(id.to_string()),
        None => symbol.and_then(Field::as_ref).map(rejected_key),
    };
    Keys {
        exact: key
            .into_iter()
            .map(|key| (KeyKind::Id, key.into()))
            .collect(),
        words: Vec::new(),
    }
}
/// Rejection-only index key; a native symbol scopes disclosures without admitting an identity.
pub(super) fn rejected_key(symbol: &GeneSymbol) -> String {
    format!("rejected-hgnc:{symbol}")
}
impl Dosage {
    pub(super) fn checked(self, problems: &mut Problems) -> Option<super::Dosage> {
        problems.record = None;
        let id = problems.required_field("HGNC ID", self.id)?;
        problems.record = crate::mapped::record(id);
        Some(super::Dosage {
            haploinsufficiency: problems.field("HAPLOINSUFFICIENCY", self.haploinsufficiency),
            triplosensitivity: problems.field("TRIPLOSENSITIVITY", self.triplosensitivity),
            date: problems.field("DATE", self.date),
        })
    }
}
