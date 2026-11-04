mod id;
mod search_validation;
use super::QueryInvalid;
use biodata::{DiagnosticDeviceId, FdaDrugApplication, GeneSymbol, NcbiGeneId, PartialDate, Text};
pub use id::DiagnosticId;
use search_validation::{validate_device, validate_recall};
use serde::{Deserialize, Serialize};
mod get;
pub use get::DiagnosticGet;

biodata::wire_enum! {
    /// One explicit diagnostic discovery source.
    DiagnosticSearchSource { #[doc = "Native Gtr spelling."] Gtr => "gtr",
 #[doc = "Native FdaCdx spelling."] FdaCdx => "fda-cdx",
 #[doc = "Native PMA submission records."] FdaPma => "fda-pma",
 #[doc = "Native 510(k) clearance records."] Fda510k => "fda-510k",
 #[doc = "Stored WHO product headers."] WhoIvd => "who-ivd",
 #[doc = "Native recalls for every medical device."] FdaDeviceRecall => "fda-device-recall" }
}
biodata::wire_enum! {
    /// One explicit exact diagnostic record source.
    DiagnosticGetSource { #[doc = "Native Gtr spelling."] Gtr => "gtr",
 #[doc = "Native WhoIvd spelling."] WhoIvd => "who-ivd",
 #[doc = "Native FdaPma spelling."] FdaPma => "fda-pma",
 #[doc = "Native 510(k) clearance spelling."] Fda510k => "fda-510k",
 #[doc = "Exact native medical device recall."] FdaDeviceRecall => "fda-device-recall" }
}
biodata::wire_enum! {
    described
    /// Source-supported diagnostic sections; no all section.
    DiagnosticSection {
        #[doc = "GTR native gene rows; five per page."] Genes => "genes",
        #[doc = "GTR native condition rows; five per page."] Conditions => "conditions",
        #[doc = "GTR native method entries; five per page."] Methods => "methods",
        #[doc = "FDA SSED PDF text: 25 MiB, 100 pages and 6144 escaped bytes per fragment; scans without text are explicit."] Summary => "summary",
        #[doc = "Selected PMA submission statement; one fragment per page."] Regulatory => "regulatory",
        #[doc = "PMA base application supplement headers; five per page."] Supplements => "supplements"
    }
}
fn limit() -> u32 {
    5
}
/// Search five source records by default. FDA device recalls cover every medical device; totals count recall records, without harm inference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct DiagnosticSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Explicit native GTR, WHO IVD, FDA PMA, 510(k), companion or medical device recall discovery.
    source: DiagnosticSearchSource,
    /// Exact source GeneID, exclusive with gene symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gene_id: Option<NcbiGeneId>,
    /// Exact source gene_symbol field, without alias expansion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gene: Option<GeneSymbol>,
    /// Exact MedGen:CUI identifier only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(regex(pattern = "^MedGen:C[0-9]+$")))]
    condition_id: Option<Text>,
    /// Exact native test_type value.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    test_type: Option<biodata::GtrTestType>,
    /// Exact PMA application, application/supplement or native K identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_id: Option<DiagnosticDeviceId>,
    /// Cell-owned typed NDA/BLA application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    drug_application: Option<FdaDrugApplication>,
    /// Exact table-0 biomarker cell; unsupported in the group table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    biomarker: Option<Text>,
    /// Literal whole-word phrase in WHO's Pathogen/Disease/Marker cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<Text>,
    /// WHO manufacturer words, native FDA applicant (PMA/510(k)) or recalling_firm phrase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    manufacturer: Option<Text>,
    /// Exact native FDA device recall product code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    product_code: Option<Text>,
    /// Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initiated_from: Option<PartialDate>,
    /// Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initiated_to: Option<PartialDate>,
    /// Native trade_name (PMA) or device_name (510(k)) phrase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<Text>,
    /// Inclusive native decision date, YYYY-MM-DD; PMA and 510(k) only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decision_from: Option<PartialDate>,
    /// Inclusive native decision date, YYYY-MM-DD; PMA and 510(k) only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decision_to: Option<PartialDate>,
    /// Five results by default; at most fifty source rows or product headers.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Zero-based source result offset.
    #[serde(default)]
    offset: u32,
    /// Immutable stored bundle revision retained by a continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle_revision: Option<Text>,
}
crate::query::validation::checked_query!(DiagnosticSearch, SearchFields);
impl TryFrom<SearchFields> for DiagnosticSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        let gtr = v.gene_id.is_some()
            || v.gene.is_some()
            || v.condition_id.is_some()
            || v.test_type.is_some();
        let cdx = v.device_id.is_some() || v.drug_application.is_some() || v.biomarker.is_some();
        let who = v.marker.is_some() || v.manufacturer.is_some();
        let recall =
            v.product_code.is_some() || v.initiated_from.is_some() || v.initiated_to.is_some();
        validate_device(&v, gtr || cdx || v.marker.is_some() || recall)?;
        validate_recall(&v, gtr || cdx, recall)?;
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if [
            v.gene.is_some(),
            v.gene_id.is_some(),
            v.condition_id.is_some(),
        ]
        .into_iter()
        .filter(|v| *v)
        .count()
            > 1
        {
            return Err(QueryInvalid::new(&["gene", "gene_id", "condition_id"]));
        }
        if v.source == DiagnosticSearchSource::Gtr && (cdx || who) {
            return Err(QueryInvalid::new(&[
                "source",
                "device_id",
                "drug_application",
                "biomarker",
                "marker",
                "manufacturer",
            ]));
        }
        if v.source == DiagnosticSearchSource::FdaCdx && (gtr || who) {
            return Err(QueryInvalid::new(&[
                "source",
                "gene_id",
                "gene",
                "condition_id",
                "type",
                "marker",
                "manufacturer",
            ]));
        }
        if v.source == DiagnosticSearchSource::WhoIvd && (gtr || cdx) {
            return Err(QueryInvalid::new(&[
                "source",
                "gene_id",
                "gene",
                "condition_id",
                "type",
                "device_id",
                "drug_application",
                "biomarker",
            ]));
        }
        if v.source == DiagnosticSearchSource::WhoIvd
            && [v.marker.as_ref(), v.manufacturer.as_ref()]
                .into_iter()
                .flatten()
                .any(|v| !v.as_str().chars().any(char::is_alphanumeric))
        {
            return Err(QueryInvalid::new(&["marker", "manufacturer"]));
        }
        if v.device_id
            .as_ref()
            .is_some_and(|id| !(id.as_str().starts_with('P') || id.as_str().starts_with('K')))
        {
            return Err(QueryInvalid::new(&["device_id"]));
        }
        if v.condition_id.as_ref().is_some_and(|id| {
            id.as_str()
                .strip_prefix("MedGen:C")
                .is_none_or(|id| id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()))
        }) {
            return Err(QueryInvalid::new(&["condition_id"]));
        }
        Ok(Self(v))
    }
}
impl DiagnosticSearch {
    /// Search companion diagnostics for one checked native device identity.
    pub fn companion(device_id: DiagnosticDeviceId) -> Result<Self, QueryInvalid> {
        Self::try_from(SearchFields {
            source: DiagnosticSearchSource::FdaCdx,
            device_id: Some(device_id),
            gene_id: None,
            gene: None,
            condition_id: None,
            test_type: None,
            drug_application: None,
            biomarker: None,
            marker: None,
            manufacturer: None,
            name: None,
            decision_from: None,
            decision_to: None,
            product_code: None,
            initiated_from: None,
            initiated_to: None,
            limit: limit(),
            offset: 0,
            bundle_revision: None,
        })
    }

