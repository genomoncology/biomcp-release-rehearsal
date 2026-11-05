mod biomarker;
mod condition;
mod continuation;
mod country;
mod filters;
use crate::{GeneInput, NctId, PartialDate, StudyType, Text, TrialSex, TrialStatus};
pub use biomarker::NciBiomarkerUse;
pub use filters::{NciStatusFilter, TrialPhaseFilter, TrialStatusFilter};
use serde::{Deserialize, Serialize};
const MAX_ALTERNATIVES: usize = 5;
biodata::wire_enum! {
    /// Registry funding category facet.
    SponsorType {
        /// National Institutes of Health.
        Nih => "nih",
        /// Other United States federal agency.
        Fed => "fed",
        /// Industry.
        Industry => "industry",
        /// Other sponsor category.
        Other => "other",
    }
}
/// Search ClinicalTrials.gov or keyed NCI with native counts. NCI --facility matches exact organization names; a listed site is not necessarily enrolling.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct TrialSearch(Box<Fields>);
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    #[doc = "clinicaltrials/nci.\nExplicit provider; defaults to ClinicalTrials.gov and never falls back."]
    #[serde(default, skip_serializing_if = "TrialSource::is_default")]
    source: TrialSource,
    #[doc = "Text; OR.\n\
        Repeat for ClinicalTrials.gov general text alternatives joined with OR; different filters combine with AND. NCI accepts one keyword."]
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(length(max = MAX_ALTERNATIVES)))]
    keyword: Box<[Text]>,
    #[doc = "Trial text.\nGeneral registry discovery text."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", extend("x-positional" = 0)))]
    query: Option<Text>,
    #[doc = "Disease or NCIt.\nCondition alternatives joined with OR; NCI requires NCIt codes."]
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(length(max = MAX_ALTERNATIVES)))]
    condition: Box<[Text]>,
    #[doc = "Exact condition.\nAsk ClinicalTrials.gov to restrict matching to the registered condition phrase; its support is partial."]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    exact_condition: bool,
    #[doc = "MONDO/DOID/NCIT:C.\nExact MONDO or DOID CURIE resolved by MyDisease; NCI also accepts explicit NCIT:C codes."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "biodata::Curie"))]
    condition_id: Option<biodata::Curie>,
    #[doc = "Treatment terms; OR.\nIntervention alternatives joined with OR; supported by ClinicalTrials.gov."]
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(length(max = MAX_ALTERNATIVES)))]
    intervention: Box<[Text]>,
    #[doc = "Gene symbol/ID.\nResolve one checked gene for registry text discovery only; no eligibility judgment."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneInput"))]
    gene: Option<GeneInput>,
    #[doc = "Variant text.\nLiteral variant text for registry discovery only; no eligibility judgment."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    variant: Option<Text>,
    #[doc = "NCI NCIt code.\nExact NCIt biomarker concept listed by NCI in trial criteria; NCI source only. \
        This listing does not establish biomarker positivity or patient eligibility."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "biodata::NcitId"))]
    biomarker_code: Option<biodata::NcitId>,
    #[doc = "NCI criterion.\nFilter by NCI's native inclusion or exclusion criterion. Check each result's biomarker entry and reported problems."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "NciBiomarkerUse"))]
    biomarker_use: Option<NciBiomarkerUse>,
    #[doc = "Criteria text.\nSearch literal registry eligibility text for discovery only; no eligibility judgment."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    criteria: Option<Text>,
    #[doc = "Native status.\nExact source status. NCI: Active, Administratively Complete, Approved, Closed to Accrual, \
        Closed to Accrual and Intervention, Complete, INACTIVE, Enrolling by Invitation, \
        Temporarily Closed to Accrual, Temporarily Closed to Accrual and Intervention, Withdrawn. \
        ClinicalTrials.gov uses registry labels."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "TrialStatusFilter"))]
    status: Option<TrialStatusFilter>,
    #[doc = "Native phase.\n\
        Exact source phase. ClinicalTrials.gov: 1, 2, 3, 4, early-1, not-applicable, 1/2, 2/3. NCI: NA, 0, I, I_II, II, II_III, III, IV."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "TrialPhaseFilter"))]
    phase: Option<TrialPhaseFilter>,
    #[doc = "Study type.\nExact registry study type."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "StudyType"))]
    study_type: Option<StudyType>,
    /// Registry sex facet.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "TrialSex"))]
    sex: Option<TrialSex>,
    /// Age in years; unstated source bounds count as no limit.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32"))]
    age: Option<u32>,
    #[doc = "Facility.\nFacility phrase; ClinicalTrials.gov broadens after zero exact matches. NCI matches exact names. Cannot combine with radius."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    facility: Option<Text>,
    #[doc = "Country label; biomcp_help search trial.\n\
        Exact ClinicalTrials.gov country label; repeat for OR. Trials must list a site in a named country. NCI uses its native country filter."]
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(schema_with = "country::schema"))]
    country: Box<[Box<str>]>,
    #[doc = "Latitude degrees.\nRadius center latitude in degrees."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    lat: Option<Text>,
    #[doc = "Longitude degrees.\nRadius center longitude in degrees."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    lon: Option<Text>,
    #[doc = "Radius, mi or km.\nSource distance with explicit mi or km units."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    distance: Option<Text>,
    #[doc = "Sponsor.\nSponsor name; ClinicalTrials.gov broadens after zero exact matches. Use --sponsor-type for the funding category."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    sponsor: Option<Text>,
    #[doc = "Funding class.\nRegistry sponsor funding category; use --sponsor for a sponsor name."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "SponsorType"))]
    sponsor_type: Option<SponsorType>,
    #[doc = "First date.\nFirst included registry update date."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PartialDate"))]
    updated_since: Option<PartialDate>,
    #[doc = "Last date.\nLast included registry update date."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "PartialDate"))]
    updated_until: Option<PartialDate>,
    #[doc = "NCT identifier.\nExact NCT identity in a conjunction with other filters."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "NctId"))]
    id: Option<NctId>,
    #[doc = "Page token.\nOpaque registry continuation; keep filters unchanged."]
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    next_page: Option<Text>,
    /// Zero-based offset of the first row in this continuation page.
    #[serde(default)]
    offset: u32,
    #[doc = "Total.\nSource total only; no returned study rows."]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    count_only: bool,
    #[doc = "Posted results.\nRequire posted source results."]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    has_results: bool,
    /// Headers per page; NCI applies its own lower publisher limit.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 100)))]
    limit: u32,
}
fn default_limit() -> u32 {
    10
}
crate::query::validation::checked_query!(TrialSearch, Fields);
impl TrialSearch {
    /// Search trials that list the exact NCI biomarker concept.
    pub fn for_nci_biomarker(code: biodata::NcitId) -> Self {
        Self(Box::new(Fields {
            source: TrialSource::Nci,
            biomarker_code: Some(code),
            limit: default_limit(),
            ..Fields::default()
        }))
    }
    /// Exact source concept, without descendant expansion.
    pub fn biomarker_code(&self) -> Option<&biodata::NcitId> {
        self.0.biomarker_code.as_ref()
    }
    /// Native source criterion requested by the caller.
    pub fn biomarker_use(&self) -> Option<NciBiomarkerUse> {
        self.0.biomarker_use
    }
    /// Search NCI trials at the exact published organization name.
    pub fn for_nci_facility(name: Text) -> Self {
        Self(Box::new(Fields {
            source: TrialSource::Nci,
            facility: Some(name),
            limit: default_limit(),
            ..Fields::default()
        }))
    }
    /// Explicit source selected by the caller.
    pub fn source(&self) -> TrialSource {
        self.0.source
    }
    /// General text alternatives; NCI accepts one explicit keyword.
    pub fn keyword(&self) -> &[Text] {
        &self.0.keyword
    }

