use super::{AdverseEventAggregate, AdverseEventSection, AdverseEventSerious, AdverseEventSource};
use crate::QueryInvalid;
use biodata::{PartialDate, ReportId, Text};
use serde::{Deserialize, Serialize};
fn default_source() -> AdverseEventSource {
    AdverseEventSource::Faers
}
fn default_limit() -> u32 {
    5
}
#[doc = " Search five report headers by default. MAUDE accepts device text or a manufacturer, \
    product-code or problem filter. One report may list several devices; filters may match \
    different devices. VAERS accepts MMR, COVID19, COVID-19 vaccine or \
    one exact qualified native influenza code, matched without case; use influenza to list \
    codes and labels. Tables remain separate and are never summed. One report may list \
    several vaccines. VAERS returns five reported symptom rows by default; seriousness and \
    age select independent tables. Counts are reports, not people. Reports do not show that a \
    vaccine caused an event and cannot measure risk or compare vaccine safety."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct AdverseEventSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    #[doc = "Drug, device or vaccine.\nLiteral drug text, MAUDE device phrase or exact supported VAERS vaccine."]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    text: Option<Text>,
    #[doc = "FAERS drug names; OR.\nFAERS literal drug name; repeat --drug for OR. OR inside one quoted value stays literal. \
        Other filters AND with the names. BioMCP caps the escaped native query, including filters, \
        at FAERS_QUERY_MAX_BYTES (4096 bytes)."]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    drug: Vec<Text>,
    #[doc = "MAUDE manufacturer.\n\
        MAUDE manufacturer phrase on device.manufacturer_d_name. A report may list several devices; filters may match different devices."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = super::AdverseEventSource::Maude.required_search_component())))]
    manufacturer: Option<Text>,
    #[doc = "MAUDE product codes; OR.\nMAUDE exact native device.device_report_product_code; repeated codes join with OR."]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = super::AdverseEventSource::Maude.required_search_component())))]
    product_code: Vec<Text>,
    #[doc = "MAUDE product problem.\nMAUDE phrase on native product_problems."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = super::AdverseEventSource::Maude.required_search_component())))]
    problem: Option<Text>,
    #[doc = "Source; default FAERS.\nFAERS by default; VAERS accepts only exact supported vaccine names."]
    #[serde(default = "default_source")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-components" = super::AdverseEventSource::search_components())))]
    source: AdverseEventSource,
    #[doc = "FAERS reaction counts.\nUse reactions for source-native count buckets across matching reports. \
        Counts are not rates or causal associations; a report may contribute to several terms. \
        Requires MedDRA and offset zero."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = super::AdverseEventSection::Reactions.required_component())))]
    aggregate: Option<AdverseEventAggregate>,
    /// Independent VAERS table; omitted means reported symptoms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "VAERS table.", extend("x-positional" = 1)))]
    section: Option<super::VaersSection>,
    #[doc = "VAERS snapshot.\nRetained VAERS table for provider-free continuation."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    snapshot: Option<Text>,
    #[doc = "FAERS MedDRA phrase.\nFAERS MedDRA term phrase match, not exact equality."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-required-component" = super::AdverseEventSection::Reactions.required_component())))]
    reaction: Option<Text>,
    #[doc = "FAERS seriousness flag.\nFAERS report flag: any uses the source seriousness flag; the specific flags can overlap. Never a reaction outcome."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    serious: Option<AdverseEventSerious>,
    #[doc = "First date, YYYY-MM[-DD].\nInclusive FAERS or MAUDE FDA receipt lower bound, YYYY-MM-DD or YYYY-MM; a month starts on its first day."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "date")))]
    received_from: Option<PartialDate>,
    #[doc = "Last date, YYYY-MM[-DD].\nInclusive FAERS or MAUDE FDA receipt upper bound, YYYY-MM-DD or YYYY-MM; a month ends on its last day."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "date")))]
    received_to: Option<PartialDate>,
    /// Native report headers, counted FAERS reaction terms or VAERS rows; default five.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native report row offset; reaction counts require zero. VAERS needs its snapshot.
    #[serde(default)]
    offset: u32,
}
impl SearchFields {
    fn validate_aggregate(&self) -> Result<(), QueryInvalid> {
        if self.aggregate.is_some() && self.source != AdverseEventSource::Faers {
            return Err(QueryInvalid::new(&["source", "aggregate"]));
        }
        if self.aggregate.is_some() && self.offset != 0 {
            return Err(QueryInvalid::new(&["aggregate", "offset"]));
        }
        Ok(())
    }
}
crate::query::validation::checked_query!(AdverseEventSearch, SearchFields);
impl TryFrom<SearchFields> for AdverseEventSearch {
    type Error = QueryInvalid;
    fn try_from(mut v: SearchFields) -> Result<Self, Self::Error> {
        v.validate_aggregate()?;
        let maude_filter =
            v.manufacturer.is_some() || !v.product_code.is_empty() || v.problem.is_some();
        if v.source != AdverseEventSource::Maude && maude_filter {
            return Err(QueryInvalid::new(&[
                "source",
                "manufacturer",
                "product_code",
                "problem",
            ]));
        }
        if v.text.is_none()
            && v.drug.is_empty()
            && !(v.source == AdverseEventSource::Maude && maude_filter)
        {
            return Err(QueryInvalid::new(&[
                "text",
                "manufacturer",
                "product_code",
                "problem",
            ]));
        }
        if v.source != AdverseEventSource::Faers && !v.drug.is_empty() {
            return Err(QueryInvalid::new(&["source", "drug"]));
        }
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        v.received_from = super::receipt::day(v.received_from, false, "received_from")?;
        v.received_to = super::receipt::day(v.received_to, true, "received_to")?;
        if v.received_from
            .zip(v.received_to)
            .is_some_and(|(a, b)| a.to_string() > b.to_string())
        {
            return Err(QueryInvalid::new(&["received_from", "received_to"]));
        }
        if v.source != AdverseEventSource::Vaers && (v.section.is_some() || v.snapshot.is_some()) {
            return Err(QueryInvalid::new(&["source", "section", "snapshot"]));
        }
        if v.source == AdverseEventSource::Vaers
            && v.text
                .as_ref()
                .and_then(|text| super::vaers::vaers_vaccine_code(text.as_str()))
                .is_none()
        {
            return Err(QueryInvalid::new(&["source", "text"]));
        }
        if v.source != AdverseEventSource::Faers && (v.reaction.is_some() || v.serious.is_some()) {
            return Err(QueryInvalid::new(&[
                "source",
                "reaction",
                "serious",
                "received_from",
                "received_to",
            ]));
        }
        if v.source == AdverseEventSource::Vaers
            && (v.received_from.is_some() || v.received_to.is_some())
        {
            return Err(QueryInvalid::new(&[
                "source",
                "received_from",
                "received_to",
            ]));
        }
        let query = Self(v);
        if query.source() == AdverseEventSource::Faers
            && query.faers_expression().len() > super::faers::FAERS_QUERY_MAX_BYTES
        {
            return Err(QueryInvalid::new(&[
                "text",
                "drug",
                "reaction",
                "received_from",
                "received_to",
            ])
            .with_message(crate::ErrorMessage::FaersQueryLength));
        }
        Ok(query)
    }
}
impl AdverseEventSearch {
    /// Discover MAUDE reports for one source-listed native product code.
    pub fn maude_product(code: Text) -> Result<Self, QueryInvalid> {
        Self::try_from(SearchFields {
            text: None,
            drug: Vec::new(),
            manufacturer: None,
            product_code: vec![code],
            problem: None,
            source: AdverseEventSource::Maude,
            aggregate: None,
            section: None,
            snapshot: None,
            reaction: None,
            serious: None,
            received_from: None,
            received_to: None,
            limit: default_limit(),
            offset: 0,
        })
    }

