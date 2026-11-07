use super::DrugSource;
use crate::QueryInvalid;
use biodata::Text;
use serde::{Deserialize, Serialize};

fn default_source() -> DrugSource {
    DrugSource::Mychem
}
fn default_limit() -> u32 {
    10
}
#[doc = " Search one drug source. FDA drug recalls accept a name or native firm, classification, \
    status, reason and initiation dates; counts are recall records. \
    WHO prequalification accepts a name or date range. openFDA labels also accept literal indication wording. \
    Other sources require a name."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct DrugSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Name phrase; optional with supported source filters.
    /// FDA drug recalls, WHO date ranges and label indication searches allow no name.
    /// Drugs@FDA searches brands and ingredients,
    /// exact names before word matches. openFDA labels search exact then word brand/generic names,
    /// then label text (including indications) when no names match; source matches need not be exact names.
    /// Harmonized recall names match but never become display names.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    #[serde(
        default,
        deserialize_with = "present_query",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    query: Option<Text>,
    /// Literal label indication; openFDA labels only.
    /// Search the native Indications and Usage field without disease synonym expansion.
    /// An optional name query further narrows the same source search.
    #[serde(
        default,
        deserialize_with = "present_query",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::OpenfdaLabel)))]
    indication: Option<Text>,
    /// Literal published recalling firm phrase; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    firm: Option<Text>,
    /// Native FDA classification; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    classification: Option<super::DrugRecallClassification>,
    /// Literal native recall status; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    status: Option<Text>,
    /// Phrase in the published recall reason; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    reason: Option<Text>,
    /// Inclusive initiation day, YYYY-MM-DD; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    initiated_from: Option<biodata::PartialDate>,
    /// Inclusive initiation day, YYYY-MM-DD; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::FdaDrugRecall)))]
    initiated_to: Option<biodata::PartialDate>,

    /// Inclusive WHO prequalification lower bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::WhoPq)))]
    prequalified_from: Option<biodata::PartialDate>,
    /// Inclusive WHO prequalification upper bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::WhoPq)))]
    prequalified_to: Option<biodata::PartialDate>,

    /// Selected source.
    ///
    /// After an empty name lookup, existing sources may supply aliases.
    #[serde(default = "default_source")]
    source: DrugSource,
    /// Explicit ChEMBL native name selector; omitted means preferred name.
    #[serde(
        default,
        deserialize_with = "selected_name",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "super::DrugNameSelector"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = crate::Source::Chembl)))]
    name_selector: Option<super::DrugNameSelector>,
    /// Native candidate rows per page.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native rows to skip.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(DrugSearch, SearchFields);
impl TryFrom<SearchFields> for DrugSearch {
    type Error = QueryInvalid;
    fn try_from(fields: SearchFields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&fields.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if fields.indication.is_some() && fields.source != DrugSource::OpenfdaLabel {
            return Err(QueryInvalid::new(&["indication", "source"]));
        }
        if fields.name_selector.is_some() && fields.source != DrugSource::Chembl {
            return Err(QueryInvalid::new(&["name_selector", "source"]));
        }
        let who_dates = fields.prequalified_from.is_some() || fields.prequalified_to.is_some();
        if who_dates && fields.source != DrugSource::WhoPq {
            return Err(QueryInvalid::new(&[
                "prequalified_from",
                "prequalified_to",
                "source",
            ]));
        }
        if fields.source == DrugSource::WhoPq && fields.query.is_none() && !who_dates {
            return Err(
                QueryInvalid::new(&["query", "prequalified_from", "prequalified_to"])
                    .with_message(crate::ErrorMessage::WhoSearchArguments),
            );
        }
        if fields
            .prequalified_from
            .zip(fields.prequalified_to)
            .is_some_and(|(a, b)| {
                (a.year(), a.month().unwrap_or(1), a.day().unwrap_or(1))
                    > (b.year(), b.month().unwrap_or(12), b.day().unwrap_or(31))
            })
        {
            return Err(QueryInvalid::new(&["prequalified_from", "prequalified_to"]));
        }
        super::recall::validate_search(
            fields.source,
            fields.query.is_some()
                || fields.indication.is_some()
                || (fields.source == DrugSource::WhoPq && who_dates),
            fields.firm.is_some()
                || fields.classification.is_some()
                || fields.status.is_some()
                || fields.reason.is_some()
                || fields.initiated_from.is_some()
                || fields.initiated_to.is_some(),
            fields.initiated_from,
            fields.initiated_to,
        )?;
        Ok(Self(fields))
    }
}
impl DrugSearch {
    /// Retry a source-stated name while preserving every native filter and page selector.
    pub fn with_name(&self, name: Text) -> Self {
        let mut query = self.clone();
        query.0.query = Some(name);
        query
    }
    /// Select the native ChEMBL synonym filter without changing the request's other fields.
    pub fn with_name_selector(
        &self,
        selector: super::DrugNameSelector,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.0.clone();
        fields.name_selector = Some(selector);
        fields.try_into()
    }
    /// Whether the caller chose a literal ChEMBL name filter.
    pub fn explicit_name_selector(&self) -> bool {
        self.0.name_selector.is_some()
    }
    /// Inclusive WHO prequalification lower bound.
    pub fn prequalified_from(&self) -> Option<biodata::PartialDate> {
        self.0.prequalified_from
    }
    /// Inclusive WHO prequalification upper bound.
    pub fn prequalified_to(&self) -> Option<biodata::PartialDate> {
        self.0.prequalified_to
    }
    /// Construct a literal name search through the same checked fields as every input surface.
    pub fn named(query: Text, source: DrugSource, limit: u32) -> Result<Self, QueryInvalid> {
        SearchFields {
            query: Some(query),
            indication: None,
            source,
            limit,
            offset: 0,
            firm: None,
            classification: None,
            status: None,
            reason: None,
            initiated_from: None,
            initiated_to: None,
            prequalified_from: None,
            prequalified_to: None,
            name_selector: None,
        }
        .try_into()
    }
    /// Checked native selector, with preferred name as the ChEMBL default.
    pub fn name_selector(&self) -> super::DrugNameSelector {
        self.0
            .name_selector
            .unwrap_or(super::DrugNameSelector::Preferred)
    }
}
fn present_query<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Text>, D::Error> {
    Text::deserialize(d).map(Some)
}

fn selected_name<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<super::DrugNameSelector>, D::Error> {
    super::DrugNameSelector::deserialize(d).map(Some)
}
impl DrugSearch {
    /// Literal source query, without rewriting.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Literal native Indications and Usage phrase; never an inferred indication.
    pub fn indication(&self) -> Option<&Text> {
        self.0.indication.as_ref()
    }
    /// Selected native recall filter.
    pub fn firm(&self) -> Option<&Text> {
        self.0.firm.as_ref()
    }
    /// Selected native recall filter.
    pub fn classification(&self) -> Option<super::DrugRecallClassification> {
        self.0.classification
    }
    /// Selected native recall filter.
    pub fn status(&self) -> Option<&Text> {
        self.0.status.as_ref()
    }
    /// Selected native recall filter.
    pub fn reason(&self) -> Option<&Text> {
        self.0.reason.as_ref()
    }
    /// Selected native recall filter.
    pub fn initiated_from(&self) -> Option<biodata::PartialDate> {
        self.0.initiated_from
    }
    /// Selected native recall filter.
    pub fn initiated_to(&self) -> Option<biodata::PartialDate> {
        self.0.initiated_to
    }

    /// Single selected native source.
    pub fn source(&self) -> DrugSource {
        self.0.source
    }
    /// Native page bound.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for DrugSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}