    /// General registry discovery text.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Condition alternatives joined with OR; NCI requires NCIt codes.
    pub fn condition(&self) -> &[Text] {
        &self.0.condition
    }
    /// Exact caller-selected concept, without a source-name search.
    pub fn condition_id(&self) -> Option<&biodata::Curie> {
        self.0.condition_id.as_ref()
    }
    /// Intervention alternatives joined with OR; supported by ClinicalTrials.gov.
    pub fn intervention(&self) -> &[Text] {
        &self.0.intervention
    }
    /// Resolve one checked gene for registry text discovery only; no eligibility judgment.
    pub fn gene(&self) -> Option<&GeneInput> {
        self.0.gene.as_ref()
    }
    /// Literal variant text for registry discovery only; no eligibility judgment.
    pub fn variant(&self) -> Option<&Text> {
        self.0.variant.as_ref()
    }
    /// Search literal registry eligibility text for discovery only; no eligibility judgment.
    pub fn criteria(&self) -> Option<&Text> {
        self.0.criteria.as_ref()
    }
    /// Exact ClinicalTrials.gov status; NCI filters use nci_status().
    pub fn status(&self) -> Option<&TrialStatus> {
        match self.0.status.as_ref()? {
            TrialStatusFilter::Registry(value) => Some(value),
            TrialStatusFilter::Nci(_) => None,
        }
    }
    /// Exact ClinicalTrials.gov phase; NCI filters use nci_phase().
    pub fn phase(&self) -> Option<&PhaseFilter> {
        match self.0.phase.as_ref()? {
            TrialPhaseFilter::Registry(value) => Some(value),
            TrialPhaseFilter::Nci(_) => None,
        }
    }
    /// Exact NCI status selected by the caller.
    pub fn nci_status(&self) -> Option<NciStatusFilter> {
        match self.0.status.as_ref()? {
            TrialStatusFilter::Nci(value) => Some(*value),
            TrialStatusFilter::Registry(_) => None,
        }
    }
    /// Exact NCI phase selected by the caller.
    pub fn nci_phase(&self) -> Option<crate::NciPhaseFilter> {
        match self.0.phase.as_ref()? {
            TrialPhaseFilter::Nci(value) => Some(*value),
            TrialPhaseFilter::Registry(_) => None,
        }
    }
    /// Exact registry study type.
    pub fn study_type(&self) -> Option<&StudyType> {
        self.0.study_type.as_ref()
    }
    /// Registry sex facet.
    pub fn sex(&self) -> Option<&TrialSex> {
        self.0.sex.as_ref()
    }
    /// Age in years; unstated source bounds count as no limit.
    pub fn age(&self) -> Option<&u32> {
        self.0.age.as_ref()
    }
    /// Requested facility phrase; a listed site is not necessarily enrolling.
    pub fn facility(&self) -> Option<&Text> {
        self.0.facility.as_ref()
    }
    /// Exact published country alternatives, without whitespace repair.
    pub fn country(&self) -> &[Box<str>] {
        &self.0.country
    }
    /// Radius center latitude in degrees.
    pub fn lat(&self) -> Option<&Text> {
        self.0.lat.as_ref()
    }
    /// Radius center longitude in degrees.
    pub fn lon(&self) -> Option<&Text> {
        self.0.lon.as_ref()
    }
    /// Source distance with explicit mi or km units.
    pub fn distance(&self) -> Option<&Text> {
        self.0.distance.as_ref()
    }
    /// Requested sponsor name.
    pub fn sponsor(&self) -> Option<&Text> {
        self.0.sponsor.as_ref()
    }
    /// Requested sponsor funding category.
    pub fn sponsor_type(&self) -> Option<&SponsorType> {
        self.0.sponsor_type.as_ref()
    }
    /// First included registry update date.
    pub fn updated_since(&self) -> Option<&PartialDate> {
        self.0.updated_since.as_ref()
    }
    /// Last included registry update date.
    pub fn updated_until(&self) -> Option<&PartialDate> {
        self.0.updated_until.as_ref()
    }
    /// Exact NCT identity in a conjunction with other filters.
    pub fn id(&self) -> Option<&NctId> {
        self.0.id.as_ref()
    }
    /// Opaque registry continuation; keep filters unchanged.
    pub fn next_page(&self) -> Option<&Text> {
        self.0.next_page.as_ref()
    }
    /// Zero-based offset carried by continuation queries.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Source count without study rows.
    pub fn count_only(&self) -> bool {
        self.0.count_only
    }
    /// Posted results restriction.
    pub fn has_results(&self) -> bool {
        self.0.has_results
    }
    /// Bounded page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Explicit alternatives in order: remove variant, then move the resolved gene
    /// to eligibility text. Existing eligibility text and continuation remain intact.
    pub fn wider_searches(&self, symbol: Option<Text>) -> Vec<Self> {
        if self.source() != TrialSource::Ctgov || self.next_page().is_some() {
            return Vec::new();
        }
        let mut alternatives = Vec::new();
        if self.variant().is_some() {
            let mut next = self.clone();
            next.0.variant = None;
            alternatives.push(next);
        }
        if self.gene().is_some()
            && self.criteria().is_none()
            && let Some(symbol) = symbol
        {
            let mut next = self.clone();
            next.0.gene = None;
            next.0.criteria = Some(symbol);
            alternatives.push(next);
        }
        alternatives
    }
    /// Continue the identical source query with its opaque cursor.
    pub fn at_page(&self, token: Text, offset: u32) -> Self {
        let mut next = self.clone();
        next.0.next_page = Some(token);
        next.0.offset = offset;
        next
    }
}

