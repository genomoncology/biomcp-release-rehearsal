use biomcp_fetch::{BadReply, KeyKind, Keys, Row};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Native {
    #[serde(rename = "WHO Reference Number")]
    reference: biodata::Text,
    #[serde(rename = "INN, Dosage Form and Strength")]
    inn: Box<str>,
    #[serde(rename = "Product Type")]
    product_type: Box<str>,
    #[serde(rename = "Therapeutic Area")]
    therapeutic_area: Box<str>,
    #[serde(rename = "Applicant")]
    applicant: Box<str>,
    #[serde(rename = "Dosage Form")]
    dosage_form: Box<str>,
    #[serde(rename = "Basis of Listing")]
    basis_of_listing: Box<str>,
    #[serde(rename = "Date of Prequalification")]
    date: Box<str>,
}
impl Row for Native {
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![(KeyKind::Id, self.reference.to_string().into())],
            words: vec![self.inn.clone()],
        }
    }
}
impl Native {
    pub(super) fn matches(&self, query: &str) -> bool {
        let words = |s: &str| {
            s.split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
        };
        let needle = words(query);
        let haystack = words(&self.inn);
        !needle.is_empty() && haystack.windows(needle.len()).any(|part| part == needle)
    }
    /// Parse only a complete native date; absent and unfamiliar cells stay absent for filtering.
    pub(super) fn prequalified_date(&self) -> Result<biodata::PartialDate, BadReply> {
        let mut parts = self
            .date
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|part| !part.is_empty());
        let day = parts
            .next()
            .ok_or(BadReply)?
            .parse()
            .map_err(|_| BadReply)?;
        let month = match parts.next().ok_or(BadReply)? {
            "Jan" => 1,
            "Feb" => 2,
            "Mar" => 3,
            "Apr" => 4,
            "May" => 5,
            "Jun" => 6,
            "Jul" => 7,
            "Aug" => 8,
            "Sep" => 9,
            "Oct" => 10,
            "Nov" => 11,
            "Dec" => 12,
            _ => return Err(BadReply),
        };
        let year = parts
            .next()
            .ok_or(BadReply)?
            .parse()
            .map_err(|_| BadReply)?;
        if parts.next().is_some() {
            return Err(BadReply);
        }
        biodata::PartialDate::new(year, Some(month), Some(day)).map_err(|_| BadReply)
    }
    pub(super) fn reference(&self) -> &biodata::Text {
        &self.reference
    }
}
impl From<Native> for biomcp_types::DrugWhoRow {
    fn from(row: Native) -> Self {
        Self {
            reference: row.reference,
            inn_dosage_form_strength: row.inn,
            product_type: row.product_type,
            therapeutic_area: row.therapeutic_area,
            applicant: row.applicant,
            dosage_form: row.dosage_form,
            basis_of_listing: row.basis_of_listing,
            date_of_prequalification: row.date,
        }
    }
}
