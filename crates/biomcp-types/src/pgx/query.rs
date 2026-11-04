use super::{PgxAnnotationKind, PgxSection, PgxSource};
use crate::QueryInvalid;
use biodata::{ClinPgxAccession, Curie, GeneSymbol, PgxDiplotypeExpression, Text};
use serde::{Deserialize, Serialize};
fn default_limit() -> u32 {
    5
}
fn section_default(source: PgxSource, kind: Option<PgxAnnotationKind>) -> u32 {
    if source == PgxSource::Clinpgx
        && kind.unwrap_or(PgxAnnotationKind::Clinical) == PgxAnnotationKind::Clinical
    {
        3
    } else {
        default_limit()
    }
}
fn explicit_section_limit<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<u32>, D::Error> {
    u32::deserialize(decoder).map(Some)
}
#[cfg(feature = "schema")]
fn conditional_limit(schema: &mut schemars::Schema) {
    schema.remove("default");
}
/// Checked gene-keyed CPIC pair search. No phenotype, phase or dose is calculated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct PgxSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Explicit source gene symbol.
    gene: GeneSymbol,
    /// Required CPIC source; ClinPGx is not supported for search.
    #[cfg_attr(feature = "schema", schemars(with="str", extend("enum"=["cpic"])))]
    source: PgxSource,
    /// Literal native level, including compound levels such as B/C.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cpic_level: Option<Text>,
    /// Rows per native page.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native source offset.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(PgxSearch, SearchFields);
impl TryFrom<SearchFields> for PgxSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        if v.source != PgxSource::Cpic {
            return Err(QueryInvalid::new(&["source"]));
        }
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(v))
    }
}
impl PgxSearch {
    /// Checked gene selector.
    pub fn gene(&self) -> &GeneSymbol {
        &self.0.gene
    }
    /// Checked source selector.
    pub fn source(&self) -> PgxSource {
        self.0.source
    }
    /// Checked cpic_level selector.
    pub fn cpic_level(&self) -> Option<&Text> {
        self.0.cpic_level.as_ref()
    }
    /// Checked limit selector.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Checked offset selector.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for PgxSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

/// PGx card or section. No phenotype, phase or dose is calculated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct PgxGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Explicit source gene symbol.
    gene: GeneSymbol,
    /// Required source, checked against the section.
    source: PgxSource,
    /// One named section; omitted means the CPIC gene card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    section: Option<PgxSection>,
    /// Unchanged declared expression required for the diplotype section only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diplotype: Option<PgxDiplotypeExpression>,
    /// Exact published CPIC drug key; recommendations only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    drug_id: Option<Curie>,
    /// One ClinPGx collection; clinical is the default only on annotations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<PgxAnnotationKind>,
    /// Rows per page: three clinical, five elsewhere.
    #[serde(
        default,
        deserialize_with = "explicit_section_limit",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with="u32",range(min = 1, max = 50),transform=conditional_limit))]
    section_limit: Option<u32>,
    /// Offset within the selected section.
    #[serde(default)]
    section_offset: u32,
    /// Retained collection for local continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    snapshot: Option<Text>,
}
crate::query::validation::checked_query!(PgxGet, GetFields);
impl TryFrom<GetFields> for PgxGet {
    type Error = QueryInvalid;
    fn try_from(mut v: GetFields) -> Result<Self, Self::Error> {
        let annotations = v.section == Some(PgxSection::Annotations);
        let limit = v
            .section_limit
            .unwrap_or_else(|| section_default(v.source, v.kind));
        if v.section.map_or(v.source == PgxSource::Clinpgx, |section| {
            section.source() != v.source.source()
        }) {
            return Err(QueryInvalid::new(&["source", "section"]));
        }
        if !(1..=50).contains(&limit) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        if (v.section == Some(PgxSection::Diplotype)) != v.diplotype.is_some() {
            return Err(QueryInvalid::new(&["section", "diplotype"]));
        }
        if v.drug_id.is_some() && v.section != Some(PgxSection::Recommendations) {
            return Err(QueryInvalid::new(&["drug_id", "section"]));
        }
        if v.kind.is_some() && !annotations {
            return Err(QueryInvalid::new(&["kind", "section"]));
        }
        if v.snapshot.is_some() && !annotations {
            return Err(QueryInvalid::new(&["snapshot", "section"]));
        }
        if v.section.is_none() && v.section_offset != 0 {
            return Err(QueryInvalid::new(&["section", "section_offset"]));
        }
        if annotations && v.kind.is_none() {
            v.kind = Some(PgxAnnotationKind::Clinical);
        }
        v.section_limit = Some(limit);
        Ok(Self(v))
    }
}
impl PgxGet {
    /// Construct a checked CPIC gene card with the standard section defaults.
    pub fn new(gene: GeneSymbol) -> Result<Self, QueryInvalid> {
        Self::try_from(GetFields {
            gene,
            source: PgxSource::Cpic,
            section: None,
            diplotype: None,
            drug_id: None,
            kind: None,
            section_limit: None,
            section_offset: 0,
            snapshot: None,
        })
    }

