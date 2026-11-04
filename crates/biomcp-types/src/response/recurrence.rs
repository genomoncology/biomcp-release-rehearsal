use crate::{NcbiGeneId, Text, Variant, VariantRecurrenceQuery};
use serde::{Deserialize, Serialize};
/// Exact-allele observations confined to one explicitly bounded source page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantRecurrence {
    /// Source-qualified resolved identity; omitted when resolution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Variant>,
    /// Genomic allele equality only, never a protein, residue or gene match.
    #[serde(default)]
    pub matching_granularity: RecurrenceGranularity,
    /// HGNC-resolved gene query bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrez_gene_id: Option<NcbiGeneId>,
    /// Study title, citation and description kept as the source states them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study: Option<RecurrenceStudy>,
    /// Sample-list size as stated by that source; never an assayed denominator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_sample_count: Option<u64>,
    /// Distinct exact-matched samples on this source page only; absent on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_sample_count: Option<u64>,
    /// Number of source mutation rows examined on this page.
    pub examined_rows: u32,
    /// True only when this invocation read the whole gene query from its first page.
    pub all_pages_read: bool,
    /// Source has further rows or cannot yet establish its end.
    pub more: bool,
    /// Study-specific reuse restrictions require verification even for public data.
    pub terms: RecurrenceTerms,
}
impl super::ResponseBody for VariantRecurrence {
    const SCHEMA: &'static str = "biomcp.variant-recurrence/11";
    type Query = VariantRecurrenceQuery;
}
/// Source-native descriptive metadata, without treating access groups as licenses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RecurrenceStudy {
    /// Source study title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Complete source description, including any literal reuse notice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Text>,
    /// Source publication citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<Text>,
}
/// Public access grants no study-specific commercial reuse permission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RecurrenceTerms {
    /// Study-specific rights remain unverified.
    pub unverified: bool,
    /// Source warning about possible commercial restrictions.
    pub notice: Box<str>,
    /// Publisher FAQ explaining study-specific terms.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Box<str>,
}

biodata::wire_enum! {
    /// Source fields required for a recurrence observation.
    #[derive(Default)]
    RecurrenceGranularity {
        /// Source build, chromosome, genomic bounds, reference and alternate all agree.
        #[default]
        ExactGenomicAllele => "exact-genomic-allele",
    }
}
