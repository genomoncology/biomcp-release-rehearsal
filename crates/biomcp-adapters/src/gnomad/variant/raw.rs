//! Native nullable measurements are independent of allele and cohort identity.
//! Publisher schema: <https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/variant.graphql>
//! VA counts and cohorts: <https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/va.graphql>
//! Null is absence; filters [] is a supplied empty list, never an inferred PASS classification.
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Response {
    pub(super) data: Data,
    errors: Option<Vec<GraphError>>,
}
#[derive(Debug, Deserialize)]
struct GraphError {
    message: String,
}
#[derive(Debug, Deserialize)]
pub(super) struct Data {
    #[serde(deserialize_with = "nullable")]
    pub(super) variant: Option<Field<Variant>>,
}
/// One exact source allele; each dataset is independently nullable.
#[derive(Debug, Deserialize)]
pub(super) struct Variant {
    /// Exact returned source allele identifier.
    pub(super) variant_id: Text,
    /// Optional source release provenance; old retained replies can lack it.
    pub(super) va: Option<Field<Va>>,
    /// Exome observation, absent when source says null.
    pub(super) exome: Option<Field<Subset>>,
    /// Genome observation, absent when source says null.
    pub(super) genome: Option<Field<Subset>>,
    /// Nullable source-published joint totals.
    pub(super) joint: Option<Field<Subset>>,
}
/// Source VA cohorts, kept separate for each sequencing dataset.
#[derive(Debug, Deserialize)]
pub(super) struct Va {
    /// Exome cohort observations.
    pub(super) exome: Option<Field<Vec<Field<CohortFrequency>>>>,
    /// Genome cohort observations.
    pub(super) genome: Option<Field<Vec<Field<CohortFrequency>>>>,
}
/// Same-response release and count association, not a resolver constant.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CohortFrequency {
    /// Source dataset metadata.
    pub(super) derived_from: Option<Field<Derivation>>,
    /// Source focus allele count.
    pub(super) focus_allele_count: Option<Field<u64>>,
    /// Source locus allele count.
    pub(super) locus_allele_count: Option<Field<u64>>,
    /// Source cohort identity.
    pub(super) cohort: Cohort,
}
/// Source dataset derivation.
#[derive(Debug, Deserialize)]
pub(super) struct Derivation {
    /// Nullable source release, never backfilled.
    pub(super) version: Option<Field<Text>>,
}
/// Source cohort identity, including the documented ALL total.
#[derive(Debug, Deserialize)]
pub(super) struct Cohort {
    /// Native cohort key.
    pub(super) id: Text,
}
/// Source counts and overlapping source groups, never summed or interpreted.
#[derive(Debug, Deserialize)]
pub(super) struct Subset {
    /// Source allele count.
    pub(super) ac: Option<Field<u64>>,
    /// Source allele number.
    pub(super) an: Option<Field<u64>>,
    /// Source total homozygote count.
    pub(super) homozygote_count: Option<Field<u64>>,
    /// Source filters, including an explicitly empty list.
    pub(super) filters: Option<Field<Vec<Field<Text>>>>,
    /// Every group in native source order, including sex-specific groups.
    pub(super) populations: Option<Field<Vec<Field<Group>>>>,
}
/// Source group count without a derived allele frequency.
#[derive(Debug, Deserialize)]
pub(super) struct Group {
    /// Native group identifier.
    pub(super) id: Text,
    /// Source allele count.
    pub(super) ac: Option<Field<u64>>,
    /// Source allele number.
    pub(super) an: Option<Field<u64>>,
}
impl Reply for Response {
    fn check(&self) -> Result<(), BadReply> {
        // The not-found response has one exact native error; successful replies may omit errors.
        if self.data.variant.is_none() {
            if !matches!(self.errors.as_deref(), Some([error]) if error.message == "Variant not found")
            {
                return Err(BadReply);
            }
        } else if self
            .errors
            .as_ref()
            .is_some_and(|errors| !errors.is_empty())
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
fn nullable<'de, T: Deserialize<'de>, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
