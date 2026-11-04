use crate::{GeneSymbol, Text, VariantInput};
use serde::{Deserialize, Serialize};
/// Observe exact-allele samples within one caller-selected source page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct VariantRecurrenceQuery(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Published variant identity; ambiguous identities are never selected silently.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    input: VariantInput,
    /// Explicit cohort source.
    source: RecurrenceSource,
    /// Identity lookup assembly; omitted uses the shared GRCh38 default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<crate::Build>,
    /// HGNC symbol resolved to the source's Entrez gene identifier.
    gene: GeneSymbol,
    /// Explicit study; there is no default cohort.
    study: Text,
    /// Explicit mutation molecular profile.
    profile: Text,
    /// Explicit sample list; selected size is never an allele denominator.
    sample_set: Text,
    /// Source mutation rows per page, including excluded alleles.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 100)))]
    limit: u32,
    /// Zero-based native source page.
    #[serde(default)]
    page: u32,
}
fn limit() -> u32 {
    10
}
crate::query::validation::checked_query!(VariantRecurrenceQuery, Fields);
impl TryFrom<Fields> for VariantRecurrenceQuery {
    type Error = super::gene::QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, Self::Error> {
        crate::VariantGet::new(v.input.clone(), crate::VariantResolver::Car, v.build)?;
        if !(1..=100).contains(&v.limit) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        if v.page.checked_mul(v.limit).is_none() {
            return Err(super::gene::QueryInvalid::new(&["page", "limit"]));
        }
        if [&v.study, &v.profile, &v.sample_set].iter().any(|s| {
            !s.as_str()
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        }) {
            return Err(super::gene::QueryInvalid::new(&[
                "study",
                "profile",
                "sample_set",
            ]));
        }
        Ok(Self(v))
    }
}
impl VariantRecurrenceQuery {
    /// Unchanged variant identifier.
    pub fn input(&self) -> &VariantInput {
        &self.0.input
    }
    /// Caller assembly for identity resolution.
    pub fn build(&self) -> Option<crate::Build> {
        self.0.build
    }
    /// Caller-selected gene symbol.
    pub fn gene(&self) -> &GeneSymbol {
        &self.0.gene
    }
    /// Caller-selected study.
    pub fn study(&self) -> &Text {
        &self.0.study
    }
    /// Caller-selected mutation profile.
    pub fn profile(&self) -> &Text {
        &self.0.profile
    }
    /// Caller-selected sample list.
    pub fn sample_set(&self) -> &Text {
        &self.0.sample_set
    }
    /// Source row page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native source page number.
    pub fn page(&self) -> u32 {
        self.0.page
    }
    /// Continue without changing cohort, allele or page size.
    pub fn next_page(&self) -> Option<Self> {
        let mut q = self.clone();
        q.0.page = q.0.page.checked_add(1)?;
        q.0.page.checked_mul(q.0.limit)?;
        Some(q)
    }
}
biodata::wire_enum! {
    /// Source requiring explicit study, profile and sample set.
    RecurrenceSource {
        /// Public cBioPortal mutation rows.
        Cbioportal => "cbioportal",
    }
}
