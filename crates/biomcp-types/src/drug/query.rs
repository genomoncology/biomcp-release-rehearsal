use super::DrugSource;
use super::source::{civic_id, inferred_source};
use crate::{CivicClaimKind, QueryInvalid};
use biodata::{DrugId, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Explicit drug disclosures; identity cards never expand into these sections.
    DrugSection, SourceUnknown {
        /// All native identity cross-references and product links, mechanically paged.
        Xrefs => "xrefs",
        /// Native target and mechanism relations.
        Targets => "targets",
        /// Native indication wording.
        Indications => "indications",
        /// Native CIViC therapy evidence and profile context.
        Civic => "civic",
        /// One exact native label heading.
        Label => "label",
        /// Native application and product history rows.
        Approvals => "approvals",
        /// Source-scoped regulator events.
        Regulatory => "regulatory",
        /// Native DDInter partner rows and levels; five rows per default page, with explicit section limits preserved.
        Interactions => "interactions",
    }
}
/// Retrieve a native ID or a complete source name.
/// Brands rank first; Drugs@FDA then prefers original applications and earliest approval, otherwise source search order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
pub struct DrugGet {
    #[serde(flatten)]
    fields: GetFields,
    #[serde(skip)]
    native_id: NativeDrugId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum NativeDrugId {
    Recall(biodata::DrugRecallNumber),
    Civic(std::num::NonZeroU32),
    Other,
    Name,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Published source-native identifier or complete brand or generic name.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    input: Text,
    /// Source inferred from an unambiguous requested section; otherwise MyChem.
    #[serde(default, deserialize_with = "selected_source")]
    #[cfg_attr(feature = "schema", schemars(with = "DrugSource", extend("default" = "mychem")))]
    source: Option<DrugSource>,
    /// Explicit named disclosures; empty means identity only.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(title = "Source sections.", extend("x-positional" = 1)))]
    section: Box<[DrugSection]>,
    /// Named section page size, refused on an identity-only card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    section_limit: Option<u32>,
    /// Named section continuation, refused on an identity-only card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_offset: Option<u32>,
    #[doc = "Exact openFDA SPL field.\nExact published openFDA SPL section field, including table fields; required for the label section."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label_section: Option<Text>,
    #[doc = "FDA product number.\nExplicit native application product number, only for approvals."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    product_number: Option<Text>,
    #[doc = "FDA letter document ID.\nExact native Drugs@FDA approval letter document ID. Text is bounded to 25 MiB, 100 pages"]
    /// and 6144 escaped bytes per fragment. No-text scans are explicit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    document_id: Option<Text>,
    /// Selected native CIViC molecular profile for a drug-scoped claim stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 2147483647)))]
    civic_profile: Option<u64>,
    #[doc = "CIViC collection.\nIndependent native evidence or assertion collection within the selected profile."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_kind: Option<CivicClaimKind>,
    #[doc = "CIViC continuation.\nNative continuation cursor for that exact drug, profile and claim collection."]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    civic_cursor: Option<Text>,
    #[serde(flatten)]
    evidence_filter: crate::CivicEvidenceFilter,
}
crate::query::validation::checked_query!(DrugGet, GetFields);
impl TryFrom<GetFields> for DrugGet {
    type Error = QueryInvalid;
    fn try_from(mut fields: GetFields) -> Result<Self, Self::Error> {
        let source = fields
            .source
            .unwrap_or_else(|| inferred_source(&fields.section));
        fields.source = Some(source);
        validate_sections(&fields)?;
        let native_id = if !fields
            .input
            .as_str()
            .parse::<DrugId>()
            .is_ok_and(|id| source.native_id(&id))
        {
            NativeDrugId::Name
        } else {
            match source {
                DrugSource::FdaDrugRecall => NativeDrugId::Recall(
                    fields
                        .input
                        .to_string()
                        .parse()
                        .map_err(|_| QueryInvalid::new(&["source", "input"]))?,
                ),
                DrugSource::Civic => NativeDrugId::Civic(
                    civic_id(&fields.input.to_string())
                        .ok_or_else(|| QueryInvalid::new(&["source", "input"]))?,
                ),
                source => {
                    if !fields
                        .input
                        .as_str()
                        .parse::<DrugId>()
                        .is_ok_and(|id| source.accepts(&id))
                    {
                        return Err(QueryInvalid::new(&["source", "input"]));
                    }
                    NativeDrugId::Other
                }
            }
        };
        validate_civic(&fields)?;
        Ok(Self { fields, native_id })
    }
}
fn validate_sections(fields: &GetFields) -> Result<(), QueryInvalid> {
    if fields.document_id.is_some()
        && (fields.source != Some(DrugSource::Drugsfda)
            || fields.section.as_ref() != [DrugSection::Approvals])
    {
        return Err(QueryInvalid::new(&["document_id", "source", "section"]));
    }
    if fields.section_limit.is_some_and(|n| !(1..=50).contains(&n)) {
        return Err(QueryInvalid::new(&["section_limit"]));
    }
    if fields.section.is_empty()
        && (fields.section_limit.is_some() || fields.section_offset.is_some())
    {
        return Err(QueryInvalid::new(&[
            "section",
            "section_limit",
            "section_offset",
        ]));
    }
    if fields.section.contains(&DrugSection::Label) != fields.label_section.is_some() {
        let cause = QueryInvalid::new(&["label_section", "section"]);
        return Err(if fields.label_section.is_none() {
            cause.with_message(crate::ErrorMessage::LabelSectionMissing)
        } else {
            cause
        });
    }
    if fields.product_number.is_some() && !fields.section.contains(&DrugSection::Approvals) {
        return Err(QueryInvalid::new(&["product_number", "section"]));
    }
    if fields.source == Some(DrugSource::FdaDrugRecall) && !fields.section.is_empty() {
        return Err(QueryInvalid::new(&["source", "section"]));
    }
    Ok(())
}
fn validate_civic(fields: &GetFields) -> Result<(), QueryInvalid> {
    if !fields.evidence_filter.is_empty() && fields.civic_kind != Some(CivicClaimKind::Evidence) {
        return Err(QueryInvalid::new(&[
            "evidence_level",
            "evidence_status",
            "civic_kind",
        ]));
    }
    if fields
        .civic_profile
        .is_some_and(|id| id == 0 || id > i32::MAX as u64)
    {
        return Err(QueryInvalid::new(&["civic_profile"]));
    }
    if fields.civic_profile.is_some() && fields.civic_kind.is_none() {
        return Err(QueryInvalid::new(&["civic_profile", "civic_kind"]));
    }
    if fields.civic_cursor.is_some() && fields.civic_kind.is_none() {
        return Err(QueryInvalid::new(&["civic_cursor", "civic_kind"]));
    }
    if fields.civic_kind.is_some()
        && (fields.source != Some(DrugSource::Civic)
            || !fields.section.contains(&DrugSection::Civic))
    {
        return Err(QueryInvalid::new(&["civic_kind", "source", "section"]));
    }
    Ok(())
}
impl DrugGet {
    fn with_fields(&self, fields: GetFields) -> Result<Self, QueryInvalid> {
        validate_sections(&fields)?;
        validate_civic(&fields)?;
        Ok(Self {
            fields,
            native_id: self.native_id.clone(),
        })
    }
    /// Whether this input requires exact source name resolution.
    pub fn is_name(&self) -> bool {
        self.native_id == NativeDrugId::Name
    }
    /// Preserve section selectors while replacing a resolved name with its native ID.
    pub fn with_native_id(&self, input: DrugId) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.input = Text::new(&input.to_string()).map_err(|_| QueryInvalid::new(&["input"]))?;
        fields.try_into()
    }
    /// FDA recall identity retained by the checked boundary.
    pub fn recall_number(&self) -> Option<&biodata::DrugRecallNumber> {
        match &self.native_id {
            NativeDrugId::Recall(id) => Some(id),
            _ => None,
        }
    }
    /// CIViC therapy number retained by the checked boundary.
    pub fn civic_id(&self) -> Option<std::num::NonZeroU32> {
        match self.native_id {
            NativeDrugId::Civic(id) => Some(id),
            _ => None,
        }
    }
    /// Select one exact native identity, with no sections requested.
    pub fn new(input: DrugId, source: DrugSource) -> Result<Self, QueryInvalid> {
        Self::named(
            Text::new(&input.to_string()).map_err(|_| QueryInvalid::new(&["input"]))?,
            source,
        )
    }
    /// Resolve a complete source name through the same checked boundary as CLI input.
    pub fn named(input: Text, source: DrugSource) -> Result<Self, QueryInvalid> {
        GetFields {
            input,
            source: Some(source),
            section: Box::default(),
            section_limit: None,
            section_offset: None,
            label_section: None,
            product_number: None,
            document_id: None,
            civic_profile: None,
            civic_kind: None,
            civic_cursor: None,
            evidence_filter: Default::default(),
        }
        .try_into()
    }
    /// Retry a source-stated alias while preserving the requested sections and filters.
    pub fn with_name(&self, input: Text) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.input = input;
        fields.try_into()
    }
    /// Continue one native claim collection while preserving selected drug, source and section paging.
    pub fn with_civic_page(
        &self,
        profile: u64,
        kind: CivicClaimKind,
        cursor: Option<Text>,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.civic_profile = Some(profile);
        fields.civic_kind = Some(kind);
        fields.civic_cursor = cursor;
        self.with_fields(fields)
    }
    /// Continue a therapy-wide claim stream without selecting a molecular profile.
    pub fn with_civic_stream(
        &self,
        kind: CivicClaimKind,
        cursor: Option<Text>,
    ) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.civic_kind = Some(kind);
        fields.civic_cursor = cursor;
        self.with_fields(fields)
    }
    /// Selected native drug-scoped CIViC molecular profile.
    pub fn civic_profile(&self) -> Option<u64> {
        self.fields.civic_profile
    }
    /// Selected independent native claim collection.
    pub fn civic_kind(&self) -> Option<CivicClaimKind> {
        self.fields.civic_kind
    }
    /// Native evidence selectors retained across pages.
    pub fn evidence_filter(&self) -> &crate::CivicEvidenceFilter {
        &self.fields.evidence_filter
    }
    /// Original native cursor for that selected claim collection.
    pub fn civic_cursor(&self) -> Option<&Text> {
        self.fields.civic_cursor.as_ref()
    }
    /// Selected native approval document ID.
    pub fn document_id(&self) -> Option<&Text> {
        self.fields.document_id.as_ref()
    }
    /// Unchanged published identifier.
    pub fn input(&self) -> &Text {
        &self.fields.input
    }
    /// Source-native identity after name resolution.
    pub fn native_input(&self) -> Result<DrugId, QueryInvalid> {
        self.fields
            .input
            .as_str()
            .parse()
            .map_err(|_| QueryInvalid::new(&["input"]))
    }
    /// Required selected source.
    pub fn source(&self) -> DrugSource {
        self.fields.source.unwrap_or(DrugSource::Mychem)
    }
    /// Explicit page bound; command handlers apply their existing default when absent.
    pub fn section_limit(&self) -> Option<u32> {
        self.fields.section_limit
    }
    /// Set a checked section page size without changing the selected source or disclosure.
    pub fn with_section_limit(&self, limit: u32) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section_limit = Some(limit);
        self.with_fields(fields)
    }
    /// Explicit section continuation offset.
    pub fn section_offset(&self) -> Option<u32> {
        self.fields.section_offset
    }
}

