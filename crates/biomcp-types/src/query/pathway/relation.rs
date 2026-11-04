use super::{PathwayId, QueryInvalid, Text, default_limit, native_id, reactome, valid_limit};
use serde::{Deserialize, Serialize};
/// Explicit full physical-context disclosure; omission keeps membership compact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
enum Details {
    Details,
}
macro_rules! fields {
    ($name:ident, $pattern:literal, $($extra:tt)*) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        struct $name {
            /// Checked source-owned human pathway identity.
            #[serde(with = "native_id")]
            #[cfg_attr(feature = "schema", schemars(with = "str", regex(pattern = $pattern), extend("x-positional" = 0)))]
            id: PathwayId,
            /// Five complete relation rows by default, at most fifty.
            #[serde(default = "default_limit")]
            #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
            limit: u32,
            /// Offset within the retained selected relation.
            #[serde(default)]
            offset: u32,
            /// Immutable relation snapshot.
            #[serde(default, deserialize_with = "super::super::gene::present", skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "schema", schemars(with = "Text"))]
            pool: Option<Text>,
            $($extra)*
        }
    };
}
fields!(EventFields, "^R-HSA-[1-9][0-9]*$",);
fields!(GeneFields, "^(R-HSA-|WP)[1-9][0-9]*$",
    /// Include every physical entity context for these references; never clips source text.
    #[serde(default, deserialize_with = "super::super::gene::present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Details", extend("x-positional" = 1)))]
    details: Option<Details>,
);
macro_rules! relation {
    ($name:ident, $fields:ident, $wire:literal, $check:ident, $doc:literal $(, $extra:ident: $default:expr)*) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = $wire)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "schema", schemars(with = $wire))]
        pub struct $name($fields);
        crate::query::validation::checked_query!($name, $fields);
        impl TryFrom<$fields> for $name {
            type Error = QueryInvalid;
            fn try_from(v: $fields) -> Result<Self, Self::Error> {
                if !valid_limit(v.limit) { return Err(QueryInvalid::new(&["limit"])); }
        $check(&v)?;
                Ok(Self(v))
            }
        }
        impl $name {
            /// Start an exact source relation with a checked page size.
            pub fn new(id: PathwayId, limit: u32) -> Result<Self, QueryInvalid> {
                Self::try_from($fields { id, limit, offset: 0, pool: None, $($extra: $default,)* })
            }
            /// Retain the same immutable relation on subsequent pages.
            pub fn with_pool(mut self, pool: Text) -> Self { self.0.pool = Some(pool); self }
            /// Checked source identity owning this relation.
            pub fn id(&self) -> &PathwayId { &self.0.id }
            /// Complete relation rows per page.
            pub fn limit(&self) -> u32 { self.0.limit }
            /// Relation offset, independent of other sections.
            pub fn offset(&self) -> u32 { self.0.offset }
            /// Immutable relation snapshot.
            pub fn pool(&self) -> Option<&Text> { self.0.pool.as_ref() }
        }
impl crate::Paged for $name {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) { self.0.offset = offset; }
    fn clear_page_context(&mut self) { self.0.pool = None; }
}

    };
}
relation!(PathwayGenesQuery, GeneFields, "GeneFields", valid_genes,
    "Read Reactome exact references or WikiPathways GeneProduct data nodes. The details layer discloses \
        complete Reactome physical contexts. Membership names a source and release; it implies no therapy, \
        response or actionability. No enrichment is calculated.", details: None);
relation!(
    PathwayEventsQuery,
    EventFields,
    "EventFields",
    valid_events,
    "Read Reactome event objects and unresolved native integer references in source order. Membership \
        names a source and release; it implies no therapy, response or actionability. No enrichment is \
        calculated."
);
impl PathwayGenesQuery {
    /// Request all physical contexts where the native source provides them.
    pub fn with_details(mut self) -> Self {
        if reactome(&self.0.id) {
            self.0.details = Some(Details::Details);
        }
        self
    }
    /// Whether full physical contexts were explicitly requested.
    pub fn details(&self) -> bool {
        self.0.details.is_some()
    }
}

fn valid_genes(v: &GeneFields) -> Result<(), QueryInvalid> {
    if !reactome(&v.id) && v.details.is_some() {
        return Err(QueryInvalid::new(&["id", "details"]));
    }
    Ok(())
}
fn valid_events(v: &EventFields) -> Result<(), QueryInvalid> {
    if !reactome(&v.id) {
        return Err(QueryInvalid::new(&["id"]));
    }
    Ok(())
}
