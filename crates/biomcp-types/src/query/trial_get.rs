use super::gene::{QueryInvalid, present};
use crate::{NctId, Text, TrialSource};
use serde::{Deserialize, Serialize};
/// Retrieve exact trial cards in input order; request bounded intact source sections with independent continuations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct TrialGet(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// One to twenty exact NCT identifiers, in caller order.
    #[cfg_attr(feature="schema", schemars(length(min=1,max=20), extend("x-positional"=0)))]
    input: Box<[NctId]>,
    #[doc = " Optional intact source sections. Default cards only; diseases and summary require NCI. \
        NCI locations, contacts, references and documents are unsupported."]
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    #[cfg_attr(feature="schema", schemars(extend("x-trailing-section"=true)))]
    section: Box<[TrialSection]>,
    /// Whole-record count per source collection page; text fragments follow the text budget.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Character offset for ClinicalTrials.gov eligibility or NCI summary; whole-record offset for other collections.
    #[serde(default)]
    offset: u32,
    /// Checked stored source record, supplied by a continuation.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
    /// Explicit provider; defaults to ClinicalTrials.gov and never falls back.
    #[serde(default, skip_serializing_if = "TrialSource::is_default")]
    source: TrialSource,
}
fn default_limit() -> u32 {
    10
}
crate::query::validation::checked_query!(TrialGet, Fields);
impl TryFrom<Fields> for TrialGet {
    type Error = QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, Self::Error> {
        if v.source == TrialSource::Ctgov
            && v.section
                .iter()
                .any(|s| matches!(s, TrialSection::Diseases | TrialSection::Summary))
        {
            return Err(QueryInvalid::new(&["source", "section"]));
        }
        if !(1..=20).contains(&v.input.len()) {
            return Err(QueryInvalid::new(&["input"]));
        }
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if (v.pool.is_some() || v.offset != 0)
            && (v.input.len() != 1
                || v.section.len() != 1
                || v.section.first() == Some(&TrialSection::All))
        {
            return Err(QueryInvalid::new(&["pool", "offset", "input", "section"]));
        }
        if v.offset != 0 && v.pool.is_none() {
            return Err(QueryInvalid::new(&["offset", "pool"]));
        }
        Ok(Self(v))
    }
}
impl TrialGet {
    /// Explicit source selected by the caller.
    pub fn source(&self) -> TrialSource {
        self.0.source
    }

    /// Select one independent provider for a generated section action.
    pub fn with_source(mut self, source: TrialSource) -> Self {
        self.0.source = source;
        self
    }
    /// Exact checked selectors.
    pub fn input(&self) -> &[NctId] {
        &self.0.input
    }
    /// Requested section words.
    pub fn sections(&self) -> &[TrialSection] {
        &self.0.section
    }
    /// Whole-record page limit.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Source character or record position.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Stored source record capability.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
    /// One card or section request, used by typed next steps.
    pub fn single(input: NctId, section: Option<TrialSection>, limit: u32) -> Self {
        Self(Fields {
            input: Box::new([input]),
            section: section.into_iter().collect(),
            limit,
            offset: 0,
            pool: None,
            source: TrialSource::Ctgov,
        })
    }
    /// Continue one immutable source record at a checked position.
    pub fn page(&self, input: NctId, section: TrialSection, pool: Text, offset: u32) -> Self {
        let mut next = Self::single(input, Some(section), self.limit());
        next.0.source = self.source();
        next.0.pool = Some(pool);
        next.0.offset = offset;
        next
    }
    /// Initial request without continuation state.
    pub fn identity(&self) -> Self {
        let mut q = self.clone();
        q.0.pool = None;
        q.0.offset = 0;
        q
    }
}
biodata::wire_enum! {
    described
    /// Available source protocol sections.
    TrialSection {
        /// Intact criteria and structured age/sex fields.
        Eligibility => "eligibility",
        /// Primary, secondary and other outcome records.
        Outcomes => "outcomes",
        /// Posted outcome values, analyses and adverse-event tables (ClinicalTrials.gov).
        Results => "results",
        /// Arms and interventions with original relationships.
        Arms => "arms",
        /// Whole source sites with their attached contacts.
        Locations => "locations",
        /// Central contacts and overall officials in their original roles.
        Contacts => "contacts",
        /// Registry publication identities and native reference types.
        References => "references",
        /// Posted document metadata and source URLs.
        Documents => "documents",
        /// Every implemented trial section, each with independent continuations.
        All => "all",
        /// Native NCI disease concepts, including source enrichment metadata.
        Diseases => "diseases",
        /// Complete NCI summary, paged by original character offsets.
        Summary => "summary",
    }
}