impl DrugGet {
    /// Whether the selected source serves every requested disclosure in one call.
    pub fn supports_sections(&self) -> bool {
        self.sections().len() <= 1
            && self
                .sections()
                .iter()
                .all(|section| self.source().sections().contains(section))
    }
    /// Explicit requested disclosures.
    pub fn sections(&self) -> &[DrugSection] {
        &self.fields.section
    }
}

impl DrugGet {
    /// Continue the same named section without changing source, identity or section selectors.
    pub fn at_section_offset(&self, offset: u32) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section_offset = Some(offset);
        self.with_fields(fields)
    }
    /// Exact selected native label heading.
    pub fn label_section(&self) -> Option<&Text> {
        self.fields.label_section.as_ref()
    }
    /// Exact application product selector.
    pub fn product_number(&self) -> Option<&Text> {
        self.fields.product_number.as_ref()
    }
}

impl DrugGet {
    /// Select one named disclosure without changing the exact source identity.
    pub fn for_section(&self, section: DrugSection) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section = vec![section].into();
        self.with_fields(fields)
    }
}

impl DrugGet {
    /// Select an explicit native label heading for the same source identity.
    pub fn for_label(&self, heading: Text) -> Result<Self, QueryInvalid> {
        let mut fields = self.fields.clone();
        fields.section = vec![DrugSection::Label].into();
        fields.label_section = Some(heading);
        self.with_fields(fields)
    }
}

fn selected_source<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<DrugSource>, D::Error> {
    DrugSource::deserialize(d).map(Some)
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for DrugGet {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DrugGet".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = GetFields::json_schema(generator);
        schema.insert(
            "description".into(),
            "A source-selected drug request echoed with a native ID after name resolution.".into(),
        );
        if let Some(properties) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
            properties.insert(
                "input".into(),
                generator.subschema_for::<DrugId>().to_value(),
            );
            properties.insert(
                "source".into(),
                generator.subschema_for::<DrugSource>().to_value(),
            );
        }
        if let Some(required) = schema.get_mut("required").and_then(|r| r.as_array_mut()) {
            required.push("source".into());
        }
        schema
    }
}
