use crate::openfda::{Envelope, Item};
pub(super) type Response = Envelope<Record>;
use crate::native_text::NativeText;
use biodata::{DrugId, Text};
use biomcp_fetch::BadReply;
use serde::Deserialize;

/// A source SPL revision; the label fields retain their native array boundaries.
#[derive(Debug, Deserialize)]
#[serde(from = "super::wire::Record")]
pub(super) struct Record {
    /// Native revision identifier, distinct from its SPL set.
    pub id: DrugId,
    /// Native SPL set identifier.
    pub set_id: DrugId,
    /// Source-stated revision number.
    pub version: Option<Text>,
    /// Native YYYYMMDD revision date.
    pub effective_time: Option<biodata::PartialDate>,
    /// Rejected native nonidentity fields.
    pub problems: Vec<biomcp_types::Problem>,
    pub(super) openfda: Names,
    pub(super) spl_product_data_elements: Option<Vec<NativeText>>,
    pub(super) boxed_warning: Option<Vec<String>>,
    pub(super) dosage_and_administration: Option<Vec<String>>,
    pub(super) drug_interactions: Option<Vec<String>>,
    pub(super) indications_and_usage: Option<Vec<String>>,
    pub(super) warnings_and_cautions: Option<Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Names {
    pub brand_name: Option<Vec<NativeText>>,
    pub generic_name: Option<Vec<NativeText>>,
    pub spl_set_id: Option<Vec<DrugId>>,
    pub application_number: Option<Vec<NativeText>>,
}
impl Record {
    /// Only native fields the consumer explicitly supports; absent remains absent.
    pub(super) fn parts(&self, heading: &str) -> Option<&[String]> {
        match heading {
            "boxed_warning" => &self.boxed_warning,
            "dosage_and_administration" => &self.dosage_and_administration,
            "drug_interactions" => &self.drug_interactions,
            "indications_and_usage" => &self.indications_and_usage,
            "warnings_and_cautions" => &self.warnings_and_cautions,
            _ => return None,
        }
        .as_deref()
        .filter(|parts| !parts.is_empty())
    }
    pub(super) fn check(&self) -> Result<(), BadReply> {
        use biomcp_types::DrugSource;
        if !DrugSource::OpenfdaLabel.accepts(&self.id)
            || !DrugSource::OpenfdaLabel.accepts(&self.set_id)
            || self
                .openfda
                .spl_set_id
                .iter()
                .flatten()
                .any(|id| id != &self.set_id)
        {
            return Err(BadReply);
        }
        Ok(())
    }
}

impl Item for Record {
    fn check(&self) -> Result<(), BadReply> {
        Record::check(self)
    }
}
