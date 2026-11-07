use super::assertion_id::AssertionId;
use crate::{Text, VariantInput, VariantNamespace};
use serde::{Deserialize, Serialize};
/// Page assertions from one explicitly selected evidence source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct VariantAssertionsGet(Fields<AssertionId>);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "I: Deserialize<'de>"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Fields"))]
struct Fields<I = Text> {
    /// Supported variant ID; resolves one ClinVar Variation ID or ERepo CAid through the shared resolver.
    #[cfg_attr(feature="schema",schemars(extend("x-positional"=0)))]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "super::variant_input::LookupInput")
    )]
    input: VariantInput,
    /// One native assertion source; shared identity conversion precedes source access when needed.
    source: AssertionSource,
    /// Identity lookup assembly; omitted uses the shared GRCh38 default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<crate::Build>,
    /// Native assertion accession or document ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    assertion: Option<I>,
    /// VCV archive or ERepo document version; RCV/SCV selectors match a returned native version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<Text>,
    /// Assertions per page; omitted uses the native source default.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 50)))]
    limit: Option<u32>,
    /// Assertions to skip in source order.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(VariantAssertionsGet, Fields);
impl TryFrom<Fields> for VariantAssertionsGet {
    type Error = super::gene::QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        crate::VariantGet::new(
            fields.input.clone(),
            crate::VariantResolver::Car,
            fields.build,
        )?;
        if fields.input.namespace() == VariantNamespace::Unsupported {
            return Err(super::gene::QueryInvalid::new(&["input"]));
        }
        if fields.limit.is_some_and(|n| !(1..=50).contains(&n)) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        if fields.version.is_some() && fields.assertion.is_none() {
            return Err(super::gene::QueryInvalid::new(&["version", "assertion"]));
        }
        let assertion = fields
            .assertion
            .as_ref()
            .map(|id| AssertionId::parse(id, fields.source))
            .transpose()?;
        if let Some(version) = &fields.version {
            let parts = version.as_str().split('.').collect::<Vec<_>>();
            let count = match fields.source {
                AssertionSource::Clinvar => 1,
                AssertionSource::Erepo => 3,
            };
            if parts.len() != count {
                return Err(super::gene::QueryInvalid::new(&["version", "source"]));
            }
            if parts.iter().any(|s| {
                s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || s.parse::<u32>().is_err()
            }) {
                return Err(super::gene::QueryInvalid::new(&["version", "source"]));
            }
            if parts
                .first()
                .is_some_and(|s| s.parse::<u32>().is_ok_and(|n| n == 0))
            {
                return Err(super::gene::QueryInvalid::new(&["version", "source"]));
            }
        }
        Ok(Self(Fields {
            input: fields.input,
            source: fields.source,
            build: fields.build,
            assertion,
            version: fields.version,
            limit: fields.limit,
            offset: fields.offset,
        }))
    }
}
impl VariantAssertionsGet {
    /// Checked source-specific identity.
    pub fn input(&self) -> &VariantInput {
        &self.0.input
    }
    /// Keep assertion and page selectors after shared identity resolution.
    pub fn with_input(&self, input: VariantInput) -> Self {
        let mut query = self.clone();
        query.0.input = input;
        query
    }
    /// Caller assembly for identity resolution.
    pub fn build(&self) -> Option<crate::Build> {
        self.0.build
    }
    /// Explicit evidence source.
    pub fn source(&self) -> AssertionSource {
        self.0.source
    }
    /// Native record selection.
    pub fn assertion(&self) -> Option<&AssertionId> {
        self.0.assertion.as_ref()
    }
    /// Native version selection.
    pub fn version(&self) -> Option<&Text> {
        self.0.version.as_ref()
    }
    /// Selected page size.
    pub fn limit(&self) -> u32 {
        self.0
            .limit
            .unwrap_or_else(|| self.0.source.default_limit())
    }
    /// Source-order page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }

    /// Construct a checked source query without selecting an assertion.
    pub fn new(
        input: VariantInput,
        source: AssertionSource,
    ) -> Result<Self, super::gene::QueryInvalid> {
        Fields {
            input,
            source,
            build: None,
            assertion: None,
            version: None,
            limit: None,
            offset: 0,
        }
        .try_into()
    }
    /// Select the assertion provider and page from a parsed candidate identity.
    pub fn from_identity(
        id: crate::VariantId,
        limit: u32,
        offset: u32,
    ) -> Result<Self, crate::QueryInvalid> {
        let source = match &id {
            crate::VariantId::ClinVarVariation(_) | crate::VariantId::Vcv(_) => {
                AssertionSource::Clinvar
            }
            crate::VariantId::Ca(_) => AssertionSource::Erepo,
            _ => return Err(crate::QueryInvalid::new(&["input", "source"])),
        };
        if !(1..=50).contains(&limit) {
            return Err(crate::QueryInvalid::new(&["limit"]));
        }
        let input =
            VariantInput::from_identity(id).map_err(|_| crate::QueryInvalid::new(&["input"]))?;
        Ok(Self(Fields {
            input,
            source,
            build: None,
            assertion: None,
            version: None,
            limit: Some(limit),
            offset,
        }))
    }
    /// Select an existing native assertion and its exact source version.
    pub fn select(
        &self,
        record: Text,
        version: Option<Text>,
    ) -> Result<Self, super::gene::QueryInvalid> {
        let mut q = self.clone();
        q.0.assertion = Some(AssertionId::parse(&record, self.source())?);
        q.0.version = version;
        q.0.offset = 0;
        Ok(q)
    }
    /// Retain the identity while selecting a page size and offset.
    pub fn page(mut self, limit: u32, offset: u32) -> Result<Self, super::gene::QueryInvalid> {
        self.0.limit = Some(limit);
        self.0.offset = offset;
        if !(1..=50).contains(&limit) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        Ok(self)
    }
}
impl crate::Paged for VariantAssertionsGet {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

biodata::wire_enum! {
    /// Native assertion provider.
    AssertionSource {
        /// NCBI ClinVar.
        Clinvar=>"clinvar",
        /// ClinGen evidence repository.
        Erepo=>"erepo",
    }
}
impl AssertionSource {
    /// Default native assertion page size.
    pub fn default_limit(self) -> u32 {
        match self {
            Self::Clinvar => 10,
            Self::Erepo => 3,
        }
    }

    /// Source policy identity.
    pub fn source(self) -> crate::Source {
        match self {
            Self::Clinvar => crate::Source::ClinVar,
            Self::Erepo => crate::Source::Erepo,
        }
    }
}
