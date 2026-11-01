use super::DrugSource;
use crate::{CivicClaimKind, QueryInvalid};
use biodata::{DrugId, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Explicit drug disclosures; identity cards never expand into these sections.
    DrugSection, SourceUnknown {
        /// All native identity cross-references and product links, mechanically paged.
        Xrefs => "xrefs",
        /// Native target and mechanism relations.
        Targets => "targets",
        /// Native indication wording.
        Indications => "indications",
        /// Native CIViC therapy evidence and profile context.
        Civic => "civic",
        /// One exact native label heading.
        Label => "label",
        /// Native application and product history rows.
        Approvals => "approvals",
        /// Source-scoped regulator events.
        Regulatory => "regulatory",
        /// Native DDInter partner rows and levels; five rows per default page, with explicit section limits preserved.
        Interactions => "interactions",
    }
}
fn default_source() -> DrugSource {
    DrugSource::Mychem
}
fn default_limit() -> u32 {
    10
}
#[doc = " Search one drug source. FDA drug recalls accept a name or native firm, classification, \
    status, reason and initiation dates; counts are recall records. Other sources require a name."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct DrugSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal name phrase; optional for FDA drug recalls. Drugs@FDA searches brands and ingredients,
    /// exact names before word matches. Harmonized recall names match but never become display names.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    #[serde(
        default,
        deserialize_with = "present_query",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    query: Option<Text>,
    /// Literal published recalling firm phrase; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firm: Option<Text>,
    /// Native FDA classification; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    classification: Option<super::DrugRecallClassification>,
    /// Literal native recall status; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<Text>,
    /// Phrase in the published recall reason; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<Text>,
    /// Inclusive initiation day, YYYY-MM-DD; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initiated_from: Option<biodata::PartialDate>,
    /// Inclusive initiation day, YYYY-MM-DD; drug recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initiated_to: Option<biodata::PartialDate>,

    /// Exactly one source; default MyChem never fans out.
    #[serde(default = "default_source")]
    source: DrugSource,
    /// Explicit ChEMBL native name selector; omitted means preferred name.
    #[serde(
        default,
        deserialize_with = "selected_name",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "super::DrugNameSelector"))]
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
        if fields.name_selector.is_some() && fields.source != DrugSource::Chembl {
            return Err(QueryInvalid::new(&["name_selector", "source"]));
        }
        super::recall::validate_search(
            fields.source,
            fields.query.is_some(),
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
    /// Checked native selector, with preferred name as the ChEMBL default.
    pub fn name_selector(&self) -> super::DrugNameSelector {
        self.0
            .name_selector
            .unwrap_or(super::DrugNameSelector::Preferred)
    }
}
/// Exact native ID retrieval with an explicitly selected source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct DrugGet {
    #[serde(flatten)]
    fields: GetFields,
    #[serde(skip)]
    native_id: NativeDrugId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum NativeDrugId {
    Recall(biodata::DrugRecallNumber),
    Civic(std::num::NonZeroU32),
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Published source-native identifier, never SOURCE:ID or a selected name.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    input: DrugId,
    /// Required source; there is no default exact-ID resolver.
    source: DrugSource,
    /// Explicit named disclosures; empty means identity only.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 1)))]
    section: Box<[DrugSection]>,
    /// Named section page size, refused on an identity-only card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Named section continuation, refused on an identity-only card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_offset: Option<u32>,
    /// Exact native label heading, required only for the label section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label_section: Option<Text>,
    /// Explicit native application product number, only for approvals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    product_number: Option<Text>,
    /// Selected native CIViC molecular profile for a drug-scoped claim stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 2147483647)))]
    civic_profile: Option<u64>,
    /// Independent native evidence or assertion collection within the selected profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_kind: Option<CivicClaimKind>,
    /// Native continuation cursor for that exact drug, profile and claim collection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_cursor: Option<Text>,
}
crate::query::validation::checked_query!(DrugGet, GetFields);
impl TryFrom<GetFields> for DrugGet {
    type Error = QueryInvalid;
    fn try_from(fields: GetFields) -> Result<Self, Self::Error> {
        validate_sections(&fields)?;
        let native_id = match fields.source {
            DrugSource::FdaDrugRecall => NativeDrugId::Recall(
                fields
                    .input
                    .to_string()
                    .parse()
                    .map_err(|_| QueryInvalid::new(&["source", "input"]))?,
            ),
            DrugSource::Civic => NativeDrugId::Civic(
                civic_id(&fields.input.to_string())
                    .ok_or_else(|| QueryInvalid::new(&["source", "input"]))?,
            ),
            source => {
                if !source.accepts(&fields.input) {
                    return Err(QueryInvalid::new(&["source", "input"]));
                }
                NativeDrugId::Other
            }
        };
        validate_civic(&fields)?;
        Ok(Self { fields, native_id })
    }
}
fn validate_sections(fields: &GetFields) -> Result<(), QueryInvalid> {
    if fields.section_limit.is_some_and(|n| !(1..=50).contains(&n)) {
        return Err(QueryInvalid::new(&["section_limit"]));
    }
    if fields.section.is_empty()
        && (fields.section_limit.is_some() || fields.section_offset.is_some())
    {
        return Err(QueryInvalid::new(&[
            "section",
            "section_limit",
            "section_offset",
        ]));
    }
    if fields.section.contains(&DrugSection::Label) != fields.label_section.is_some() {
        return Err(QueryInvalid::new(&["section", "label_section"]));
    }
    if fields.product_number.is_some() && !fields.section.contains(&DrugSection::Approvals) {
        return Err(QueryInvalid::new(&["product_number", "section"]));
    }
    if fields.source == DrugSource::FdaDrugRecall && !fields.section.is_empty() {
        return Err(QueryInvalid::new(&["source", "section"]));
    }
    Ok(())
}
fn validate_civic(fields: &GetFields) -> Result<(), QueryInvalid> {
    if fields
        .civic_profile
        .is_some_and(|id| id == 0 || id > i32::MAX as u64)
    {
        return Err(QueryInvalid::new(&["civic_profile"]));
    }
    if fields.civic_profile.is_some() && fields.civic_kind.is_none() {
        return Err(QueryInvalid::new(&["civic_profile", "civic_kind"]));
    }
    if fields.civic_cursor.is_some() && fields.civic_kind.is_none() {
        return Err(QueryInvalid::new(&["civic_cursor", "civic_kind"]));
    }
    if fields.civic_kind.is_some()
        && (fields.source != DrugSource::Civic || !fields.section.contains(&DrugSection::Civic))
    {
        return Err(QueryInvalid::new(&["civic_kind", "source", "section"]));
    }
    Ok(())
}
impl DrugGet {
    fn with_fields(&self, fields: GetFields) -> Result<Self, QueryInvalid> {
        validate_sections(&fields)?;
        validate_civic(&fields)?;
        Ok(Self {
            fields,
            native_id: self.native_id.clone(),
        })
    }
    /// FDA recall identity retained by the checked boundary.
    pub fn recall_number(&self) -> Option<&biodata::DrugRecallNumber> {
        match &self.native_id {
            NativeDrugId::Recall(id) => Some(id),
            _ => None,
        }
    }
    /// CIViC therapy number retained by the checked boundary.
    pub fn civic_id(&self) -> Option<std::num::NonZeroU32> {
        match self.native_id {
            NativeDrugId::Civic(id) => Some(id),
            _ => None,
        }
    }
    /// Select one exact native identity, with no sections requested.
    pub fn new(input: DrugId, source: DrugSource) -> Result<Self, QueryInvalid> {
        GetFields {
            input,
            source,
            section: Box::default(),
            section_limit: None,
            section_offset: None,
            label_section: None,
            product_number: None,
            civic_profile: None,
            civic_kind: None,
            civic_cursor: None,
        }
        .try_into()
    }
    /// Continue one native claim collection while preserving selected drug, source and section paging.
    pub fn with_civic_page(
        &self,
        profile: u64,
        kind: CivicClaimKind,
        cursor: Option<Text>,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.civic_profile = Some(profile);
        fields.civic_kind = Some(kind);
        fields.civic_cursor = cursor;
        self.with_fields(fields)
    }
    /// Continue a therapy-wide claim stream without selecting a molecular profile.
    pub fn with_civic_stream(
        &self,
        kind: CivicClaimKind,
        cursor: Option<Text>,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.civic_kind = Some(kind);
        fields.civic_cursor = cursor;
        self.with_fields(fields)
    }
    /// Selected native drug-scoped CIViC molecular profile.
    pub fn civic_profile(&self) -> Option<u64> {
        self.fields.civic_profile
    }
    /// Selected independent native claim collection.
    pub fn civic_kind(&self) -> Option<CivicClaimKind> {
        self.fields.civic_kind
    }
    /// Original native cursor for that selected claim collection.
    pub fn civic_cursor(&self) -> Option<&Text> {
        self.fields.civic_cursor.as_ref()
    }
    /// Unchanged published identifier.
    pub fn input(&self) -> &DrugId {
        &self.fields.input
    }
    /// Required selected source.
    pub fn source(&self) -> DrugSource {
        self.fields.source
    }
    /// Explicit page bound; command handlers apply their existing default when absent.
    pub fn section_limit(&self) -> Option<u32> {
        self.fields.section_limit
    }
    /// Explicit section continuation offset.
    pub fn section_offset(&self) -> Option<u32> {
        self.fields.section_offset
    }
}
impl DrugSource {
    /// Check only published ID grammar for this endpoint, without inferring identity level or joining records.
    pub fn accepts(self, id: &DrugId) -> bool {
        let id = id.to_string();
        if id.split_once(':').is_some_and(|(prefix, _)| {
            Self::ALL
                .into_iter()
                .any(|source| source.as_str().eq_ignore_ascii_case(prefix))
        }) {
            return false;
        }
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        match self {
            Self::FdaDrugRecall => id.parse::<biodata::DrugRecallNumber>().is_ok(),
            Self::WhoPq => false,
            Self::Mychem => true,
            Self::Chembl | Self::Opentargets => id.strip_prefix("CHEMBL").is_some_and(digits),
            Self::Civic => civic_id(&id).is_some(),
            Self::Ddinter => id.strip_prefix("DDInter").is_some_and(digits),
            Self::Drugsfda => ["NDA", "ANDA", "BLA"].iter().any(|p| {
                id.strip_prefix(p)
                    .is_some_and(|s| s.len() == 6 && digits(s))
            }),
            Self::Ema => id
                .strip_prefix("EMEA/H/C/")
                .is_some_and(|s| s.len() == 6 && digits(s)),
            Self::OpenfdaLabel => {
                let pieces: Vec<_> = id.split('-').collect();
                pieces.iter().map(|s| s.len()).eq([8, 4, 4, 4, 12])
                    && pieces
                        .iter()
                        .all(|s| s.bytes().all(|b| b.is_ascii_hexdigit()))
            }
        }
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

impl DrugGet {
    /// Explicit requested disclosures.
    pub fn sections(&self) -> &[DrugSection] {
        &self.fields.section
    }
}

impl DrugGet {
    /// Continue the same named section without changing source, identity or section selectors.
    pub fn at_section_offset(&self, offset: u32) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section_offset = Some(offset);
        self.with_fields(fields)
    }
    /// Exact selected native label heading.
    pub fn label_section(&self) -> Option<&Text> {
        self.fields.label_section.as_ref()
    }
    /// Exact application product selector.
    pub fn product_number(&self) -> Option<&Text> {
        self.fields.product_number.as_ref()
    }
}

impl DrugGet {
    /// Select one named disclosure without changing the exact source identity.
    pub fn for_section(&self, section: DrugSection) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section = vec![section].into();
        self.with_fields(fields)
    }
}

impl DrugGet {
    /// Select an explicit native label heading for the same source identity.
    pub fn for_label(&self, heading: Text) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section = vec![DrugSection::Label].into();
        fields.label_section = Some(heading);
        self.with_fields(fields)
    }
}

fn civic_id(value: &str) -> Option<std::num::NonZeroU32> {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value
        .parse::<std::num::NonZeroU32>()
        .ok()
        .filter(|id| id.get() <= i32::MAX as u32)
}
