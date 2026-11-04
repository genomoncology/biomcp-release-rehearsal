use super::{ComplexPortalId, Position1, QueryInvalid, Text, UniProtIsoform, limit, present};
use serde::{Deserialize, Serialize};
macro_rules! relation {
    ($name:ident, $fields:ident, $wire:literal, { $($(#[$meta:meta])* $field:ident: $ty:ty),* }, $doc:literal $(, $check:expr)?) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = $wire)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "schema", schemars(with = $wire))]
        pub struct $name($fields);
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        struct $fields {
            /// Exact source protein sequence identity.
            #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
            accession: UniProtIsoform,
            $($(#[$meta])* $field: $ty,)*
            /// Five rows by default, at most 25.
            #[serde(default = "limit")]
            #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
            limit: u32,
            /// Native or retained page offset.
            #[serde(default)]
            offset: u32,
            /// Retained collection identity.
            #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "schema", schemars(with = "Text"))]
            pool: Option<Text>,
        }
        crate::query::validation::checked_query!($name, $fields);
        impl TryFrom<$fields> for $name {
            type Error = QueryInvalid;
            fn try_from(v: $fields) -> Result<Self, QueryInvalid> {
                if !(1..=25).contains(&v.limit) { return Err(QueryInvalid::new(&["limit"])); }
                $(if !($check)(&v) { return Err(QueryInvalid::new(&["position", "accession"])); })?
                Ok(Self(v))
            }
        }
        impl $name {
            /// Continue a retained source collection without changing selectors.
            pub fn at_page(&self, pool: Text, offset: u32) -> Self {
                let mut q = self.clone(); q.0.pool = Some(pool); q.0.offset = offset; q
            }
            /// Exact anchor accession.
            pub fn accession(&self) -> &UniProtIsoform { &self.0.accession }
            /// Bounded page size.
            pub fn limit(&self) -> u32 { self.0.limit }
            /// Page offset.
            pub fn offset(&self) -> u32 { self.0.offset }
            /// Retained collection, if selected.
            pub fn pool(&self) -> Option<&Text> { self.0.pool.as_ref() }
        }
impl crate::Paged for $name {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) { self.0.offset = 0; }
    fn clear_page_context(&mut self) { self.0.pool = None; }
}

    };
}
relation!(ProteinFeaturesQuery, FeatureFields, "FeatureFields", {
    /// Native one-based residue; zero fails during deserialization.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = u32::MAX)))]
    position: Option<Position1>,
    /// Exact UniProt native label, without an alias map.
    #[serde(default, rename = "type", deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    feature_type: Option<Text>
}, "Page five native UniProt features by exact type and displayed residue. No variant effect or clinical meaning \
        is inferred; no bioinformatics tool runs.",
|v: &FeatureFields| v.position.is_none() || !v.accession.as_str().contains('-'));
relation!(ProteinDomainsQuery, DomainFields, "DomainFields", {
    /// Exact native InterPro type, not a normalized domain label.
    #[serde(default, rename = "type", deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    entry_type: Option<Text>
}, "Page five native InterPro matches with original types and fragments. No structure or clinical meaning is \
        inferred; no bioinformatics tool runs.");
relation!(
    ProteinRelationQuery,
    RelationFields,
    "RelationFields",
    {},
    "Page five source isoforms, exact-participant complexes or PDB links. Source identities stay \
        separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs."
);
relation!(ProteinComplexMembersQuery, MemberFields, "MemberFields", {
    /// Exact complex whose native participants contain the anchor.
    complex: ComplexPortalId
}, "Page five native participants of an exact Complex Portal complex containing the anchor. No \
    interaction or clinical meaning is inferred; no bioinformatics tool runs.");
impl ProteinFeaturesQuery {
    /// Exact native residue filter.
    pub fn position(&self) -> Option<Position1> {
        self.0.position
    }
    /// Native UniProt feature label.
    pub fn feature_type(&self) -> Option<&Text> {
        self.0.feature_type.as_ref()
    }
}
impl ProteinDomainsQuery {
    /// Native InterPro entry type.
    pub fn entry_type(&self) -> Option<&Text> {
        self.0.entry_type.as_ref()
    }
}
impl ProteinComplexMembersQuery {
    /// Exact containing complex.
    pub fn complex(&self) -> &ComplexPortalId {
        &self.0.complex
    }
}
/// Read one complete native sequence for an exact source accession.
/// No variant effect, structure or clinical meaning is inferred; no bioinformatics tool runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProteinSequenceQuery {
    /// Exact isoform or base accession; no canonical suffix is synthesized.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    pub isoform: UniProtIsoform,
}

impl ProteinDomainsQuery {
    /// Build one bounded first page of native accession matches.
    pub fn new(accession: UniProtIsoform, limit: u32) -> Result<Self, QueryInvalid> {
        DomainFields {
            accession,
            entry_type: None,
            limit,
            offset: 0,
            pool: None,
        }
        .try_into()
    }
}
impl ProteinRelationQuery {
    /// Build a bounded unfiltered accession relation page.
    pub fn new(accession: UniProtIsoform, limit: u32) -> Result<Self, QueryInvalid> {
        RelationFields {
            accession,
            limit,
            offset: 0,
            pool: None,
        }
        .try_into()
    }
}
impl ProteinComplexMembersQuery {
    /// Build the first five participants of one exact containing complex.
    pub fn new(accession: UniProtIsoform, complex: ComplexPortalId) -> Self {
        Self(MemberFields {
            accession,
            complex,
            limit: limit(),
            offset: 0,
            pool: None,
        })
    }
}

crate::query::validation::checked_query!(ProteinSequenceQuery);
