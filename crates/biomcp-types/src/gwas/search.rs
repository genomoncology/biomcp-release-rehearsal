//! Standalone GWAS requests. Shared command registration follows separately.
use crate::Paged;
use crate::{GeneSymbol, Pmid, QueryInvalid, RsId, Text};
use biodata::{GwasStudyId, GwasTraitId, ScientificPValue};
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    described
    /// Native source order, echoed without a configurable sort command.
    GwasOrder {
        #[doc = "Ascending source p value."] PValueAscending => "p_value,asc"
    }
}
fn order() -> GwasOrder {
    GwasOrder::PValueAscending
}
/// Source associations in default p-value-ascending order. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct GwasSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Return study records for the selected publication instead of associations.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    studies: bool,
    /// Exact source rsID.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "RsId"))]
    variant: Option<RsId>,
    /// Literal native efo_trait label; never searches reported_trait prose.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    r#trait: Option<Text>,
    /// Exact native ontology identity.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GwasTraitId"))]
    trait_id: Option<GwasTraitId>,
    /// Exact native mapped gene; legacy positional query aliases this field.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneSymbol"))]
    gene: Option<GeneSymbol>,
    /// Exact source study accession.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GwasStudyId"))]
    study: Option<GwasStudyId>,
    /// Exact source publication identity.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Pmid"))]
    pubmed: Option<Pmid>,
    /// Inclusive positive probability in scientific notation, such as 1e-5. The qualifying total stays unknown until the ordered walk reaches its end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "threshold")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    p_value: Option<ScientificPValue>,
    /// Source child-trait expansion; requires trait_id.
    #[serde(default)]
    include_child_traits: bool,
    /// Source extended gene set; requires gene.
    #[serde(default)]
    extended_geneset: bool,
    /// Source order; threshold requests use this same native order.
    #[serde(default = "order")]
    order: GwasOrder,
    /// Maximum association rows.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Retained threshold walk identity; requires a threshold.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
    /// Native association offset.
    #[serde(default)]
    offset: u32,
}
fn limit() -> u32 {
    3
}
impl crate::CheckedQuery for GwasSearch {
    const USAGE_MESSAGE: Option<crate::ErrorMessage> =
        Some(crate::ErrorMessage::GwasSearchArguments);
    fn invalid_field(name: &str) -> QueryInvalid {
        let cause = QueryInvalid::new(&[name]);
        if name == "p_value" {
            cause.with_message(crate::ErrorMessage::GwasPositiveThreshold)
        } else {
            cause
        }
    }
    fn from_fields<'de, D: serde::Deserializer<'de, Error = QueryInvalid>>(
        deserializer: D,
    ) -> Result<Self, QueryInvalid> {
        Self::try_from(SearchFields::deserialize(deserializer)?)
    }
}

impl TryFrom<SearchFields> for GwasSearch {
    type Error = QueryInvalid;
    fn try_from(f: SearchFields) -> Result<Self, Self::Error> {
        if f.studies
            && (f.pubmed.is_none()
                || f.variant.is_some()
                || f.r#trait.is_some()
                || f.trait_id.is_some()
                || f.gene.is_some()
                || f.study.is_some()
                || f.p_value.is_some())
        {
            return Err(QueryInvalid::new(&["studies", "pubmed"]));
        }
        if f.variant.is_none()
            && f.r#trait.is_none()
            && f.trait_id.is_none()
            && f.gene.is_none()
            && f.study.is_none()
            && f.pubmed.is_none()
        {
            return Err(QueryInvalid::new(&[
                "variant", "trait", "trait_id", "gene", "study", "pubmed",
            ]));
        }
        if f.include_child_traits && f.trait_id.is_none() {
            return Err(QueryInvalid::new(&["include_child_traits", "trait_id"]));
        }
        if f.extended_geneset && f.gene.is_none() {
            return Err(QueryInvalid::new(&["extended_geneset", "gene"]));
        }
        if f.pool.is_some() && f.p_value.is_none() {
            return Err(QueryInvalid::new(&["pool", "p_value"]));
        }
        if !(1..=50).contains(&f.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(f))
    }
}
impl GwasSearch {
    /// Reuse the native search selector for a checked study section window.
    pub fn study_page(id: GwasStudyId, page: &super::GwasSectionPage) -> Self {
        Self(SearchFields {
            studies: false,
            variant: None,
            r#trait: None,
            trait_id: None,
            gene: None,
            study: Some(id),
            pubmed: None,
            p_value: None,
            include_child_traits: false,
            extended_geneset: false,
            order: order(),
            limit: page.limit(),
            offset: page.offset(),
            pool: None,
        })
    }

    /// Select publication studies rather than association rows.
    pub fn studies(&self) -> bool {
        self.0.studies
    }

    /// Read only one association to obtain a native study total.
    pub fn study_count(id: GwasStudyId) -> Self {
        let mut query = Self::study_page(id, &super::GwasSectionPage::default());
        query.0.limit = 1;
        query
    }

    /// Checked variant selector.
    pub fn variant(&self) -> Option<&RsId> {
        self.0.variant.as_ref()
    }
    /// Checked trait selector.
    pub fn trait_label(&self) -> Option<&Text> {
        self.0.r#trait.as_ref()
    }
    /// Checked trait_id selector.
    pub fn trait_id(&self) -> Option<&GwasTraitId> {
        self.0.trait_id.as_ref()
    }
    /// Checked gene selector.
    pub fn gene(&self) -> Option<&GeneSymbol> {
        self.0.gene.as_ref()
    }
    /// Checked study selector.
    pub fn study(&self) -> Option<&GwasStudyId> {
        self.0.study.as_ref()
    }
    /// Checked pubmed selector.
    pub fn pubmed(&self) -> Option<&Pmid> {
        self.0.pubmed.as_ref()
    }
    /// Checked p_value selector.
    pub fn p_value(&self) -> Option<&ScientificPValue> {
        self.0.p_value.as_ref()
    }
    /// Native include_child_traits selector.
    pub fn include_child_traits(&self) -> bool {
        self.0.include_child_traits
    }
    /// Native extended_geneset selector.
    pub fn extended_geneset(&self) -> bool {
        self.0.extended_geneset
    }
    /// Native order selector.
    pub fn order(&self) -> GwasOrder {
        self.0.order
    }
    /// Native limit selector.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native offset selector.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained threshold walk identity.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }

    /// Continue a retained threshold walk.
    pub fn in_pool(&self, pool: Text, offset: u32) -> Self {
        let mut q = self.at_offset(offset);
        q.0.pool = Some(pool);
        q
    }
}
impl crate::Paged for GwasSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}

mod threshold {
    use biodata::ScientificPValue;
    use serde::{Deserialize, Deserializer, Serializer, ser::Error as _};
    #[expect(
        clippy::ref_option,
        reason = "Serde serialize_with passes a reference to the entire optional field"
    )]
    pub(super) fn serialize<S: Serializer>(
        value: &Option<ScientificPValue>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(number) => serializer.serialize_str(&number.to_string()),
            None => Err(S::Error::custom("absent thresholds must be omitted")),
        }
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<ScientificPValue>, D::Error> {
        use serde::de::Error as _;
        String::deserialize(deserializer)?
            .parse()
            .map(Some)
            .map_err(D::Error::custom)
    }
}
