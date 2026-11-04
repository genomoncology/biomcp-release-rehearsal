use crate::{Build, VariantInput, VariantNamespace};
use serde::{Deserialize, Serialize};
#[doc = " Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or \
    VCF-style genomic coordinates such as 7-140753336-A-T through the shared resolver. \
    MyVariant identity lookups default to GRCh38; --build overrides it. \
    Chromosome genomic HGVS requires --build. Use search variant --gene BRAF --protein V600E \
    or search variant --gene PTEN --coding c.333G>T for gene shorthand. \
    Every candidate remains available; source assertions do not classify the variant for the caller."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(with = "Fields", extend("x-reply-query-titles" = ["VariantGetQuery"]))
)]
pub struct VariantGet(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or VCF-style genomic coordinates.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    input: VariantInput,
    /// Named source sections; omitted by default. Each calls its own source. all keeps bounded pages and source skips; it never enables a missing key.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 1)))]
    section: Box<[VariantSection]>,
    /// Rows per named source section page; omitted uses the native source default.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Source rows to skip independently of identity candidate paging.
    #[serde(default)]
    section_offset: u32,
    /// Selected CIViC molecular profile for native claim paging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 2147483647)))]
    civic_profile: Option<u64>,
    /// Explicit tumor context for keyed OncoKB annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tumor_type: Option<crate::Text>,
    /// Native CIViC claim collection within the selected profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_kind: Option<CivicClaimKind>,
    /// Native CIViC cursor, retained unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_cursor: Option<crate::Text>,
    #[serde(flatten)]
    evidence_filter: crate::CivicEvidenceFilter,
    /// Resolver for reference-qualified notation; never repairs the input locally.
    #[serde(default = "resolver")]
    resolver: VariantResolver,
    /// MyVariant identity assembly defaults to GRCh38; chromosome genomic HGVS requires an explicit build.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    build: Option<Build>,
    /// Candidate rows per page; more remain reachable through typed next commands.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Candidate rows to skip in source order.
    #[serde(default)]
    offset: u32,
}
fn default_section_limit() -> u32 {
    3
}
fn default_limit() -> u32 {
    10
}
fn resolver() -> VariantResolver {
    VariantResolver::Car
}
crate::query::validation::checked_query!(VariantGet, Fields);
impl TryFrom<Fields> for VariantGet {
    type Error = super::gene::QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        if !fields.evidence_filter.is_empty()
            && !fields
                .section
                .iter()
                .any(|s| matches!(s, VariantSection::Civic | VariantSection::All))
        {
            return Err(super::gene::QueryInvalid::new(&[
                "evidence_level",
                "evidence_status",
                "civic_kind",
                "section",
            ]));
        }
        if !(1..=50).contains(&fields.limit) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        if fields.section_limit.is_some_and(|n| !(1..=50).contains(&n)) {
            return Err(super::gene::QueryInvalid::new(&["section_limit"]));
        }
        if fields
            .civic_profile
            .is_some_and(|id| id == 0 || id > i32::MAX as u64)
        {
            return Err(super::gene::QueryInvalid::new(&["civic_profile"]));
        }
        if fields.civic_profile.is_some() != fields.civic_kind.is_some() {
            return Err(super::gene::QueryInvalid::new(&[
                "civic_profile",
                "civic_kind",
            ]));
        }
        if fields.civic_cursor.is_some() && fields.civic_profile.is_none() {
            return Err(super::gene::QueryInvalid::new(&[
                "civic_cursor",
                "civic_profile",
            ]));
        }
        if fields.civic_profile.is_some()
            && !fields
                .section
                .iter()
                .any(|s| matches!(s, VariantSection::Civic | VariantSection::All))
        {
            return Err(super::gene::QueryInvalid::new(&[
                "civic_profile",
                "section",
            ]));
        }
        if fields.section.contains(&VariantSection::Oncokb)
            && (fields.build.is_none() || fields.tumor_type.is_none())
        {
            return Err(super::gene::QueryInvalid::new(&[
                "section",
                "build",
                "tumor_type",
            ]));
        }
        let mut query = Self::new(fields.input, fields.resolver, fields.build)?;
        query.0.tumor_type = fields.tumor_type;
        query.0.civic_profile = fields.civic_profile;
        query.0.civic_kind = fields.civic_kind;
        query.0.civic_cursor = fields.civic_cursor;
        query.0.evidence_filter = fields.evidence_filter;
        query.0.section_limit = fields.section_limit;
        query.0.section_offset = fields.section_offset;
        query.0.section = fields.section;
        query.0.limit = fields.limit;
        query.0.offset = fields.offset;
        Ok(query)
    }
}
impl VariantGet {
    /// Default identity lookup assembly; --build overrides this setting.
    pub const DEFAULT_BUILD: Build = Build::GRCh38;
    /// Check coordinate assembly and preserve resolver choice.
    pub fn new(
        input: VariantInput,
        resolver: VariantResolver,
        build: Option<Build>,
    ) -> Result<Self, super::gene::QueryInvalid> {
        let build = build.or_else(|| input.coordinate_form().then_some(Self::DEFAULT_BUILD));
        if input.namespace() == VariantNamespace::Unsupported {
            return Err(super::gene::QueryInvalid::new(&["input"]));
        }
        if resolver == VariantResolver::Variantvalidator && build.is_none() {
            return Err(super::gene::QueryInvalid::new(&["resolver", "build"]));
        }
        if build.is_some_and(|b| !matches!(b, Build::GRCh37 | Build::GRCh38)) {
            return Err(super::gene::QueryInvalid::new(&["build"]));
        }
        if input.namespace() == VariantNamespace::Chromosome && build.is_none() {
            return Err(super::gene::QueryInvalid::new(&["input", "build"]));
        }
        Ok(Self(Fields {
            input,
            section: Box::default(),
            section_limit: None,
            section_offset: 0,
            civic_profile: None,
            tumor_type: None,
            civic_kind: None,
            civic_cursor: None,
            evidence_filter: Default::default(),
            resolver,
            build,
            limit: default_limit(),
            offset: 0,
        }))
    }
    /// Explicit section selection; an empty list means identity only.
    pub fn sections(&self) -> &[VariantSection] {
        &self.0.section
    }
    /// Keep the input and page selectors while choosing sections.
    pub fn with_sections(mut self, sections: Box<[VariantSection]>) -> Self {
        self.0.section = sections;
        self
    }
    /// Caller tumor context, never inferred from an allele or gene.
    pub fn tumor_type(&self) -> Option<&crate::Text> {
        self.0.tumor_type.as_ref()
    }
    /// Supply explicit tumor context while checking the same public request rules.
    pub fn with_tumor_type(
        mut self,
        tumor: crate::Text,
    ) -> Result<Self, super::gene::QueryInvalid> {
        self.0.tumor_type = Some(tumor);
        self.0.try_into()
    }
    /// Selected native CIViC profile.
    pub fn civic_profile(&self) -> Option<u64> {
        self.0.civic_profile
    }
    /// Selected native CIViC claim collection.
    pub fn civic_kind(&self) -> Option<CivicClaimKind> {
        self.0.civic_kind
    }
    /// Native evidence selectors retained across pages.
    pub fn evidence_filter(&self) -> &crate::CivicEvidenceFilter {
        &self.0.evidence_filter
    }
    /// Selected native CIViC cursor.
    pub fn civic_cursor(&self) -> Option<&crate::Text> {
        self.0.civic_cursor.as_ref()
    }
    /// Continue a native CIViC collection without changing identity page selectors.
    pub fn with_civic_page(
        &self,
        profile: u64,
        kind: CivicClaimKind,
        cursor: Option<crate::Text>,
    ) -> Result<Self, super::gene::QueryInvalid> {
        let mut fields = self.0.clone();
        fields.civic_profile = Some(profile);
        fields.civic_kind = Some(kind);
        fields.civic_cursor = cursor;
        fields.try_into()
    }
    /// Pin a source candidate without changing section paging or source selectors.
    pub fn for_candidate(&self, input: VariantInput) -> Result<Self, super::gene::QueryInvalid> {
        let mut fields = self.0.clone();
        fields.input = input;
        fields.offset = 0;
        fields.try_into()
    }
    /// Select an assembly through the same checked request boundary.
    pub fn with_build(&self, build: Build) -> Result<Self, super::gene::QueryInvalid> {
        let mut fields = self.0.clone();
        fields.build = Some(build);
        fields.try_into()
    }
    /// Named section rows per page.
    pub fn section_limit(&self) -> u32 {
        self.0.section_limit.unwrap_or_else(default_section_limit)
    }
    /// Assertion rows per page, using the selected source's default when omitted.
    pub fn assertion_limit(&self, source: crate::AssertionSource) -> u32 {
        self.0
            .section_limit
            .unwrap_or_else(|| source.default_limit())
    }
    /// Named section rows to skip.
    pub fn section_offset(&self) -> u32 {
        self.0.section_offset
    }
    /// Page source rows without changing the selected identity candidate page.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut q = self.clone();
        q.0.section_offset = offset;
        q
    }
    /// Candidate rows per page.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Candidate page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }

    /// Unchanged checked identifier.
    pub fn input(&self) -> &VariantInput {
        &self.0.input
    }
    /// Explicit resolver service.
    pub fn resolver(&self) -> VariantResolver {
        self.0.resolver
    }
    /// Assembly selected for a MyVariant identity request.
    pub fn myvariant_build(&self) -> Build {
        self.build().unwrap_or(Self::DEFAULT_BUILD)
    }
    /// Echo the assembly selected for a MyVariant identity request.
    pub fn with_myvariant_build(mut self) -> Self {
        self.0.build = Some(self.myvariant_build());
        self
    }
    /// Caller assembly; never a default.
    pub fn build(&self) -> Option<Build> {
        self.0.build
    }
}
impl crate::Paged for VariantGet {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

biodata::wire_enum! {
    /// Explicit source service for reference-qualified notation.
    VariantResolver {
        /// ClinGen Allele Registry.
        Car => "car",
        /// Mutalyzer.
        Mutalyzer => "mutalyzer",
        /// VariantValidator.
        Variantvalidator => "variantvalidator",
    }
}
impl VariantResolver {
    /// Declared source policy for the selected service.
    pub fn source(self) -> crate::Source {
        match self {
            Self::Car => crate::Source::AlleleRegistry,
            Self::Mutalyzer => crate::Source::Mutalyzer,
            Self::Variantvalidator => crate::Source::VariantValidator,
        }
    }
}
biodata::canonical_enum! {
    /// Named variant source sections, without automatic expansion of identity cards.
    VariantSection, SourceUnknown {
        /// ClinVar native assertions.
        Clinvar => "clinvar",
        /// ClinGen native assertions.
        Erepo => "erepo",
        /// gnomAD population observations.
        Population => "population",
        /// GWAS Catalog associations.
        Gwas => "gwas",
        /// CIViC molecular-profile evidence.
        Civic => "civic",
        /// Keyed AlphaGenome native numeric scorer values.
        AlphaGenome => "alphagenome",
        /// Keyed OncoKB annotation with explicit tumor context and build.
        Oncokb => "oncokb",
        /// Cancer Hotspots candidate residue counts; no established exact allele join or clinical actionability.
        Hotspots => "hotspots",
        /// MyVariant source predictor scores.
        Predict => "predict",
        /// MyVariant conservation scores.
        Conservation => "conservation",
        /// Historical COSMIC fragments carried by MyVariant; parent-to-fragment allele identity remains unverified.
        Cosmic => "cosmic",
        /// Every implemented variant section.
        All => "all",
    }
}
biodata::wire_enum! {
    /// CIViC native claim collection.
    CivicClaimKind {
        /// Native evidence items.
        Evidence=>"evidence",
        /// Native assertions.
        Assertions=>"assertions",
    }
}

impl VariantSection {
    /// Direct publisher; component sections retain their MyVariant route.
    pub const fn direct_source(self) -> Option<crate::Source> {
        use crate::Source;
        match self {
            Self::Clinvar => Some(Source::ClinVar),
            Self::Erepo => Some(Source::Erepo),
            Self::Population => Some(Source::Gnomad),
            Self::Gwas => Some(Source::Gwas),
            Self::Civic => Some(Source::Civic),
            Self::Oncokb => Some(Source::Oncokb),
            Self::AlphaGenome => Some(Source::AlphaGenome),
            Self::Hotspots => Some(Source::CancerHotspots),
            Self::Predict | Self::Conservation | Self::Cosmic | Self::All => None,
        }
    }
}
