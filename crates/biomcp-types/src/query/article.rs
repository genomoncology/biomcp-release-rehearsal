use super::gene::{QueryInvalid, present};
use crate::{GeneInput, PartialDate, Text};
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    /// Source selectors supported by article search.
    ArticleSource {
        /// PubMed's own result order.
        Pubmed => "pubmed",
        /// Europe PMC native relevance or publication-date order.
        Europepmc => "europepmc",
        /// Exact variant publications in LitVar2 native order.
        Litvar => "litvar",
        /// Semantic Scholar native relevance order.
        SemanticScholar => "semantic-scholar",
        /// Sentence resemblance in LitSense2 native order.
        Litsense => "litsense",
        /// PubTator3 entity and word search in native order.
        Pubtator => "pubtator",
    }
}
// Preserve the original declaration order for source provenance keys.
impl Ord for ArticleSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}
impl PartialOrd for ArticleSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
/// Article ordering supported by this command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature="schema", schemars(with="str",extend("enum"=["rank", "date", "citations"])))]
pub enum ArticleSort {
    /// Native relevance order.
    Rank,
    /// PubMed most recently added order.
    Date,
    /// Whole-pool iCite citation count, with native-rank ties.
    Citations,
}
/// Publication-type selectors sent to the named sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature="schema",
 schemars(with="str",
extend("enum"=["trial",
 "randomized",
 "review",
 "systematic-review",
 "meta-analysis",
 "guideline",
 "case-report"])))]
pub enum ArticleType {
    /// Clinical trials, including narrower phases.
    Trial,
    /// Randomized controlled trials.
    Randomized,
    /// Reviews.
    Review,
    /// Systematic reviews.
    SystematicReview,
    /// Meta-analyses.
    MetaAnalysis,
    /// Guidelines and practice guidelines.
    Guideline,
    /// Case reports.
    CaseReport,
}
/// Checked search selectors and pool cursor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature="schema",
 schemars(with="Fields",
 extend("anyOf"=[{"required":["text"]},
{"required":["gene"]},
{"required":["variant"]},
{"required":["disease"]},
{"required":["drug"]},
{"required":["author"]},
{"required":["journal"]},
{"required":["since"]},
{"required":["until"]},
{"required":["type"]}])))]
pub struct ArticleSearch(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Require the source's native open-access filter; Semantic Scholar means a public PDF.
    #[serde(default, skip_serializing_if = "is_false")]
    open_access: bool,
    /// Exclude preprints using the source's native filter.
    #[serde(default, skip_serializing_if = "is_false")]
    no_preprints: bool,
    /// Include retracted papers by default, or exclude them using a native filter.
    #[serde(default, skip_serializing_if = "is_include")]
    #[cfg_attr(feature = "schema",
 schemars(extend("x-cli-flags" = {"include-retracted":RetractionFilter::Include,
 "exclude-retracted":RetractionFilter::Exclude})))]
    retracted: RetractionFilter,
    /// Words sent to each named source in its default fields.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature="schema", schemars(with="Text", extend("x-positional"=0)))]
    text: Option<Text>,
    /// Sources to search in caller order, comma separated. Default: pubmed.
    /// Several sources take turns, remove duplicates and show every native rank.
    /// pubmed: keywords with PubMed MeSH mapping; dropped words appear as notices.
    /// europepmc: words in open full text and preprints; incidental mentions are possible.
    /// litvar: exact variant publications; requires --variant and a protein change needs --gene; unresolved inputs return candidates.
    /// semantic-scholar: broad discovery; a shared keyless pool may return 429; an optional free key raises the limit.
    /// litsense: sentence resemblance, not a match; unrelated hits and timeouts are possible.
    /// pubtator: entity and word search including full text; needs the right entity ID; --gene uses HGNC Entrez identity and cannot run without that join.
    #[serde(default = "sources")]
    #[cfg_attr(feature = "schema", schemars(length(min = 1), extend("x-comma-separated"=true)))]
    source: Box<[ArticleSource]>,
    /// rank (default): native provider order; several sources take turns; native search cost.
    /// date: native search cost; PubMed means date added and Europe PMC means newest first; other sources refuse it.
    /// citations: most cited first across the whole pool; adds iCite calls, about 1 s per 1,000 PMIDs.
    /// All sources support citations.
    #[serde(default = "sort")]
    sort: ArticleSort,
    /// Gene input; PubMed resolves it through HGNC.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneInput"))]
    gene: Option<GeneInput>,
    /// Variant words.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    variant: Option<Text>,
    /// Disease words.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    disease: Option<Text>,
    /// Drug words.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    drug: Option<Text>,
    /// Author words sent to PubMed's author field.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    author: Option<Text>,
    /// Journal words sent to PubMed's journal field.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    journal: Option<Text>,
    /// Inclusive lower publication-date bound at the stated precision; PubMed matches either electronic or print date.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PartialDate"))]
    since: Option<PartialDate>,
    /// Inclusive upper publication-date bound at the stated precision; PubMed matches either electronic or print date.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PartialDate"))]
    until: Option<PartialDate>,
    /// Publication-type filter.
    #[serde(
        default,
        rename = "type",
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "ArticleType"))]
    kind: Option<ArticleType>,
    /// Maximum page length.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Number of pool items to skip.
    #[serde(default)]
    offset: u32,
    /// Stable snapshot from an earlier response.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
}
fn is_false(v: &bool) -> bool {
    !v
}
fn is_include(v: &RetractionFilter) -> bool {
    *v == RetractionFilter::Include
}
biodata::wire_enum! {
    /// Native retraction inclusion policy.
    #[derive(Default)]
    RetractionFilter {
        /// Preserve every source row, including retractions.
        #[default]
        Include => "include",
        /// Ask the source to omit retracted publications.
        Exclude => "exclude",
    }
}
fn sources() -> Box<[ArticleSource]> {
    Box::new([ArticleSource::Pubmed])
}
fn sort() -> ArticleSort {
    ArticleSort::Rank
}
fn limit() -> u32 {
    10
}
crate::query::validation::checked_query!(ArticleSearch, Fields);
impl TryFrom<Fields> for ArticleSearch {
    type Error = QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, QueryInvalid> {
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if v.source.is_empty() {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.source
            .iter()
            .enumerate()
            .any(|(i, source)| v.source.iter().take(i).any(|prior| prior == source))
        {
            return Err(QueryInvalid::new(&["source"]));
        }
        if [
            &v.text, &v.variant, &v.disease, &v.drug, &v.author, &v.journal,
        ]
        .iter()
        .all(|v| v.is_none())
            && v.gene.is_none()
            && v.since.is_none()
            && v.until.is_none()
            && v.kind.is_none()
        {
            return Err(QueryInvalid::new(&[
                "text", "variant", "disease", "drug", "author", "journal", "gene", "since",
                "until", "type",
            ]));
        }
        if v.since.zip(v.until).is_some_and(|(since, until)| {
            (
                since.year(),
                since.month().unwrap_or(1),
                since.day().unwrap_or(1),
            ) > (
                until.year(),
                until.month().unwrap_or(12),
                until.day().unwrap_or(31),
            )
        }) {
            return Err(QueryInvalid::new(&["since", "until"]));
        }
        Ok(Self(v))
    }
}
impl ArticleSearch {
    /// Whether native open access is required.
    pub fn open_access(&self) -> bool {
        self.0.open_access
    }
    /// Whether native preprints must be excluded.
    pub fn no_preprints(&self) -> bool {
        self.0.no_preprints
    }
    /// Whether native retractions must be excluded.
    pub fn exclude_retracted(&self) -> bool {
        self.0.retracted == RetractionFilter::Exclude
    }
    /// Sources in the caller’s turn order.
    pub fn sources(&self) -> &[ArticleSource] {
        &self.0.source
    }
    /// Native provider order requested by the caller.
    pub fn sort(&self) -> ArticleSort {
        self.0.sort
    }
    /// Author filter.
    pub fn author(&self) -> Option<&Text> {
        self.0.author.as_ref()
    }
    /// Journal filter.
    pub fn journal(&self) -> Option<&Text> {
        self.0.journal.as_ref()
    }
    /// Inclusive date bounds, preserving precision.
    pub fn dates(&self) -> (Option<PartialDate>, Option<PartialDate>) {
        (self.0.since, self.0.until)
    }
    /// Requested publication type.
    pub fn kind(&self) -> Option<ArticleType> {
        self.0.kind
    }

