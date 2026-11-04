use crate::{Capped, Doi, EuropePmcId, PartialDate, Pmcid, Pmid, PublicationType, Text};
use serde::{Deserialize, Serialize};
// Keep the wire fields and checked entity in one declaration.
macro_rules! article {
    ($( $(#[$attribute:meta])* pub $field:ident: $ty:ty, )*) => {
        /// Article facts with at least one checked identity.
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "ArticleFields<A>")]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "schema", schemars(!try_from,
            extend("anyOf" = [
                {"required":["pmid"]},
                {"required":["pmcid"]},
                {"required":["doi"]},
                {"required":["europepmc_id"]}
            ])))]
        pub struct Article<A = Box<[Text]>> {
            $( $(#[$attribute])* pub $field: $ty, )*
        }
        #[derive(Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
                struct ArticleFields<A> {
            $( $(#[$attribute])* $field: $ty, )*
        }
        impl<A> TryFrom<ArticleFields<A>> for Article<A> {
            type Error = biodata::Error;
            fn try_from(value: ArticleFields<A>) -> Result<Self, Self::Error> {
                if value.pmid.is_none() && value.pmcid.is_none() && value.doi.is_none() && value.europepmc_id.is_none() {
                    return Err(biodata::Error::ArticleIdentityRequired);
                }
                Ok(Self { $( $field: value.$field, )* })
            }
        }
    };
}
article! {
    /// Europe PMC identity when this card comes from its core record.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "EuropePmcId"))]
    pub europepmc_id: Option<EuropePmcId>,
    /// PubMed identity.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Pmid"))]
    pub pmid: Option<Pmid>,
    /// DOI assigned to this article.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Doi"))]
    pub doi: Option<Doi>,
    /// PMC identity assigned to this article.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Pmcid"))]
    pub pmcid: Option<Pmcid>,
    /// Whole title with inline text preserved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub title: Option<Text>,
    /// Journal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<Text>,
    /// Publication date at source precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<PartialDate>,
    /// Electronic publication date when different from the publication date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electronic_date: Option<PartialDate>,
    /// Every clinically meaningful source publication type.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub types: Box<[PublicationType]>,
    /// Related correction notices or originals.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub corrections: Box<[Correction]>,
    /// Authors in source order.
    pub authors: A,
    /// Whole abstract, preserving labels and source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub abstract_parts: Box<[AbstractPart]>,
}
impl Article {
    /// Disclose the first ten authors and retain the whole count.
    pub fn card(self) -> Article<Capped<Text>> {
        Article {
            europepmc_id: self.europepmc_id,
            pmid: self.pmid,
            doi: self.doi,
            pmcid: self.pmcid,
            title: self.title,
            journal: self.journal,
            date: self.date,
            electronic_date: self.electronic_date,
            types: self.types,
            corrections: self.corrections,
            authors: Capped {
                total: self.authors.len() as u64,
                items: self.authors.into_iter().take(10).collect(),
            },
            abstract_parts: self.abstract_parts,
        }
    }
}
/// One source-labelled abstract paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AbstractPart {
    /// Source label, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Whole paragraph text.
    pub text: crate::Markdown,
}
biodata::canonical_enum! {
    /// Source correction relationship kind.
    CorrectionKind, SourceUnknown {
        /// Retraction relationship.
        Retraction => "retraction",
        /// Erratum relationship.
        Erratum => "erratum",
        /// Expression of concern relationship.
        ExpressionOfConcern => "expression-of-concern",
    }
}
biodata::canonical_enum! {
    /// Direction of a source correction relationship.
    CorrectionRelation, SourceUnknown {
        /// The related record is the notice.
        Notice => "notice",
        /// The related record is the original.
        Original => "original",
    }
}
/// One correction relationship supplied by PubMed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Correction {
    /// Correction category.
    pub kind: CorrectionKind,
    /// Whether the related record is a notice or original.
    pub relation: CorrectionRelation,
    /// Related PMID, when supplied.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Pmid"))]
    pub pmid: Option<Pmid>,
}

fn present<'de, T: Deserialize<'de>, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}