    /// Requested source-native aggregation, distinct from report discovery.
    pub fn aggregate(&self) -> Option<AdverseEventAggregate> {
        self.0.aggregate
    }

    /// Selected VAERS grouping.
    pub fn vaers_section(&self) -> super::VaersSection {
        self.0.section.unwrap_or(super::VaersSection::Symptoms)
    }
    /// Checked retained table identity.
    pub fn snapshot(&self) -> Option<&Text> {
        self.0.snapshot.as_ref()
    }
    /// Bind navigation to one native table.
    pub fn with_snapshot(&self, snapshot: Text) -> Self {
        let mut q = self.clone();
        q.0.snapshot = Some(snapshot);
        q
    }
    /// Begin a separate grouping without borrowing its sibling denominator.
    pub fn with_vaers_section(&self, section: super::VaersSection) -> Self {
        let mut q = self.clone();
        q.0.section = Some(section);
        q.0.snapshot = None;
        q.0.offset = 0;
        q
    }

    /// Exact supported vaccine code, never an inferred code bridge.
    pub fn vaccine(&self) -> Option<&'static str> {
        self.0
            .text
            .as_ref()
            .and_then(|text| super::vaers::vaers_vaccine_code(text.as_str()))
    }
    /// Literal source text.
    pub fn text(&self) -> Option<&Text> {
        self.0.text.as_ref()
    }
    /// Explicit FAERS literal drug names, ORed with any positional text.
    pub fn drug(&self) -> &[Text] {
        &self.0.drug
    }
    /// Native MAUDE manufacturer phrase.
    pub fn manufacturer(&self) -> Option<&Text> {
        self.0.manufacturer.as_ref()
    }
    /// Exact native MAUDE product codes joined with OR.
    pub fn product_code(&self) -> &[Text] {
        &self.0.product_code
    }
    /// Native MAUDE product problem phrase.
    pub fn problem(&self) -> Option<&Text> {
        self.0.problem.as_ref()
    }
    /// Selected source.
    pub fn source(&self) -> AdverseEventSource {
        self.0.source
    }
    /// Reaction phrase.
    pub fn reaction(&self) -> Option<&Text> {
        self.0.reaction.as_ref()
    }
    /// Report flag.
    pub fn serious(&self) -> Option<AdverseEventSerious> {
        self.0.serious
    }
    /// Inclusive lower first-receipt date.
    pub fn received_from(&self) -> Option<PartialDate> {
        self.0.received_from
    }
    /// Inclusive upper first-receipt date.
    pub fn received_to(&self) -> Option<PartialDate> {
        self.0.received_to
    }
    /// Report page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Report offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for AdverseEventSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.snapshot = None;
    }
}

