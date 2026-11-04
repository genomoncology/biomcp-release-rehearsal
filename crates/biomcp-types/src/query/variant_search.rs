use super::ProteinQuery;
use super::gene::{QueryInvalid, present};
use crate::{Build, Chromosome, GeneSymbol, Text, text_value};
use biodata::CodingChange;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
const MAX_FILTER_CHARS: usize = 256;
biodata::wire_enum! {
    /// The source-native query language exposed by variant search.
    VariantSearchSource {
        /// MyVariant.info candidate discovery.
        Myvariant => "myvariant",
    }
}
biodata::wire_enum! {
    /// Source-proven component fields supported by the presence filter.
    VariantSearchField {
        /// Presence of MyVariant's ClinVar variant identifier, without interpreting its assertions.
        Clinvar => "clinvar",
    }
}
/// MyVariant's one-based inclusive primary-chromosome region spelling, kept unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantRegion(Box<str>);
impl FromStr for VariantRegion {
    type Err = QueryInvalid;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (chromosome, bounds) = value
            .split_once(':')
            .ok_or_else(|| QueryInvalid::new(&["region"]))?;
        chromosome
            .strip_prefix("chr")
            .ok_or_else(|| QueryInvalid::new(&["region"]))?
            .parse::<Chromosome>()
            .map_err(|_| QueryInvalid::new(&["region"]))?;
        let (start, end) = bounds
            .split_once('-')
            .ok_or_else(|| QueryInvalid::new(&["region"]))?;
        let position = |v: &str| {
            if v.starts_with('0') {
                return Err(QueryInvalid::new(&["region"]));
            }
            if !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err(QueryInvalid::new(&["region"]));
            }
            v.parse::<u32>().map_err(|_| QueryInvalid::new(&["region"]))
        };
        if position(start)? > position(end)? {
            return Err(QueryInvalid::new(&["region"]));
        }
        Ok(Self(value.into()))
    }
}
impl fmt::Display for VariantRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
text_value!(
    VariantRegion,
    "VariantRegion",
    format!(
        "^chr([1-9]|1[0-9]|2[0-2]|X|Y|MT):{}-{}$",
        biodata::schema_pattern::decimal(u32::MAX.into()),
        biodata::schema_pattern::decimal(u32::MAX.into())
    ),
    "Primary-chromosome region. The parser additionally requires the end to be at least the start."
);

