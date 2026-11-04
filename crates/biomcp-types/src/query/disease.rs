use super::gene::{QueryInvalid, present};
use crate::Text;
use serde::{Deserialize, Serialize};
mod id;
mod source;
pub use id::DiseaseId;
pub use source::{DiseaseOntology, DiseaseSource};
biodata::wire_enum! { described
    /// One named disease page.
    DiseaseSection {
        /// Publisher scoped synonyms.
        Synonyms => "synonyms",
        /// Native cross references.
        Xrefs => "xrefs",
        /// Native parent relations.
        Hierarchy => "hierarchy",
        /// Native gene association rows.
        Genes => "genes",
        /// Native phenotype association rows.
        Phenotypes => "phenotypes",
        /// Native stage incidence rows.
        Incidence => "incidence",
        /// Annual survival rows.
        Survival => "survival",
    }
}
fn default_limit() -> u32 {
    5
}
fn get_source() -> DiseaseSource {
    DiseaseSource::MyDisease
}
/// Literal discovery text or an explicit SEER catalog page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct DiseaseSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal source search text; absent for the SEER site catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(length(min=1),extend("x-positional" = 0)))]
    text: Option<Box<str>>,
    /// One explicit source, default OLS.
    #[serde(default)]
    source: DiseaseSource,
    /// Exactly one ontology, default MONDO.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "DiseaseOntology", extend("default" = "mondo")))]
    ontology: Option<DiseaseOntology>,
    /// Maximum source rows on one page.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native row offset.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(DiseaseSearch, SearchFields);
impl TryFrom<SearchFields> for DiseaseSearch {
    type Error = QueryInvalid;
    fn try_from(mut v: SearchFields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if !matches!(v.source, DiseaseSource::Ols | DiseaseSource::Seer) {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.source == DiseaseSource::Ols && v.text.as_ref().is_none_or(|t| t.trim().is_empty()) {
            return Err(QueryInvalid::new(&["source", "text"]));
        }
        if v.source == DiseaseSource::Seer && (v.text.is_some() || v.ontology.is_some()) {
            return Err(QueryInvalid::new(&["source", "text", "ontology"]));
        }
        if v.source == DiseaseSource::Ols {
            v.ontology.get_or_insert_with(DiseaseOntology::default);
        }
        Ok(Self(v))
    }
}
/// Exact concept or native SEER site with independently paged section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct DiseaseGet {
    #[serde(flatten)]
    fields: GetFields<DiseaseId>,
    #[serde(skip)]
    record: Text,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "GetFields"))]
struct GetFields<I = Text> {
    /// Exact ontology CURIE or an explicit SEER catalog site ID.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: I,
    /// Default MyDisease; no automatic fallback.
    #[serde(default = "get_source")]
    source: DiseaseSource,
    /// One OLS ontology.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "DiseaseOntology", extend("default" = "mondo")))]
    ontology: Option<DiseaseOntology>,
    /// One named section; absent means identity card only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 1)))]
    section: Option<DiseaseSection>,
    /// Native rows per page: default 2 for Monarch genes, 1 for Monarch phenotypes, otherwise 5.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Native row offset within the selected section.
    #[serde(default)]
    section_offset: u32,
    /// Native sex code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sex: Option<Text>,
    /// Native race code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    race: Option<Text>,
    /// Native age-range code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    age_range: Option<Text>,
    /// Requested native dataset selector, not proof of returned membership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dataset: Option<Text>,
    /// Incidence-only native period code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    period: Option<Text>,
    /// Survival-only interval code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    interval: Option<Text>,
}
crate::query::validation::checked_query!(DiseaseGet, GetFields);
impl TryFrom<GetFields> for DiseaseGet {
    type Error = QueryInvalid;
    fn try_from(mut v: GetFields) -> Result<Self, Self::Error> {
        let statistic = matches!(
            v.section,
            Some(DiseaseSection::Incidence | DiseaseSection::Survival)
        );
        if v.section_limit
            .is_some_and(|limit| !(1..=50).contains(&limit))
        {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        let id = if v.source == DiseaseSource::Seer {
            if !statistic || !v.id.as_str().bytes().all(|b| b.is_ascii_digit()) {
                return Err(QueryInvalid::new(&["source", "id", "section"]));
            }
            DiseaseId::SeerSite(v.id.clone())
        } else {
            if statistic {
                return Err(QueryInvalid::new(&["source", "id", "section"]));
            }
            DiseaseId::Concept(
                v.id.as_str()
                    .parse()
                    .map_err(|_| QueryInvalid::new(&["source", "id", "section"]))?,
            )
        };
        if v.source != DiseaseSource::Seer
            && [
                &v.sex,
                &v.race,
                &v.age_range,
                &v.dataset,
                &v.period,
                &v.interval,
            ]
            .iter()
            .any(|v| v.is_some())
        {
            return Err(QueryInvalid::new(&[
                "source",
                "sex",
                "race",
                "age_range",
                "dataset",
                "period",
                "interval",
            ]));
        }
        if v.source != DiseaseSource::Ols && v.ontology.is_some() {
            return Err(QueryInvalid::new(&["source", "ontology"]));
        }
        if v.section != Some(DiseaseSection::Incidence) && v.period.is_some() {
            return Err(QueryInvalid::new(&["section", "period"]));
        }
        if v.section != Some(DiseaseSection::Survival) && v.interval.is_some() {
            return Err(QueryInvalid::new(&["section", "interval"]));
        }
        if v.source == DiseaseSource::Ols {
            v.ontology.get_or_insert_with(DiseaseOntology::default);
        }
        v.section_limit.get_or_insert(match (v.source, v.section) {
            (DiseaseSource::Monarch, Some(DiseaseSection::Genes)) => 2,
            (DiseaseSource::Monarch, Some(DiseaseSection::Phenotypes)) => 1,
            _ => default_limit(),
        });
        Ok(Self {
            record: v.id,
            fields: GetFields {
                id,
                source: v.source,
                ontology: v.ontology,
                section: v.section,
                section_limit: v.section_limit,
                section_offset: v.section_offset,
                sex: v.sex,
                race: v.race,
                age_range: v.age_range,
                dataset: v.dataset,
                period: v.period,
                interval: v.interval,
            },
        })
    }
}

impl DiseaseSearch {
    /// Select the SEER site catalog with the standard page bounds.
    pub fn seer() -> Result<Self, QueryInvalid> {
        Self::try_from(SearchFields {
            text: None,
            source: DiseaseSource::Seer,
            ontology: None,
            limit: default_limit(),
            offset: 0,
        })
    }