/// Exact native report identity and an optional independently paged section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct AdverseEventGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Published report ID with no source prefix.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: ReportId,
    /// FAERS by default; MAUDE is explicit.
    #[serde(default = "default_source")]
    #[cfg_attr(feature = "schema", schemars(with="str", extend("enum"=["faers", "maude"])))]
    source: AdverseEventSource,
    /// FAERS owns drugs, reactions and identifiers; MAUDE owns devices and narrative. Omitted means card only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 1)))]
    section: Option<AdverseEventSection>,
    /// Checked retained report snapshot for provider-free continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    snapshot: Option<Text>,
    /// Absolute source-array index; required only for identifier sections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    drug_index: Option<u32>,
    /// Whole source rows per section page.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    section_limit: u32,
    /// Offset within the selected native section.
    #[serde(default)]
    section_offset: u32,
}
crate::query::validation::checked_query!(AdverseEventGet, GetFields);
impl TryFrom<GetFields> for AdverseEventGet {
    type Error = QueryInvalid;
    fn try_from(v: GetFields) -> Result<Self, Self::Error> {
        if v.source == AdverseEventSource::Vaers {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.source == AdverseEventSource::Maude
            && !v.id.as_str().bytes().all(|b| b.is_ascii_digit())
        {
            return Err(QueryInvalid::new(&["source", "id"]));
        }
        if !(1..=50).contains(&v.section_limit) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        if v.section.is_some_and(|s| s.source() != v.source) {
            return Err(QueryInvalid::new(&["section", "source"]));
        }
        if v.drug_index.is_some() != v.section.is_some_and(AdverseEventSection::is_identifier) {
            return Err(QueryInvalid::new(&["drug_index", "section"]));
        }
        Ok(Self(v))
    }
}
impl AdverseEventGet {
    /// Check a source-owned report identity with the default card bounds.
    pub fn new(id: ReportId, source: AdverseEventSource) -> Result<Self, QueryInvalid> {
        Self::try_from(GetFields {
            id,
            source,
            section: None,
            snapshot: None,
            drug_index: None,
            section_limit: default_limit(),
            section_offset: 0,
        })
    }

    /// Exact published identity.
    pub fn id(&self) -> &ReportId {
        &self.0.id
    }
    /// Explicit family.
    pub fn source(&self) -> AdverseEventSource {
        self.0.source
    }
    /// Named section.
    pub fn section(&self) -> Option<AdverseEventSection> {
        self.0.section
    }
    /// Retained report identity, absent for an initial get.
    pub fn snapshot(&self) -> Option<&Text> {
        self.0.snapshot.as_ref()
    }
    /// Bind all subsequent section pages to this checked report.
    pub fn with_snapshot(&self, snapshot: Text) -> Self {
        let mut q = self.clone();
        q.0.snapshot = Some(snapshot);
        q
    }

    /// Absolute native drug index, only for identifier sections.
    pub fn drug_index(&self) -> Option<u32> {
        self.0.drug_index
    }
    /// Whole row limit.
    pub fn section_limit(&self) -> u32 {
        self.0.section_limit
    }
    /// Native section offset.
    pub fn section_offset(&self) -> u32 {
        self.0.section_offset
    }
    /// Continue while preserving source, identity and drug index.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut q = self.clone();
        q.0.section_offset = offset;
        q
    }
    /// Select a section through the same validation used at deserialization.
    pub fn with_section(
        &self,
        section: AdverseEventSection,
        drug_index: Option<u32>,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.0.clone();
        fields.section = Some(section);
        fields.drug_index = drug_index;
        fields.section_offset = 0;
        fields.try_into()
    }
}
impl crate::Paged for AdverseEventGet {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.section_offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.snapshot = None;
    }
}