biodata::wire_enum! {
    /// Explicit source phase selection, including the registry's combined phases.
    PhaseFilter {
        /// Early phase 1.
        Early1 => "early-1",
        /// Phase 1.
        One => "1",
        /// Phase 2.
        Two => "2",
        /// Phase 3.
        Three => "3",
        /// Phase 4.
        Four => "4",
        /// Not applicable.
        NotApplicable => "not-applicable",
        /// Combined phase 1 and phase 2.
        OneTwo => "1/2",
        /// Combined phase 2 and phase 3.
        TwoThree => "2/3",
    }
}
impl PhaseFilter {
    /// Exact registry phases required together.
    pub fn phases(self) -> &'static [crate::TrialPhase] {
        use crate::TrialPhase as P;
        match self {
            Self::Early1 => &[P::Early1],
            Self::One => &[P::Phase1],
            Self::Two => &[P::Phase2],
            Self::Three => &[P::Phase3],
            Self::Four => &[P::Phase4],
            Self::NotApplicable => &[P::NotApplicable],
            Self::OneTwo => &[P::Phase1, P::Phase2],
            Self::TwoThree => &[P::Phase2, P::Phase3],
        }
    }
}

biodata::wire_enum! {
    /// Independent trial providers; results are never folded or used as fallbacks.
    #[derive(Default)]
    TrialSource {
        /// ClinicalTrials.gov registry.
        #[default]
        Ctgov => "ctgov",
        /// NCI requires the user key; accepts NCIt condition, keyword and native phase/status filters.
        Nci => "nci",
    }
}

