//! Source-native gnomAD population subsets; no frequency inference.
//! VA release/count/cohort fields: <https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/va.graphql>
//! The resolver supplies an ALL cohort alongside ancestry rows; its constants are not observed releases.
use super::SPEC;
use crate::graphql::Query;
use biodata::Text;
use biomcp_fetch::Input;
use biomcp_fetch::{Endpoint, Freshness, Leg, RequestError};
use std::collections::BTreeMap;
const VARIANT: Endpoint = SPEC
    .post_json("/api")
    .idempotent()
    .fresh(Freshness::Days(7));
const QUERY: &str = concat!(
    "query($id:String!){variant(variantId:$id,dataset:gnomad_r4){variant_id exome{ac an ho",
    "mozygote_count filters populations{id ac an}} genome{ac an homozygote_count filters p",
    "opulations{id ac an}} joint{ac an homozygote_count} va{exome{derivedFrom{version} foc",
    "usAlleleCount locusAlleleCount cohort{id}} genome{derivedFrom{version} focusAlleleCou",
    "nt locusAlleleCount cohort{id}}}}}",
);
/// Dataset selector sent unchanged to the source.
pub const DATASET: &str = "gnomad_r4";
mod map;
mod raw;
/// One exact source allele; each dataset is independently nullable.
#[derive(Debug)]
pub struct Variant {
    /// Exact returned source allele identifier.
    pub variant_id: Text,
    /// Optional source release provenance; old retained replies can lack it.
    pub va: Option<Va>,
    /// Exome observation, absent when source says null.
    pub exome: Option<Subset>,
    /// Genome observation, absent when source says null.
    pub genome: Option<Subset>,
    /// Joint totals supplied independently by the source, never summed.
    pub joint: Option<Subset>,
}
/// Source VA cohorts, kept separate for each sequencing dataset.
#[derive(Debug)]
pub struct Va {
    /// Exome cohort observations.
    pub exome: Option<Vec<CohortFrequency>>,
    /// Genome cohort observations.
    pub genome: Option<Vec<CohortFrequency>>,
}
/// Same-response release and count association, not a resolver constant.
#[derive(Debug)]
pub struct CohortFrequency {
    /// Source dataset metadata.
    pub derived_from: Option<Derivation>,
    /// Source focus allele count.
    pub focus_allele_count: Option<u64>,
    /// Source locus allele count.
    pub locus_allele_count: Option<u64>,
    /// Source cohort identity.
    pub cohort: Cohort,
}
/// Source dataset derivation.
#[derive(Debug)]
pub struct Derivation {
    /// Nullable source release, never backfilled.
    pub version: Option<String>,
}
/// Source cohort identity, including the documented ALL total.
#[derive(Debug)]
pub struct Cohort {
    /// Native cohort key.
    pub id: Text,
}
/// Source counts and overlapping source groups, never summed or interpreted.
#[derive(Debug)]
pub struct Subset {
    /// Source allele count.
    pub ac: Option<u64>,
    /// Source allele number.
    pub an: Option<u64>,
    /// Source total homozygote count, including a reported zero.
    pub homozygote_count: Option<u64>,
    /// Source filters, including an explicitly empty list.
    pub filters: Option<Vec<Text>>,
    /// Every group in native source order, including sex-specific groups.
    pub populations: Vec<Group>,
}
/// Source group count without a derived allele frequency.
#[derive(Debug)]
pub struct Group {
    /// Native group identifier.
    pub id: Text,
    /// Source allele count.
    pub ac: Option<u64>,
    /// Source allele number.
    pub an: Option<u64>,
}
/// Read one source allele while admitting independent measurements separately.
pub fn population(leg: &Leg<'_>, id: &str) -> Result<crate::Mapped<Option<Variant>>, RequestError> {
    let body = Query {
        query: QUERY.into(),
        variables: BTreeMap::from([("id".into(), id.into())]),
    };
    let response = leg
        .call(
            &VARIANT,
            Input::json(&[], &body),
            biomcp_fetch::Read::<raw::Response>::json().cache_if(&|response| {
                response.data.variant.as_ref().is_none_or(|value| {
                    value
                        .as_ref()
                        .is_some_and(|variant| variant.variant_id.as_str() == id)
                })
            }),
        )?
        .value;
    Ok(map::variant(response.data.variant, id))
}