    /// Explicit source, without fallback.
    pub fn source(&self) -> DiagnosticSearchSource {
        self.0.source
    }
    /// Exact source GeneID.
    pub fn gene_id(&self) -> Option<NcbiGeneId> {
        self.0.gene_id
    }
    /// Exact source symbol.
    pub fn gene(&self) -> Option<&GeneSymbol> {
        self.0.gene.as_ref()
    }
    /// Exact source MedGen CUI.
    pub fn condition_id(&self) -> Option<&Text> {
        self.0.condition_id.as_ref()
    }
    /// Exact native test type.
    pub fn test_type(&self) -> Option<biodata::GtrTestType> {
        self.0.test_type
    }
    /// Selected PMA base, supplement or native K identity.
    pub fn device_id(&self) -> Option<&DiagnosticDeviceId> {
        self.0.device_id.as_ref()
    }
    /// Owned NDA/BLA application.
    pub fn drug_application(&self) -> Option<&FdaDrugApplication> {
        self.0.drug_application.as_ref()
    }
    /// Whole exact biomarker cell.
    pub fn biomarker(&self) -> Option<&Text> {
        self.0.biomarker.as_ref()
    }
    /// Literal WHO marker words.
    pub fn marker(&self) -> Option<&Text> {
        self.0.marker.as_ref()
    }
    /// WHO manufacturer words, native FDA applicant (PMA/510(k)) or recalling_firm phrase.
    pub fn manufacturer(&self) -> Option<&Text> {
        self.0.manufacturer.as_ref()
    }
    /// Exact native recall device category code.
    pub fn product_code(&self) -> Option<&Text> {
        self.0.product_code.as_ref()
    }
    /// Inclusive source initiation range.
    pub fn initiated_from(&self) -> Option<PartialDate> {
        self.0.initiated_from
    }
    /// Inclusive source initiation range.
    pub fn initiated_to(&self) -> Option<PartialDate> {
        self.0.initiated_to
    }
    /// Native device name phrase.
    pub fn name(&self) -> Option<&Text> {
        self.0.name.as_ref()
    }
    /// Inclusive native decision range lower bound.
    pub fn decision_from(&self) -> Option<PartialDate> {
        self.0.decision_from
    }
    /// Inclusive native decision range upper bound.
    pub fn decision_to(&self) -> Option<PartialDate> {
        self.0.decision_to
    }
    /// Native rows per page.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native row offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Selected immutable bundle revision.
    pub fn bundle_revision(&self) -> Option<&Text> {
        self.0.bundle_revision.as_ref()
    }

