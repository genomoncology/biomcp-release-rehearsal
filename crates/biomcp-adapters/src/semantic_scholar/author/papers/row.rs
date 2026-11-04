use crate::{
    mapped::{Problems, record},
    native_text::NativeText,
    raw::Field,
};
use biodata::{
    AuthorByline, AuthorPaperAccess, AuthorPaperDetail, AuthorPaperExternalId, AuthorPaperHeader,
    PartialDate, SemanticScholarId, Text,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// Checked paper metadata and problems persist together in source snapshots.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "Input")]
pub struct Paper {
    /// Actual refused native records, independent of optional fields.
    #[serde(with = "crate::mapped::refusal_serde")]
    pub refusals: crate::Refusal,
    detail: AuthorPaperDetail,
    problems: Vec<biomcp_types::Problem>,
}
// Accept the previous native snapshot shape as well as the mapped shape. No
// invalid provider value is serialized as a sentinel or silently lost on reuse.
#[derive(Deserialize)]
#[serde(untagged)]
enum Input {
    Cached {
        #[serde(
            default = "crate::mapped::refusal_serde::none",
            with = "crate::mapped::refusal_serde"
        )]
        refusals: crate::Refusal,
        detail: AuthorPaperDetail,
        problems: Vec<biomcp_types::Problem>,
    },
    Native(Raw),
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Raw {
    paper_id: SemanticScholarId,
    external_ids: Option<Field<BTreeMap<String, Option<Field<NativeId>>>>>,
    title: Option<Field<NativeText>>,
    year: Option<Field<u32>>,
    venue: Option<Field<String>>,
    #[serde(rename = "abstract")]
    abstract_text: Option<Field<NativeText>>,
    publication_date: Option<Field<PartialDate>>,
    reference_count: Option<Field<u64>>,
    citation_count: Option<Field<u64>>,
    influential_citation_count: Option<Field<u64>>,
    is_open_access: Option<Field<bool>>,
    open_access_pdf: Option<Field<Access>>,
    fields_of_study: Option<Field<Vec<Field<NativeText>>>>,
    publication_types: Option<Field<Vec<Field<Text>>>>,
    authors: Option<Field<Vec<Field<Byline>>>>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum NativeId {
    Text(String),
    Number(u64),
}
impl std::fmt::Display for NativeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(v) => v.fmt(f),
            Self::Number(v) => v.fmt(f),
        }
    }
}
#[derive(Deserialize)]
struct Byline {
    #[serde(rename = "authorId")]
    id: Option<Field<biodata::SemanticScholarAuthorId>>,
    name: Option<Field<NativeText>>,
}
#[derive(Deserialize)]
struct Access {
    url: Option<Field<String>>,
    status: Option<Field<Text>>,
    license: Option<Field<Text>>,
    disclaimer: Option<Field<NativeText>>,
}
impl Access {
    fn checked(self, p: &mut Problems) -> AuthorPaperAccess {
        let url = p
            .field("open_access_pdf.url", self.url)
            .and_then(|s| p.optional("open_access_pdf.url", Some(Text::new(&s))));
        AuthorPaperAccess {
            url,
            status: p.field("open_access_pdf.status", self.status),
            license: p.field("open_access_pdf.license", self.license),
            disclaimer: p.source_field("open_access_pdf.disclaimer", self.disclaimer),
        }
    }
}
impl From<Input> for Paper {
    fn from(input: Input) -> Self {
        match input {
            Input::Cached {
                detail,
                problems,
                refusals,
            } => Self {
                detail,
                problems,
                refusals,
            },
            Input::Native(raw) => raw.checked(),
        }
    }
}
impl Raw {
    fn header(&mut self, p: &mut Problems) -> AuthorPaperHeader {
        let venue = p
            .field("venue", self.venue.take())
            .filter(|v| !v.is_empty())
            .and_then(|v| p.text("venue", Some(&v)));
        let mut header = AuthorPaperHeader {
            paper_id: self.paper_id.clone(),
            pmid: None,
            pmcid: None,
            doi: None,
            external_ids: Box::default(),
            title: p.source_field("title", self.title.take()),
            year: p.field("year", self.year.take()),
            venue,
        };
        let mut external = Vec::new();
        for (namespace, value) in p
            .field("external_ids", self.external_ids.take())
            .into_iter()
            .flatten()
        {
            let Some(value) = p.field("external_ids", value) else {
                continue;
            };
            let text = value.to_string();
            match namespace.as_str() {
                "PubMed" => header.pmid = p.optional("pmid", Some(text.parse())),
                "DOI" => header.doi = p.optional("doi", Some(text.parse())),
                "PubMedCentral" => {
                    let id = if text.starts_with("PMC") {
                        text
                    } else {
                        format!("PMC{text}")
                    };
                    header.pmcid = p.optional("pmcid", Some(id.parse()));
                }
                _ => {
                    if let (Some(id), Some(namespace)) = (
                        p.optional("external_ids", Some(format!("{namespace}:{text}").parse())),
                        p.optional(
                            "external_ids.namespace",
                            Some(crate::mapped::text(&namespace)),
                        ),
                    ) {
                        external.push(AuthorPaperExternalId { namespace, id });
                    }
                }
            }
        }
        header.external_ids = external.into();
        header
    }

    fn checked(mut self) -> Paper {
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: record(&self.paper_id),
            list: Vec::new(),
        };
        let header = self.header(&mut p);
        let mut refusals = crate::Refusal::None;
        let fields_of_study = p
            .field("fields_of_study", self.fields_of_study)
            .map(|rows| {
                rows.into_iter()
                    .filter_map(|r| p.source_field("fields_of_study", Some(r)))
                    .collect()
            });
        let publication_types = p
            .field("publication_types", self.publication_types)
            .map(|rows| {
                rows.into_iter()
                    .filter_map(|r| p.field("publication_types", Some(r)))
                    .collect()
            });
        let authors = p.field("authors", self.authors).map(|rows| {
            rows.into_iter()
                .filter_map(|r| {
                    let r = p.record("authors", r, &mut refusals)?;
                    Some(AuthorByline {
                        id: p.field("authors.id", r.id),
                        name: p.source_field("authors.name", r.name),
                    })
                })
                .collect()
        });
        let open_access_pdf = p
            .field("open_access_pdf", self.open_access_pdf)
            .map(|v| v.checked(&mut p));
        let detail = AuthorPaperDetail {
            header,
            abstract_text: p.source_field("abstract_text", self.abstract_text),
            publication_date: p.field("publication_date", self.publication_date),
            reference_count: p.field("reference_count", self.reference_count),
            citation_count: p.field("citation_count", self.citation_count),
            influential_citation_count: p.field(
                "influential_citation_count",
                self.influential_citation_count,
            ),
            is_open_access: p.field("is_open_access", self.is_open_access),
            open_access_pdf,
            fields_of_study,
            publication_types,
            authors,
        };
        Paper {
            refusals,
            detail,
            problems: p.list,
        }
    }
}
impl Paper {
    /// Return source metadata and its retained problems without repeating admission.
    pub fn detail(
        self,
        problems: &mut Vec<biomcp_types::Problem>,
    ) -> Result<AuthorPaperDetail, biomcp_fetch::BadReply> {
        problems.extend(self.problems);
        Ok(self.detail)
    }
}
