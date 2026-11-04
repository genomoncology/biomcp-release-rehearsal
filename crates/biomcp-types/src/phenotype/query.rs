use crate::QueryInvalid;
use biodata::{HpId, Text};
use serde::{Deserialize, Serialize};
biodata::wire_enum! { described
    /// One independent term publisher.
    PhenotypeSource {
        /// Direct HPO term metadata.
        Hpo => "hpo",
        /// Monarch term search and associations.
        Monarch => "monarch",
    }
}
biodata::wire_enum! { described
    /// A named source term section.
    PhenotypeSection {
        /// Native synonyms.
        Synonyms => "synonyms",
        /// Native cross references.
        Xrefs => "xrefs",
        /// Native direct parent and child links.
        Hierarchy => "hierarchy",
        /// Native disease association rows.
        Diseases => "diseases",
        /// Native human gene association rows.
        Genes => "genes",
    }
}
fn monarch() -> PhenotypeSource {
    PhenotypeSource::Monarch
}
fn five() -> u32 {
    5
}
/// Search literal human HP term candidates through Monarch with bounded native pages. A term lookup answers terminology and does not establish a diagnosis.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct PhenotypeSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Unchanged literal term search text.
    #[cfg_attr(feature = "schema", schemars(length(min=1), extend("x-positional" = 0)))]
    text: Box<str>,
    /// Monarch only; HPO search is unsupported.
    #[serde(default = "monarch")]
    source: PhenotypeSource,
    /// Maximum native headers, default five.
    #[serde(default = "five")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native source row offset.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(PhenotypeSearch, SearchFields);
impl TryFrom<SearchFields> for PhenotypeSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        if v.source != PhenotypeSource::Monarch {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.text.trim().is_empty() {
            return Err(QueryInvalid::new(&["text"]));
        }
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(v))
    }
}
#[doc = " Read an exact HP term from HPO or Monarch, or page native source sections. A term lookup \
    answers terminology; an association answers what its publisher asserts. Neither \
    establishes a diagnosis."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct PhenotypeGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Exact HP identifier.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: HpId,
    #[doc = " HPO or Monarch for cards, synonyms and xrefs; Monarch only for hierarchy, diseases and \
        genes. Defaults to HPO for cards, synonyms and xrefs, otherwise Monarch."]
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PhenotypeSource"))]
    source: Option<PhenotypeSource>,
    /// Absent means the source identity card.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PhenotypeSection", extend("x-positional" = 1)))]
    section: Option<PhenotypeSection>,
    /// Default five text/link rows or one complete association row.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Native section row offset.
    #[serde(default)]
    section_offset: u32,
    /// Retained exact source reply for local text and link pages; unsupported for diseases and genes.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    snapshot: Option<Text>,
}
crate::query::validation::checked_query!(PhenotypeGet, GetFields);
impl TryFrom<GetFields> for PhenotypeGet {
    type Error = QueryInvalid;
    fn try_from(mut v: GetFields) -> Result<Self, Self::Error> {
        let (source, _) = defaults(v.section);
        if v.source == Some(PhenotypeSource::Hpo)
            && v.section
                .is_some_and(|section| !section.sources().contains(&crate::Source::Hpo))
        {
            return Err(QueryInvalid::new(&["source", "section"]));
        }
        if v.section_limit.is_some_and(|n| !(1..=50).contains(&n)) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        if matches!(
            v.section,
            Some(PhenotypeSection::Diseases | PhenotypeSection::Genes)
        ) && v.snapshot.is_some()
        {
            return Err(QueryInvalid::new(&["section", "snapshot"]));
        }
        v.source.get_or_insert(source);
        Ok(Self(v))
    }
}
fn defaults(section: Option<PhenotypeSection>) -> (PhenotypeSource, u32) {
    match section {
        Some(PhenotypeSection::Diseases | PhenotypeSection::Genes) => (PhenotypeSource::Monarch, 1),
        Some(PhenotypeSection::Hierarchy) => (PhenotypeSource::Monarch, five()),
        _ => (PhenotypeSource::Hpo, five()),
    }
}
impl PhenotypeSearch {
    /// Literal caller text without normalization.
    pub fn text(&self) -> &str {
        &self.0.text
    }
    /// Selected source.
    pub fn source(&self) -> PhenotypeSource {
        self.0.source
    }
    /// Bounded native page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for PhenotypeSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

impl PhenotypeGet {
    /// Publisher selected when the caller omits the source flag for this section.
    pub fn default_source(&self) -> PhenotypeSource {
        defaults(self.section()).0
    }
    /// Page size selected when the caller omits the section limit.
    pub fn default_section_limit(&self) -> u32 {
        defaults(self.section()).1
    }

    /// Construct a default direct HPO card request from a checked identity.
    pub fn new(id: HpId) -> Self {
        Self(GetFields {
            id,
            source: Some(PhenotypeSource::Hpo),
            section: None,
            section_limit: None,
            section_offset: 0,
            snapshot: None,
        })
    }
    /// Checked term identity.
    pub fn id(&self) -> &HpId {
        &self.0.id
    }
    /// Resolved independent publisher.
    pub fn source(&self) -> PhenotypeSource {
        self.0.source.unwrap_or(PhenotypeSource::Hpo)
    }
    /// Selected section or the card.
    pub fn section(&self) -> Option<PhenotypeSection> {
        self.0.section
    }
    /// Checked bounded section size.
    pub fn section_limit(&self) -> u32 {
        self.0
            .section_limit
            .unwrap_or_else(|| defaults(self.section()).1)
    }
    /// Section offset.
    pub fn section_offset(&self) -> u32 {
        self.0.section_offset
    }
    /// Retained exact source reply selector.
    pub fn snapshot(&self) -> Option<&Text> {
        self.0.snapshot.as_ref()
    }
    /// Bind a card or local section to the same source reply.
    pub fn with_snapshot(&self, snapshot: Text) -> Self {
        let mut q = self.clone();
        q.0.snapshot = Some(snapshot);
        q
    }
    /// Select a source-available section without replacing the retained reply.
    pub fn with_section(&self, section: PhenotypeSection) -> Result<Self, QueryInvalid> {
        let mut fields = self.0.clone();
        fields.section = Some(section);
        fields.section_offset = 0;
        if matches!(
            section,
            PhenotypeSection::Diseases | PhenotypeSection::Genes
        ) {
            fields.snapshot = None;
        }
        Self::try_from(fields)
    }

    /// Continue the same section and publisher.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut q = self.clone();
        q.0.section_offset = offset;
        q
    }
}
impl crate::Paged for PhenotypeGet {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.section_offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.snapshot = None;
    }
}

impl PhenotypeSection {
    /// Publishers implementing this term section, including nondefault choices.
    pub const fn sources(self) -> &'static [crate::Source] {
        use crate::Source;
        match self {
            Self::Synonyms | Self::Xrefs => &[Source::Hpo, Source::Monarch],
            Self::Hierarchy | Self::Diseases | Self::Genes => &[Source::Monarch],
        }
    }
}
