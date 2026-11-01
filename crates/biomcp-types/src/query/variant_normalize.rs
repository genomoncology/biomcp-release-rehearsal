use crate::{Build, HgvsText, VariantInput, VariantNamespace};
use serde::{Deserialize, Serialize};
/// Send unchanged reference-qualified notation to explicitly named services.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct VariantNormalize(Fields<VariantInput>);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Fields"))]
struct Fields<H = String> {
    /// Explicit service; all preserves independent results without consensus.
    source: NormalizationSource,
    /// Reference-qualified HGVS, or chromosome genomic HGVS with an explicit build for CAR.
    /// Checked for reference and syntax only; no full HGVS parser.
    #[cfg_attr(feature = "schema", schemars(schema_with = "hgvs_schema"))]
    hgvs: H,
    /// Caller assembly; required when VariantValidator is selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<Build>,
    /// Source mapping rows per service, with typed continuation for remaining rows.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Source mapping rows to skip per service.
    #[serde(default)]
    offset: u32,
}
fn limit() -> u32 {
    10
}
crate::query::validation::checked_query!(VariantNormalize, Fields);
impl TryFrom<Fields> for VariantNormalize {
    type Error = super::gene::QueryInvalid;
    fn try_from(f: Fields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&f.limit) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        let hgvs =
            VariantInput::new(&f.hgvs).map_err(|_| super::gene::QueryInvalid::new(&["hgvs"]))?;
        if f.hgvs.len() > 2048
            || !(hgvs.hgvs().is_some()
                || f.source == NormalizationSource::Car
                    && f.build.is_some()
                    && hgvs.namespace() == VariantNamespace::Chromosome)
        {
            return Err(super::gene::QueryInvalid::new(&["hgvs"]));
        }
        if f.build
            .is_some_and(|build| !Self::supports_build(hgvs.hgvs(), build))
        {
            return Err(super::gene::QueryInvalid::new(&["hgvs", "build"]));
        }
        if matches!(
            f.source,
            NormalizationSource::All | NormalizationSource::Variantvalidator
        ) && f.build.is_none()
        {
            return Err(super::gene::QueryInvalid::new(&["source", "build"]));
        }
        if f.build
            .is_some_and(|b| !matches!(b, Build::GRCh37 | Build::GRCh38))
        {
            return Err(super::gene::QueryInvalid::new(&["build"]));
        }
        Ok(Self(Fields {
            source: f.source,
            hgvs,
            build: f.build,
            limit: f.limit,
            offset: f.offset,
        }))
    }
}
#[cfg(feature = "schema")]
fn hgvs_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "string",
        "pattern": format!("({}|{})", HgvsText::pattern(), VariantInput::CHROMOSOME_PATTERN)
    })
}
impl VariantNormalize {
    /// Refuse only a provable primary-reference conflict; never assign an unknown accession.
    pub fn supports_build(input: Option<&HgvsText>, build: Build) -> bool {
        let Some(accession) = input.and_then(|value| value.accession().ok()) else {
            return true;
        };
        let other = if build == Build::GRCh37 {
            Build::GRCh38
        } else {
            Build::GRCh37
        };
        biodata::Chromosome::from_accession(accession, build).is_ok()
            || biodata::Chromosome::from_accession(accession, other).is_err()
    }

    /// Selected service set.
    pub fn source(&self) -> NormalizationSource {
        self.0.source
    }
    /// Unchanged caller description.
    pub fn hgvs(&self) -> &str {
        self.0.hgvs.as_str()
    }
    /// Explicit caller assembly.
    pub fn build(&self) -> Option<Build> {
        self.0.build
    }
    /// Mapping page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Mapping page offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for VariantNormalize {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

biodata::wire_enum! {
    /// Explicit normalization service selection.
    NormalizationSource {
        /// ClinGen Allele Registry.
        Car => "car",
        /// Mutalyzer.
        Mutalyzer => "mutalyzer",
        /// VariantValidator.
        Variantvalidator => "variantvalidator",
        /// Every service independently, without a consensus.
        All => "all",
    }
}