    /// Free text followed by variant, drug and disease selectors in the approved term order.
    pub fn words(&self) -> String {
        [&self.0.text, &self.0.variant, &self.0.drug, &self.0.disease]
            .into_iter()
            .flatten()
            .map(Text::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
    /// Optional gene identity selector.
    pub fn gene(&self) -> Option<&GeneInput> {
        self.0.gene.as_ref()
    }
    /// Replace only the checked gene selector.
    pub fn with_gene(&self, gene: GeneInput) -> Self {
        let mut v = self.clone();
        v.0.gene = Some(gene);
        v
    }
    /// Page length.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Optional explicit snapshot.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
    /// Select a page without changing selectors.
    pub fn page(&self, offset: u32, pool: Option<Text>) -> Self {
        let mut v = self.clone();
        v.0.offset = offset;
        v.0.pool = pool;
        v
    }
    /// Change ordering and restart the request without an old pool.
    pub fn ordered(&self, sort: ArticleSort) -> Self {
        let mut v = self.page(0, None);
        v.0.sort = sort;
        v
    }
    /// Canonical identity excludes cursor and page size.
    pub fn identity(&self) -> Self {
        let mut v = self.page(0, None);
        v.0.limit = limit();
        v
    }
}

impl ArticleSearch {
    /// Start a PubMed search for one source-stated gene.
    pub fn for_gene(gene: GeneInput) -> Self {
        Self(Fields {
            gene: Some(gene),
            text: None,
            variant: None,
            disease: None,
            drug: None,
            author: None,
            journal: None,
            since: None,
            until: None,
            kind: None,
            source: sources(),
            sort: sort(),
            limit: limit(),
            offset: 0,
            pool: None,
            open_access: false,
            no_preprints: false,
            retracted: RetractionFilter::default(),
        })
    }
    /// Broaden an exact gene and variant search without changing its other filters.
    pub fn without_variant(&self) -> Option<Self> {
        self.gene()?;
        self.variant()?;
        let mut value = self.with_source(ArticleSource::Pubmed);
        value.0.variant = None;
        Some(value)
    }
    /// Restart once per supplied word group, dropping only that group in fixed order.
    /// Gene identity and every other selector remain unchanged.
    pub fn term_drops(&self) -> Vec<Self> {
        let present = [&self.0.text, &self.0.variant, &self.0.disease, &self.0.drug];
        if present.iter().filter(|term| term.is_some()).count() < 2 {
            return Vec::new();
        }
        present
            .iter()
            .enumerate()
            .filter_map(|(index, term)| {
                term.as_ref()?;
                let mut next = self.page(0, None);
                match index {
                    0 => next.0.text = None,
                    1 => next.0.variant = None,
                    2 => next.0.disease = None,
                    _ => next.0.drug = None,
                }
                Some(next)
            })
            .collect()
    }
    /// Exact variant selector, kept distinct from ordinary word search.
    pub fn variant(&self) -> Option<&Text> {
        self.0.variant.as_ref()
    }
    /// Offer PubMed title/abstract terms without relying on phrase interpretation.
    pub fn strict_pubmed(&self) -> Option<Self> {
        let mut words = self.words();
        if let Some(gene) = self.gene() {
            words = format!("{gene} {words}");
        }
        if words.contains('[') || words.contains(']') {
            return None;
        }
        let terms: Vec<_> = words
            .split_whitespace()
            .map(|word| word.trim_matches('"'))
            .filter(|word| !word.is_empty())
            .map(|word| format!("{word}[Title/Abstract]"))
            .collect();
        if terms.is_empty() {
            return None;
        }
        let mut value = self.page(0, None);
        value.0.source = Box::new([ArticleSource::Pubmed]);
        value.0.text = Some(Text::new(&terms.join(" AND ")).ok()?);
        value.0.gene = None;
        value.0.variant = None;
        value.0.drug = None;
        value.0.disease = None;
        Some(value)
    }
    /// Restart with an exact candidate rsID, retaining source and order choices.
    pub fn with_variant(&self, variant: Text) -> Self {
        let mut value = self.page(0, None);
        value.0.variant = Some(variant);
        value
    }
    /// Restart with one source while retaining the request selectors.
    pub fn with_source(&self, source: ArticleSource) -> Self {
        let mut value = self.page(0, None);
        value.0.source = Box::new([source]);
        value
    }
    /// Restart with the approved PubMed and LitVar discovery sources.
    pub fn with_litvar(&self) -> Self {
        let mut value = self.page(0, None);
        value.0.source = Box::new([ArticleSource::Pubmed, ArticleSource::Litvar]);
        value
    }
}

biodata::wire_enum! {
    /// Source capability selectors checked before any lookup or transport request.
    ArticleFilter {
        /// Free text.
        Text => "TEXT",
        /// Gene selector.
        Gene => "--gene",
        /// Variant selector.
        Variant => "--variant",
        /// Disease selector.
        Disease => "--disease",
        /// Drug selector.
        Drug => "--drug",
        /// Author selector.
        Author => "--author",
        /// Journal selector.
        Journal => "--journal",
        /// Lower publication date bound.
        Since => "--since",
        /// Upper publication date bound.
        Until => "--until",
        /// Publication type selector.
        Type => "--type",
        /// Native date ordering.
        DateSort => "--sort date",
        /// Native open-access selector.
        OpenAccess => "--open-access",
        /// Native preprint exclusion.
        NoPreprints => "--no-preprints",
        /// Native retraction exclusion.
        ExcludeRetracted => "--exclude-retracted",
    }
}
impl ArticleSearch {
    /// Whether any named unsupported capability is requested.
    pub fn rejects(&self, filters: &[ArticleFilter]) -> Option<ArticleFilter> {
        filters.iter().copied().find(|filter| match filter {
            ArticleFilter::OpenAccess => self.open_access(),
            ArticleFilter::NoPreprints => self.no_preprints(),
            ArticleFilter::ExcludeRetracted => self.exclude_retracted(),
            ArticleFilter::Text => self.0.text.is_some(),
            ArticleFilter::Gene => self.0.gene.is_some(),
            ArticleFilter::Variant => self.0.variant.is_some(),
            ArticleFilter::Disease => self.0.disease.is_some(),
            ArticleFilter::Drug => self.0.drug.is_some(),
            ArticleFilter::Author => self.0.author.is_some(),
            ArticleFilter::Journal => self.0.journal.is_some(),
            ArticleFilter::Since => self.0.since.is_some(),
            ArticleFilter::Until => self.0.until.is_some(),
            ArticleFilter::Type => self.0.kind.is_some(),
            ArticleFilter::DateSort => self.0.sort == ArticleSort::Date,
        })
    }
}
