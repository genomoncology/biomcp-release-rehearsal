use crate::{Build, VariantGet, VariantInput, VariantResolver};
use serde::{Deserialize, Serialize};
/// Read a bounded ancestry-detail page from the exact source-given GRCh38 allele.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct VariantPopulationQuery(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Exact published allele identity; ambiguous candidates must first be selected.
    #[cfg_attr(feature="schema",schemars(extend("x-positional"=0)))]
    input: VariantInput,
    /// Resolver for reference-qualified input.
    #[serde(default = "resolver")]
    resolver: VariantResolver,
    /// Explicit source assembly when required by identity input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<Build>,
    /// Read source ancestry groups, preserving overlapping groups individually.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    detail: bool,
    /// Maximum rows per page.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native rows to skip.
    #[serde(default)]
    offset: u32,
}
fn resolver() -> VariantResolver {
    VariantResolver::Car
}
fn limit() -> u32 {
    10
}
crate::query::validation::checked_query!(VariantPopulationQuery, Fields);
impl TryFrom<Fields> for VariantPopulationQuery {
    type Error = super::gene::QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        VariantGet::new(fields.input.clone(), fields.resolver, fields.build)?;
        if !(1..=50).contains(&fields.limit) {
            return Err(super::gene::QueryInvalid::new(&["limit"]));
        }
        Ok(Self(fields))
    }
}
impl VariantPopulationQuery {
    /// Read checked identity selectors without changing their meaning.
    pub fn identity(&self) -> Result<VariantGet, super::gene::QueryInvalid> {
        VariantGet::new(self.0.input.clone(), self.0.resolver, self.0.build)
    }
    /// Select one source candidate while retaining population-detail selectors.
    pub fn with_identity(
        &self,
        input: VariantInput,
        build: Option<Build>,
    ) -> Result<Self, super::gene::QueryInvalid> {
        let mut query = self.clone();
        query.0.input = input;
        query.0.build = build;
        query.0.resolver = VariantResolver::Car;
        query.0.offset = 0;
        Self::try_from(query.0)
    }
    /// Whether source ancestry detail was requested.
    pub fn detail(&self) -> bool {
        self.0.detail
    }
    /// Row bound.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native row offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for VariantPopulationQuery {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}