/// Explicit MyVariant query text, preserving surrounding whitespace and source syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MyVariantQuery(Box<str>);
impl FromStr for MyVariantQuery {
    type Err = QueryInvalid;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let maximum = crate::VariantInput::MAX_BYTES * 2 + "_id:\"\"".len();
        if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
            return Err(QueryInvalid::new(&["query"]));
        }
        Ok(Self(value.into()))
    }
}
impl fmt::Display for MyVariantQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
text_value!(
    MyVariantQuery,
    "MyVariantQuery",
    {
        let maximum = crate::VariantInput::MAX_BYTES * 2 + "_id:\"\"".len();
        let whitespace = biodata::schema_pattern::WHITESPACE;
        format!(r"^(?![{whitespace}]*$)[^\x00-\x1f\x7f-\x9f]{{1,{maximum}}}$")
    },
    "Explicit MyVariant query text. The schema limits Unicode character count; the parser additionally limits UTF-8 byte count to the same maximum."
);
/// Discover source candidates using one declared selector, without interpreting free text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields", extend("anyOf" = [
    {"required":["gene","protein"]},
    {"required":["gene","coding"]},
    {"required":["region","build"]},
    {"required":["query","source"]}
])))]
pub struct VariantSearch(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Checked gene symbol for the dbNSFP substitution or ClinVar coding/synonymous selector.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneSymbol"))]
    gene: Option<GeneSymbol>,
    /// Explicit substitution or synonymous change using checked amino-acid codes.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "ProteinQuery"))]
    protein: Option<ProteinQuery>,
    /// Coding HGVS change searched literally in ClinVar with --gene, without choosing a transcript.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "CodingChange"))]
    coding: Option<CodingChange>,
    /// Exact one-based inclusive MyVariant region, such as chr7:55259500-55259600; requires --build.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "VariantRegion"))]
    region: Option<VariantRegion>,
    #[doc = " Raw MyVariant query, passed and echoed unchanged when component policy allows; requires \
        --source myvariant. With disabled components use declared selectors instead."]
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "MyVariantQuery"))]
    query: Option<MyVariantQuery>,
    /// Optional caller assembly sent to MyVariant; regions require it explicitly.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Build"))]
    build: Option<Build>,
    /// Explicit source language; required for a raw query.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "VariantSearchSource"))]
    source: Option<VariantSearchSource>,
    /// Literal MyVariant clinvar.rcv.review_status phrase; no star conversion. Requires a declared protein or region selector.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", length(max = MAX_FILTER_CHARS)))]
    review_status: Option<Text>,
    /// Literal MyVariant clinvar.rcv.conditions.name phrase; source text is not a local diagnosis. Requires a declared protein or region selector.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", length(max = MAX_FILTER_CHARS)))]
    condition: Option<Text>,
    /// Require the selected component's identifier field. Only source-proven mappings are offered.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "VariantSearchField"))]
    has: Option<VariantSearchField>,
    /// Native candidates per page, default 10, maximum 50; all candidates remain candidates.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native offset within this command's bounded result window; larger native totals remain visible.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(range(max = VariantSearch::MAX_RESULTS - 1)))]
    offset: u32,
}
fn default_limit() -> u32 {
    10
}
crate::query::validation::checked_query!(VariantSearch, Fields);
impl TryFrom<Fields> for VariantSearch {
    type Error = QueryInvalid;
    fn try_from(f: Fields) -> Result<Self, Self::Error> {
        let protein = f.gene.is_some()
            && f.protein.is_some()
            && f.coding.is_none()
            && f.region.is_none()
            && f.query.is_none()
            && (!f.protein.as_ref().is_some_and(ProteinQuery::is_synonymous)
                || (f.build.is_none() && f.source.is_none() && !f.filtered()));
        let coding = f.gene.is_some()
            && f.coding.is_some()
            && f.protein.is_none()
            && f.region.is_none()
            && f.query.is_none()
            && f.build.is_none()
            && f.source.is_none()
            && !f.filtered();
        let region = f.region.is_some()
            && f.build.is_some()
            && f.gene.is_none()
            && f.protein.is_none()
            && f.coding.is_none()
            && f.query.is_none();
        let raw = f.query.is_some()
            && f.source.is_some()
            && f.gene.is_none()
            && f.protein.is_none()
            && f.coding.is_none()
            && f.region.is_none();
        if !(protein || coding || region || raw) {
            return Err(QueryInvalid::new(&[
                "gene", "protein", "coding", "region", "build", "query", "source",
            ]));
        }
        if !(1..=50).contains(&f.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if f.offset >= Self::MAX_RESULTS {
            return Err(QueryInvalid::new(&["offset"]));
        }
        if raw && f.filtered() {
            return Err(QueryInvalid::new(&[
                "query",
                "review_status",
                "condition",
                "has",
            ]));
        }
        if [&f.review_status, &f.condition]
            .into_iter()
            .flatten()
            .any(|v| {
                v.as_str().chars().count() > MAX_FILTER_CHARS
                    || v.as_str().chars().any(char::is_control)
            })
        {
            return Err(QueryInvalid::new(&["review_status", "condition"]));
        }
        Ok(Self(f))
    }
}
impl VariantSearch {
    /// Inspect one unchanged source record without assigning an assembly.
    /// https://docs.myvariant.info/en/latest/doc/variant_query_service.html#fielded-queries
    pub fn source_record(id: &crate::VariantInput) -> Result<Self, QueryInvalid> {
        Self::try_from(Fields {
            gene: None,
            protein: None,
            coding: None,
            region: None,
            build: None,
            query: Some(format!("_id:\"{}\"", literal(id.as_str())).parse()?),
            source: Some(VariantSearchSource::Myvariant),
            review_status: None,
            condition: None,
            has: None,
            limit: default_limit(),
            offset: 0,
        })
    }
    /// Maximum native result window this command retrieves; larger totals are disclosed.
    pub const MAX_RESULTS: u32 = 10_000;
    /// Whether this search requires the independently controlled ClinVar component.
    pub fn uses_clinvar_filters(&self) -> bool {
        self.0.filtered()
    }
    /// Whether the caller selected opaque source syntax rather than declared selectors.
    pub fn raw_query(&self) -> bool {
        self.0.query.is_some()
    }
    /// Whether the declared selector searches dbNSFP's gene and protein fields.
    pub fn uses_dbnsfp_selector(&self) -> bool {
        self.0.protein.as_ref().is_some_and(|p| !p.is_synonymous())
    }
    /// Whether the caller selected native ClinVar coding or synonymous notation.
    pub fn uses_clinvar_selector(&self) -> bool {
        self.0.coding.is_some()
            || self
                .0
                .protein
                .as_ref()
                .is_some_and(ProteinQuery::is_synonymous)
    }
    /// Compare a source protein spelling with the caller's checked substitution spellings.
    pub fn matches_protein(&self, gene: Option<&str>, notation: &str) -> bool {
        self.0
            .gene
            .as_ref()
            .is_some_and(|g| Some(g.as_str()) == gene)
            && self
                .0
                .protein
                .as_ref()
                .is_some_and(|p| p.forms().iter().any(|form| form == notation))
    }
    /// Exact declared source query; no alternative notation or free-text guessing.
    pub fn source_query(&self) -> String {
        if let Some(q) = &self.0.query {
            return q.to_string();
        }
        if let (Some(gene), Some(coding)) = (&self.0.gene, &self.0.coding) {
            return format!("{gene}[GENE] AND \"{coding}\"[VRNM]");
        }
        if let (Some(gene), Some(protein)) = (&self.0.gene, &self.0.protein)
            && protein.is_synonymous()
        {
            return format!("{gene}[GENE] AND \"{protein}\"[VRNM]");
        }
        let mut term = if let Some(region) = &self.0.region {
            region.to_string()
        } else {
            match (&self.0.gene, &self.0.protein) {
                (Some(g), Some(p)) => format!(
                    "dbnsfp.genename:{} AND dbnsfp.hgvsp:\"p.{}\"",
                    literal(g.as_str()),
                    literal(p.as_str().strip_prefix("p.").unwrap_or(p.as_str()))
                ),
                _ => String::new(),
            }
        };
        self.0.append_filters(&mut term);
        term
    }
    /// Open SnpEff selector using the same literal protein forms as transcript matching.
    /// Other selectors retain their ordinary source query.
    pub fn snpeff_source_query(&self) -> String {
        if !self.uses_dbnsfp_selector() {
            return self.source_query();
        }
        let mut term = match (&self.0.gene, &self.0.protein) {
            (Some(gene), Some(protein)) => {
                let forms = protein
                    .forms()
                    .map(|p| format!("\"{}\"", literal(&p)))
                    .join(" OR ");
                format!(
                    "snpeff.ann.genename:{} AND snpeff.ann.hgvs_p:({forms})",
                    literal(gene.as_str())
                )
            }
            _ => String::new(),
        };
        self.0.append_filters(&mut term);
        term
    }
    /// Assembly explicitly declared in the request.
    pub fn build(&self) -> Option<Build> {
        self.0.build
    }
    /// Native page size, clipped at this command's result-window boundary.
    pub fn limit(&self) -> u32 {
        self.0.limit.min(Self::MAX_RESULTS - self.0.offset)
    }
    /// Native page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for VariantSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

impl Fields {
    fn filtered(&self) -> bool {
        self.review_status.is_some() || self.condition.is_some() || self.has.is_some()
    }
    fn append_filters(&self, term: &mut String) {
        for (field, value) in [
            ("review_status", &self.review_status),
            ("conditions.name", &self.condition),
        ] {
            if let Some(value) = value {
                term.push_str(&format!(
                    " AND clinvar.rcv.{field}:\"{}\"",
                    literal(value.as_str())
                ));
            }
        }
        if let Some(VariantSearchField::Clinvar) = self.has {
            term.push_str(" AND _exists_:clinvar.variant_id");
        }
    }
}

// MyVariant query documentation, "Escaping reserved characters":
// https://docs.myvariant.info/en/latest/doc/variant_query_service.html#escaping-reserved-characters
// This changes source syntax only; it never rewrites the declared symbol or substitution.
fn literal(value: &str) -> String {
    let mut escaped = String::new();
    for c in value.chars() {
        if "+-=|&><!(){}[]^\"~*?:\\/".contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}
