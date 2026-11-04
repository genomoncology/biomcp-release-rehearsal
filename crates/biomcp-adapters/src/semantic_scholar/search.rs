use crate::native_text::NativeText;
use crate::{
    mapped::{Problems, record},
    raw::Field,
};
use biodata::{Doi, PartialDate, Pmcid, Pmid, SemanticScholarId};
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, Reply, RequestError};
use biomcp_types::{ArticleHeader, ArticleItem, ArticleKey};
use serde::Deserialize;
mod receive;
const SEARCH: Endpoint = super::SPEC.get("/paper/search").fresh(Freshness::Days(1));
/// A native search window, before duplicate suppression or DOI resolution.
#[derive(Debug, Deserialize)]
#[serde(from = "RawSearch")]
pub struct Search {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Whole source-reported result count.
    pub total: u64,
    /// Native result order.
    pub data: Vec<Paper>,
    /// Rejected paper identities without invented fallback IDs.
    pub problems: Vec<biomcp_types::Problem>,
    received: usize,
}
#[derive(Deserialize)]
struct RawSearch {
    total: u64,
    data: Vec<Field<Paper>>,
}
/// Common paper metadata shared by search and graph expansion.
#[derive(Debug, Deserialize)]
#[serde(from = "RawPaper")]
pub struct Paper {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Stable source identity, never a title-derived fallback.
    pub paper_id: Option<SemanticScholarId>,
    /// Identifiers explicitly reported by the source.
    pub external_ids: Option<ExternalIds>,
    /// Whole title when supplied.
    pub title: Option<NativeText>,
    /// Source venue.
    pub venue: Option<String>,
    /// Publication year when full date is absent.
    pub year: Option<u16>,
    /// Publication date at source precision.
    pub publication_date: Option<PartialDate>,
    /// Authors in source order.
    pub authors: Option<Vec<Author>>,
    problems: Vec<biomcp_types::Problem>,
}
/// External identities, preserving absence independently of paper identity.
#[derive(Debug)]
pub struct ExternalIds {
    /// PubMed identity.
    pub pubmed: Option<Pmid>,
    /// PMC identity.
    pub pmc: Option<Pmcid>,
    /// DOI spelling reported by the source.
    pub doi: Option<Doi>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPaper {
    paper_id: Option<SemanticScholarId>,
    external_ids: Option<Field<RawIds>>,
    title: Option<Field<NativeText>>,
    venue: Option<Field<String>>,
    year: Option<Field<u16>>,
    publication_date: Option<Field<PartialDate>>,
    authors: Option<Field<Vec<Field<RawAuthor>>>>,
}
#[derive(Debug, Deserialize)]
struct RawIds {
    #[serde(rename = "PubMed")]
    pubmed: Option<Field<Pmid>>,
    #[serde(rename = "PubMedCentral")]
    pmc: Option<Field<String>>,
    #[serde(rename = "DOI")]
    doi: Option<Field<Doi>>,
}
#[derive(Debug, Deserialize)]
struct RawAuthor {
    #[serde(rename = "authorId")]
    id: Option<Field<biodata::SemanticScholarAuthorId>>,
    name: Option<Field<NativeText>>,
}
impl From<RawPaper> for Paper {
    fn from(raw: RawPaper) -> Self {
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: raw.paper_id.as_ref().and_then(record),
            list: Vec::new(),
        };
        let mut refusals = crate::Refusal::None;
        let external_ids = p.field("external_ids", raw.external_ids).map(|ids| {
            let pmc = p.field("pmcid", ids.pmc).and_then(|id| {
                let id = if id.starts_with("PMC") {
                    id
                } else {
                    format!("PMC{id}")
                };
                p.optional("pmcid", Some(id.parse()))
            });
            ExternalIds {
                pubmed: p.field("pmid", ids.pubmed),
                pmc,
                doi: p.field("doi", ids.doi),
            }
        });
        let authors = p.field("authors", raw.authors).map(|rows| {
            rows.into_iter()
                .filter_map(|r| {
                    let r = p.record("authors", r, &mut refusals)?;
                    Some(Author {
                        id: p.field("authors.id", r.id),
                        name: p.source_field("authors.name", r.name),
                    })
                })
                .collect()
        });
        Self {
            refusals,
            paper_id: raw.paper_id,
            external_ids,
            title: p.field("title", raw.title),
            venue: p.field("venue", raw.venue),
            year: p.field("year", raw.year),
            publication_date: p.field("date", raw.publication_date),
            authors,
            problems: p.list,
        }
    }
}
/// One source author.
#[derive(Debug)]
pub struct Author {
    /// Exact native author identity when this reply supplies one.
    pub id: Option<biodata::SemanticScholarAuthorId>,
    /// Source display name.
    pub name: Option<biodata::Text>,
}
impl Reply for Search {
    fn discoveries(&self) -> Option<Vec<biomcp_fetch::Identities>> {
        if self.refusals != crate::Refusal::None {
            return None;
        }
        paper_discoveries(self.data.iter())
    }
    fn check(&self) -> Result<(), BadReply> {
        if self.received > 100 || self.received as u64 > self.total {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// Fetch the first relevance-ordered page using only checked mappings.
pub fn search(
    leg: &Leg<'_>,
    term: &str,
    year: Option<&str>,
    open_access: bool,
) -> Result<Search, RequestError> {
    let mut args = vec![
        ("query", Arg::Text(term)),
        ("limit", Arg::Num(100)),
        (
            "fields",
            Arg::Text("externalIds,title,venue,year,publicationDate,authors"),
        ),
    ];
    if let Some(year) = year {
        args.push(("year", Arg::Text(year)));
    }
    if open_access {
        args.push(("openAccessPdf", Arg::Text("")));
    }
    leg.call(&SEARCH, Input::Params(&args), Read::json())
        .map(|received| received.value)
}
impl Paper {
    /// Map identifiers and source metadata without joining another provider.
    pub fn item(self, problems: &mut Vec<biomcp_types::Problem>) -> ArticleItem {
        problems.extend(self.problems);
        let record = self.paper_id.as_ref().and_then(crate::mapped::record);
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: record.clone(),
            list: Vec::new(),
        };
        let date = self.publication_date.or_else(|| {
            self.year
                .and_then(|v| p.optional("date", Some(v.to_string().parse())))
        });
        problems.extend(p.list);
        let ids = self.external_ids;
        let pmid = ids.as_ref().and_then(|v| v.pubmed);
        let pmcid = ids.as_ref().and_then(|v| v.pmc);
        let doi = ids.and_then(|v| v.doi);
        let key = if let Some(id) = pmid {
            Some(ArticleKey::Pubmed(id))
        } else if let Some(id) = pmcid {
            Some(ArticleKey::Pmc(id))
        } else if let Some(id) = doi.clone() {
            Some(ArticleKey::Doi(id))
        } else {
            self.paper_id.map(ArticleKey::SemanticScholar)
        };
        let header = self.title.map(|title| ArticleHeader {
            pmid,
            pmcid,
            doi,
            europepmc_id: None,
            title: Some(title.into()),
            journal: crate::mapped::field_text(
                problems,
                biomcp_types::Source::SemanticScholar,
                record,
                "venue",
                self.venue
                    .filter(|v| !v.is_empty())
                    .map(crate::raw::Field::Value),
            ),
            electronic_date: None,
            date,
            types: Box::default(),
            types_unknown: true,
            first_author: self
                .authors
                .as_ref()
                .and_then(|a| a.first())
                .and_then(|a| a.name.clone()),
            first_author_id: self
                .authors
                .as_ref()
                .and_then(|a| a.first())
                .and_then(|a| a.id.clone()),
            author_count: self.authors.as_ref().map_or(0, |a| a.len() as u32),
        });
        ArticleItem {
            key,
            pmid,
            header,
            cited_by: None,
            sentence: None,
            intents: None,
            influential: None,
            contexts: None,
            found_by: Default::default(),
        }
    }
}
pub(super) fn paper_discoveries<'a>(
    papers: impl Iterator<Item = &'a Paper>,
) -> Option<Vec<biomcp_fetch::Identities>> {
    let mut pmids = Vec::new();
    let mut dois = Vec::new();
    for paper in papers {
        if paper.refusals != crate::Refusal::None || !paper.problems.is_empty() {
            return None;
        }
        if let Some(ids) = &paper.external_ids {
            if let Some(pmid) = &ids.pubmed {
                pmids.push(pmid.to_string());
            } else if let Some(doi) = &ids.doi {
                dois.push(doi.to_string());
            }
        }
    }
    Some(vec![
        biomcp_fetch::Identities::bounded("pmid", pmids)?,
        biomcp_fetch::Identities::bounded("doi", dois)?,
    ])
}