impl TrialSource {
    /// Native source identity for the selected provider.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Ctgov => crate::Source::ClinicalTrialsGov,
            Self::Nci => crate::Source::Nci,
        }
    }
    /// Whether the registry default can be omitted from a serialized query.
    pub fn is_default(&self) -> bool {
        *self == Self::Ctgov
    }
}

biodata::wire_enum! {
    /// Checked trial capability names, never caller-supplied values.
    TrialFilter {
        /// Registry general text.
        Text => "TEXT",
        /// Registry gene discovery.
        Gene => "--gene",
        /// Registry variant discovery.
        Variant => "--variant",
        /// Registry criteria text.
        Criteria => "--criteria",
        /// Registry intervention text.
        Intervention => "--intervention",
        /// Registry status vocabulary.
        Status => "--status",
        /// Registry phase vocabulary.
        Phase => "--phase",
        /// Registry study type.
        StudyType => "--study-type",
        /// Registry sex vocabulary.
        Sex => "--sex",
        /// Registry age range.
        Age => "--age",
        /// Registry facility text.
        Facility => "--facility",
        /// Registry radius.
        Latitude => "--lat",
        /// Registry sponsor text.
        Sponsor => "--sponsor",
        /// Registry funding vocabulary.
        SponsorType => "--sponsor-type",
        /// Registry lower update bound.
        UpdatedSince => "--updated-since",
        /// Registry upper update bound.
        UpdatedUntil => "--updated-until",
        /// Registry ID conjunction.
        Id => "--id",
        /// Registry results restriction.
        HasResults => "--has-results",
        /// Explicit NCI keyword mode.
        Keyword => "--keyword",
    }
}
impl TrialSearch {
    /// Reject a source disease name instead of silently degrading to free text.
    pub fn nci_disease_invalid(&self) -> bool {
        self.source() == TrialSource::Nci
            && self.condition().iter().any(|v| {
                !v.as_str()
                    .strip_prefix('C')
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            })
    }
    /// Name the first unsupported capability before any source, key, or cache access.
    pub fn unsupported(&self) -> Option<TrialFilter> {
        if self.source() == TrialSource::Ctgov {
            return None;
        }
        use TrialFilter as F;
        [
            (self.query().is_some(), F::Text),
            (self.keyword().len() > 1, F::Keyword),
            (self.gene().is_some(), F::Gene),
            (self.variant().is_some(), F::Variant),
            (self.criteria().is_some(), F::Criteria),
            (!self.intervention().is_empty(), F::Intervention),
            (self.study_type().is_some(), F::StudyType),
            (self.sex().is_some(), F::Sex),
            (self.age().is_some(), F::Age),
            (self.lat().is_some(), F::Latitude),
            (self.sponsor().is_some(), F::Sponsor),
            (self.sponsor_type().is_some(), F::SponsorType),
            (self.updated_since().is_some(), F::UpdatedSince),
            (self.updated_until().is_some(), F::UpdatedUntil),
            (self.id().is_some(), F::Id),
            (self.has_results(), F::HasResults),
        ]
        .into_iter()
        .find_map(|(present, option)| present.then_some(option))
    }
}

mod validation;