    /// Checked gene selector.
    pub fn gene(&self) -> &GeneSymbol {
        &self.0.gene
    }
    /// Checked source selector.
    pub fn source(&self) -> PgxSource {
        self.0.source
    }
    /// Checked section selector.
    pub fn section(&self) -> Option<PgxSection> {
        self.0.section
    }
    /// Checked diplotype selector.
    pub fn diplotype(&self) -> Option<&PgxDiplotypeExpression> {
        self.0.diplotype.as_ref()
    }
    /// Checked drug_id selector.
    pub fn drug_id(&self) -> Option<&Curie> {
        self.0.drug_id.as_ref()
    }
    /// Checked kind selector.
    pub fn kind(&self) -> Option<PgxAnnotationKind> {
        self.0.kind
    }
    /// Checked section_limit selector.
    pub fn section_limit(&self) -> u32 {
        self.0
            .section_limit
            .unwrap_or_else(|| self.default_section_limit())
    }
    /// Default complete row count for the checked source and tier.
    pub fn default_section_limit(&self) -> u32 {
        section_default(self.source(), self.kind())
    }
    /// Checked section_offset selector.
    pub fn section_offset(&self) -> u32 {
        self.0.section_offset
    }
    /// Retained ClinPGx collection selector.
    pub fn snapshot(&self) -> Option<&Text> {
        self.0.snapshot.as_ref()
    }
    /// Bind later local pages to the complete received collection.
    pub fn with_snapshot(&self, snapshot: Text) -> Result<Self, QueryInvalid> {
        let mut q = self.0.clone();
        q.snapshot = Some(snapshot);
        Self::try_from(q)
    }

    /// Continue the same independent section.
    pub fn at_section_offset(&self, offset: u32) -> Result<Self, QueryInvalid> {
        let mut q = self.0.clone();
        q.section_offset = offset;
        Self::try_from(q)
    }
    /// Select a section with an explicit source and check its required context.
    pub fn with_section(
        &self,
        section: PgxSection,
        source: PgxSource,
    ) -> Result<Self, QueryInvalid> {
        let mut q = self.0.clone();
        q.section = Some(section);
        if q.source != source {
            q.section_limit = None;
        }
        q.source = source;
        q.section_offset = 0;
        Self::try_from(q)
    }
}
impl crate::Paged for PgxGet {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.section_offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.snapshot = None;
    }
}

/// Exact single clinical annotation get, with no inferred source or annotation kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AnnotationFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "AnnotationFields"))]
pub struct PgxAnnotationGet(AnnotationFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct AnnotationFields {
    /// Checked ClinPGx accession, never a shared PaId.
    id: ClinPgxAccession,
    /// Required ClinPGx source.
    source: PgxSource,
}
crate::query::validation::checked_query!(PgxAnnotationGet, AnnotationFields);
impl TryFrom<AnnotationFields> for PgxAnnotationGet {
    type Error = QueryInvalid;
    fn try_from(v: AnnotationFields) -> Result<Self, Self::Error> {
        if v.source != PgxSource::Clinpgx {
            return Err(QueryInvalid::new(&["source"]));
        }
        Ok(Self(v))
    }
}
impl PgxAnnotationGet {
    /// Select a checked native ClinPGx accession.
    pub fn new(id: ClinPgxAccession) -> Result<Self, QueryInvalid> {
        Self::try_from(AnnotationFields {
            id,
            source: PgxSource::Clinpgx,
        })
    }

    /// Exact selected accession.
    pub fn id(&self) -> &ClinPgxAccession {
        &self.0.id
    }
    /// Explicit source.
    pub fn source(&self) -> PgxSource {
        self.0.source
    }
}
