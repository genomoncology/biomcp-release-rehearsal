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
    pub(super) sections: std::collections::BTreeMap<String, Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Names {
    pub brand_name: Option<Vec<NativeText>>,
    pub generic_name: Option<Vec<NativeText>>,
    pub spl_set_id: Option<Vec<DrugId>>,
    pub application_number: Option<Vec<NativeText>>,
}
impl Record {
    /// Only published native sections survive admission; absent remains absent.
    pub(super) fn parts(&self, heading: &str) -> Option<&[String]> {
        self.sections
            .get(heading)
            .map(Vec::as_slice)
            .filter(|parts| !parts.is_empty())
    }
    /// Check product-data text without changing the original section strings.
    pub(super) fn product_name(&self) -> (Option<Text>, Vec<biomcp_types::Problem>) {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::OpenfdaLabel,
            record: crate::mapped::record(&self.id),
            list: Vec::new(),
        };
        let mut name = None;
        for value in self
            .parts("spl_product_data_elements")
            .iter()
            .copied()
            .flatten()
        {
            let value =
                problems.optional("spl_product_data_elements", Some(Text::from_source(value)));
            if name.is_none() {
                name = value;
            }
        }
        (name, problems.list)
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
