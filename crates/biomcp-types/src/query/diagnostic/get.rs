use super::DiagnosticId;
use super::{DiagnosticGetSource, DiagnosticSection, QueryInvalid, limit};
use biodata::{PmaSupplement, Text};
use serde::{Deserialize, Serialize};
/// Get one exact source record or supported section. A listing is not authorization; a companion row is not a treatment recommendation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct DiagnosticGet {
    #[serde(flatten)]
    fields: GetFields<DiagnosticId>,
    #[serde(skip)]
    record: Text,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "GetFields"))]
struct GetFields<I = Text> {
    /// Exact native identity, including version for GTR.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: I,
    /// Explicit source: GTR accession/version, WHO Product ID, PMA P number,
    /// 510(k) K number or device recall Z-NNNN-YYYY.
    source: DiagnosticGetSource,
    /// GTR: genes, conditions, methods. PMA: regulatory, supplements.
    /// WHO, 510(k) and device recalls have no sections; omit for the card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 1)))]
    section: Option<DiagnosticSection>,
    /// Exact native PMA supplement; absent selects the base row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    supplement: Option<PmaSupplement>,
    /// Maximum native rows; omitted means five rows, or one regulatory fragment.
    #[serde(
        default,
        deserialize_with = "super::super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Zero-based native row or fragment offset.
    #[serde(default)]
    section_offset: u32,
    /// Immutable stored bundle revision retained by a continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle_revision: Option<Text>,
}
crate::query::validation::checked_query!(DiagnosticGet, GetFields);
impl TryFrom<GetFields> for DiagnosticGet {
    type Error = QueryInvalid;
    fn try_from(v: GetFields) -> Result<Self, Self::Error> {
        let invalid = || QueryInvalid::new(&["id", "source"]);
        let id = match v.source {
            DiagnosticGetSource::Gtr => {
                DiagnosticId::Gtr(v.id.as_str().parse().map_err(|_| invalid())?)
            }
            DiagnosticGetSource::WhoIvd => {
                DiagnosticId::Who(v.id.as_str().parse().map_err(|_| invalid())?)
            }
            DiagnosticGetSource::FdaPma => {
                DiagnosticId::Pma(v.id.as_str().parse().map_err(|_| invalid())?)
            }
            DiagnosticGetSource::FdaDeviceRecall => {
                DiagnosticId::Recall(v.id.as_str().parse().map_err(|_| invalid())?)
            }
            DiagnosticGetSource::Fda510k => {
                if !v.id.as_str().starts_with('K') {
                    return Err(invalid());
                }
                DiagnosticId::Clearance(v.id.as_str().parse().map_err(|_| invalid())?)
            }
        };
        validate_source(&v)?;
        if v.section_limit.is_some_and(|v| !(1..=50).contains(&v)) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        Ok(Self {
            record: v.id,
            fields: GetFields {
                id,
                source: v.source,
                section: v.section,
                supplement: v.supplement,
                section_limit: v.section_limit,
                section_offset: v.section_offset,
                bundle_revision: v.bundle_revision,
            },
        })
    }
}
impl DiagnosticGet {
    /// Check a source record and optional PMA supplement with the card defaults.
    pub fn new(
        id: Text,
        source: DiagnosticGetSource,
        supplement: Option<PmaSupplement>,
    ) -> Result<Self, QueryInvalid> {
        Self::try_from(GetFields {
            id,
            source,
            section: None,
            supplement,
            section_limit: None,
            section_offset: 0,
            bundle_revision: None,
        })
    }

    /// Exact checked source identity.
    pub fn id(&self) -> &DiagnosticId {
        &self.fields.id
    }
    /// Original edge-admitted spelling for source record attribution.
    pub fn record(&self) -> &Text {
        &self.record
    }
    /// Native recall identity parsed once by the checked input boundary.
    pub fn recall_number(&self) -> Option<&biodata::DeviceRecallNumber> {
        match self.id() {
            DiagnosticId::Recall(id) => Some(id),
            _ => None,
        }
    }
    /// Explicit selected source.
    pub fn source(&self) -> DiagnosticGetSource {
        self.fields.source
    }
    /// One supported named page.
    pub fn section(&self) -> Option<DiagnosticSection> {
        self.fields.section
    }
    /// Explicit selected supplement, absent for the base.
    pub fn supplement(&self) -> Option<&PmaSupplement> {
        self.fields.supplement.as_ref()
    }
    /// Maximum rows or fragments per section page.
    pub fn section_limit(&self) -> u32 {
        self.fields.section_limit.unwrap_or_else(|| {
            if self.fields.section == Some(DiagnosticSection::Regulatory) {
                1
            } else {
                limit()
            }
        })
    }
    /// Native row or text-fragment offset.
    pub fn section_offset(&self) -> u32 {
        self.fields.section_offset
    }
    /// Stored source snapshot.
    pub fn bundle_revision(&self) -> Option<&Text> {
        self.fields.bundle_revision.as_ref()
    }
    /// Continue the exact selected section and record.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut q = self.clone();
        q.fields.section_offset = offset;
        q
    }
    /// Select a supported section; invalid source combinations remain refused.
    pub fn with_section(&self, section: DiagnosticSection) -> Result<Self, QueryInvalid> {
        let mut q = self.fields.clone();
        q.section = Some(section);
        q.section_offset = 0;
        validate_source(&q)?;
        Ok(Self {
            fields: q,
            record: self.record.clone(),
        })
    }
    /// Open a base or supplemental card with the original constructor defaults.
    pub fn with_supplement(&self, supplement: Option<PmaSupplement>) -> Result<Self, QueryInvalid> {
        let fields = GetFields {
            id: self.fields.id.clone(),
            source: self.source(),
            section: None,
            supplement,
            section_limit: None,
            section_offset: 0,
            bundle_revision: None,
        };
        validate_source(&fields)?;
        Ok(Self {
            fields,
            record: self.record.clone(),
        })
    }
    /// Bind the selected record to one immutable stored revision.
    pub fn with_bundle_revision(&self, revision: Text) -> Self {
        let mut q = self.clone();
        q.fields.bundle_revision = Some(revision);
        q
    }
}

fn validate_source<I>(v: &GetFields<I>) -> Result<(), QueryInvalid> {
    let valid_section = v
        .section
        .is_none_or(|section| v.source.sections().contains(&section));
    if !valid_section {
        return Err(QueryInvalid::new(&["section", "source"]));
    }
    if v.source != DiagnosticGetSource::FdaPma && v.supplement.is_some() {
        return Err(QueryInvalid::new(&["supplement", "source"]));
    }
    if v.supplement.is_some() && v.section == Some(DiagnosticSection::Supplements) {
        return Err(QueryInvalid::new(&["supplement", "section"]));
    }
    if matches!(
        v.source,
        DiagnosticGetSource::WhoIvd
            | DiagnosticGetSource::Fda510k
            | DiagnosticGetSource::FdaDeviceRecall
    ) {
        if v.section_limit.is_some() {
            return Err(QueryInvalid::new(&["section_limit", "source"]));
        }
        if v.section_offset != 0 {
            return Err(QueryInvalid::new(&["section_offset", "source"]));
        }
    }
    if matches!(
        v.source,
        DiagnosticGetSource::Fda510k | DiagnosticGetSource::FdaDeviceRecall
    ) && v.bundle_revision.is_some()
    {
        return Err(QueryInvalid::new(&["bundle_revision", "source"]));
    }
    Ok(())
}