    /// Literal search text, absent for a native site catalog.
    pub fn text(&self) -> Option<&str> {
        self.0.text.as_deref()
    }
    /// Explicit service.
    pub fn source(&self) -> DiseaseSource {
        self.0.source
    }
    /// Selected OLS ontology, absent for other sources.
    pub fn ontology(&self) -> Option<DiseaseOntology> {
        self.0.ontology
    }
    /// Maximum source rows.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native row offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for DiseaseSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

impl DiseaseGet {
    /// Check an exact OLS concept within its source-listed ontology.
    pub fn ols(id: Text, ontology: DiseaseOntology) -> Result<Self, QueryInvalid> {
        Self::try_from(GetFields {
            id,
            source: DiseaseSource::Ols,
            ontology: Some(ontology),
            section: None,
            section_limit: None,
            section_offset: 0,
            sex: None,
            race: None,
            age_range: None,
            dataset: None,
            period: None,
            interval: None,
        })
    }

    /// Read an exact native disease CURIE through the default disease command.
    pub fn from_id(id: biodata::Curie) -> Result<Self, QueryInvalid> {
        let record = Text::new(id.as_str()).map_err(|_| QueryInvalid::new(&["id"]))?;
        Ok(Self {
            record,
            fields: GetFields {
                id: DiseaseId::Concept(id),
                source: get_source(),
                ontology: None,
                section: None,
                section_limit: Some(default_limit()),
                section_offset: 0,
                sex: None,
                race: None,
                age_range: None,
                dataset: None,
                period: None,
                interval: None,
            },
        })
    }

    /// Original edge-admitted spelling for source record attribution.
    pub fn record(&self) -> &Text {
        &self.record
    }
    /// Exact source identifier.
    pub fn id(&self) -> &DiseaseId {
        &self.fields.id
    }
    /// Explicit service.
    pub fn source(&self) -> DiseaseSource {
        self.fields.source
    }
    /// Selected OLS ontology.
    pub fn ontology(&self) -> Option<DiseaseOntology> {
        self.fields.ontology
    }
    /// Selected source section.
    pub fn section(&self) -> Option<DiseaseSection> {
        self.fields.section
    }
    /// Maximum section rows.
    pub fn section_limit(&self) -> u32 {
        self.fields.section_limit.unwrap_or_else(default_limit)
    }
    /// Section row offset.
    pub fn section_offset(&self) -> u32 {
        self.fields.section_offset
    }
    /// Continue the selected section while keeping its selectors.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut q = self.clone();
        q.fields.section_offset = offset;
        q
    }
    /// Select an available section from this exact card.
    pub fn with_section(&self, section: DiseaseSection) -> Self {
        let mut q = self.clone();
        q.fields.section = Some(section);
        q.fields.section_offset = 0;
        q
    }
    /// Apply source-resolved demographics to a checked SEER query.
    pub fn with_demographics(
        &self,
        sex: Text,
        race: Text,
        age_range: Text,
    ) -> Result<Self, QueryInvalid> {
        if self.source() != DiseaseSource::Seer {
            return Err(QueryInvalid::new(&["source"]));
        }
        let mut fields = self.fields.clone();
        fields.sex = Some(sex);
        fields.race = Some(race);
        fields.age_range = Some(age_range);
        Ok(Self {
            fields,
            record: self.record.clone(),
        })
    }
    /// Native sex selector.
    pub fn sex(&self) -> Option<&Text> {
        self.fields.sex.as_ref()
    }
    /// Native race selector.
    pub fn race(&self) -> Option<&Text> {
        self.fields.race.as_ref()
    }
    /// Native age range selector.
    pub fn age_range(&self) -> Option<&Text> {
        self.fields.age_range.as_ref()
    }
    /// Native dataset selector.
    pub fn dataset(&self) -> Option<&Text> {
        self.fields.dataset.as_ref()
    }
    /// Native period selector.
    pub fn period(&self) -> Option<&Text> {
        self.fields.period.as_ref()
    }
    /// Native interval selector.
    pub fn interval(&self) -> Option<&Text> {
        self.fields.interval.as_ref()
    }
}

impl DiseaseSection {
    /// Publishers implementing this native section.
    pub const fn sources(self) -> &'static [crate::Source] {
        use crate::Source;
        match self {
            Self::Synonyms | Self::Xrefs => &[Source::Ols, Source::MyDisease],
            Self::Hierarchy => &[Source::MyDisease],
            Self::Genes | Self::Phenotypes => &[Source::Monarch],
            Self::Incidence | Self::Survival => &[Source::Seer],
        }
    }
}