    /// Bind a stored query to its immutable source snapshot.
    pub fn with_bundle_revision(&self, revision: Text) -> Self {
        let mut v = self.clone();
        v.0.bundle_revision = Some(revision);
        v
    }
}
impl crate::Paged for DiagnosticSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

impl DiagnosticGetSource {
    /// Source identity for this explicit native selector.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Gtr => crate::Source::Gtr,
            Self::WhoIvd => crate::Source::WhoIvd,
            Self::FdaPma => crate::Source::FdaPma,
            Self::Fda510k => crate::Source::Fda510k,
            Self::FdaDeviceRecall => crate::Source::FdaDeviceRecall,
        }
    }
}

impl DiagnosticSearchSource {
    /// Source identity for this explicit native selector.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Gtr => crate::Source::Gtr,
            Self::FdaCdx => crate::Source::FdaCdx,
            Self::FdaPma => crate::Source::FdaPma,
            Self::Fda510k => crate::Source::Fda510k,
            Self::WhoIvd => crate::Source::WhoIvd,
            Self::FdaDeviceRecall => crate::Source::FdaDeviceRecall,
        }
    }
}

impl DiagnosticGetSource {
    /// Named disclosures implemented by the exact-record endpoint.
    pub const fn sections(self) -> &'static [DiagnosticSection] {
        use DiagnosticSection::{Conditions, Genes, Methods, Regulatory, Summary, Supplements};
        match self {
            Self::Gtr => &[Genes, Conditions, Methods],
            Self::FdaPma => &[Regulatory, Supplements, Summary],
            Self::WhoIvd | Self::Fda510k | Self::FdaDeviceRecall => &[],
        }
    }
}
