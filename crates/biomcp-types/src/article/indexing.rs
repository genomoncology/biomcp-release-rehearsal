use crate::{Capped, Text};
use serde::{Deserialize, Serialize};
/// Index terms and author identities supplied by this article's EFetch citation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Indexing {
    /// MeSH headings in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub mesh: Box<[MeshHeading]>,
    /// Author keywords in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub keywords: Box<[Text]>,
    /// First ten author affiliations and the complete count.
    pub affiliations: Capped<Text>,
    /// Author ORCID identifiers, as supplied by the source.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub orcids: Box<[Text]>,
}
/// A MeSH descriptor and its qualifiers retain their independent major-topic flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeshHeading {
    /// Source descriptor.
    pub descriptor: Text,
    /// Descriptor major-topic flag.
    pub major: bool,
    /// Source qualifiers.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub qualifiers: Box<[MeshQualifier]>,
}
/// A qualifier attached to a MeSH descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeshQualifier {
    /// Source qualifier.
    pub name: Text,
    /// Qualifier major-topic flag.
    pub major: bool,
}
