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
    /// Variant identity accepted by the recurrence comparison.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "super::variant_input::RecurrenceInput")
    )]
    #[cfg_attr(feature = "schema", schemars(extend("description" = format!("Variant identity resolving to {}.", VariantRecurrenceQuery::INPUT_REQUIREMENT))))]
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
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = VariantRecurrenceQuery::MAX_LIMIT)))]
    limit: u32,
    /// Zero-based native source page.
    #[serde(default)]
    page: u32,
}
fn limit() -> u32 {
    VariantRecurrenceQuery::MAX_LIMIT
}
crate::query::validation::checked_query!(VariantRecurrenceQuery, Fields);
impl TryFrom<Fields> for VariantRecurrenceQuery {
    type Error = super::gene::QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, Self::Error> {
        crate::VariantGet::new(
            v.input.clone(),
            crate::VariantResolver::Car,
            Some(identity_build(v.build)),
        )?;
        if !(1..=Self::MAX_LIMIT).contains(&v.limit) {
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
    /// Default and largest source page: one call reads a common variant's whole cohort.
    pub const MAX_LIMIT: u32 = 5_000;
    /// Source evidence required for exact recurrence matching and its public guidance.
    pub const INPUT_REQUIREMENT: &str = concat!(
        "one unique source-qualified genomic replacement with nonempty reference and alternate alleles ",
        "(pure insertions and deletions are unavailable)"
    );
    /// Unchanged variant identifier.
    pub fn input(&self) -> &VariantInput {
        &self.0.input
    }
    /// Effective identity assembly, including the shared default.
    pub fn build(&self) -> Option<crate::Build> {
        Some(identity_build(self.0.build))
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

fn identity_build(build: Option<crate::Build>) -> crate::Build {
    build.unwrap_or(crate::VariantGet::DEFAULT_BUILD)
}
