//! Native identity discovery follows NCBI's ClinVar E-utilities search contract.
//! https://www.ncbi.nlm.nih.gov/clinvar/docs/programmatic_access/
use super::SPEC;
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::{ClinVarVariationId, Variant, VariantId};
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{Source, SourceRecord, VariantHeader, VariantSearch};
use serde::Deserialize;
const SEARCH: Endpoint = SPEC.get("/esearch.fcgi").fresh(Freshness::Days(1));
/// One native ClinVar candidate page, without inferred transcripts or alleles.
#[derive(Debug)]
pub struct SearchPage {
    /// Native total before paging.
    pub total: u64,
    /// Native positions consumed, including refused records.
    pub consumed: u32,
    /// Source identities in their original order.
    pub rows: Mapped<Vec<VariantHeader>>,
}
#[derive(Deserialize)]
struct Search {
    esearchresult: Page,
}
#[derive(Deserialize)]
struct Page {
    #[serde(deserialize_with = "count")]
    count: u64,
    retstart: String,
    idlist: Vec<Field<Id>>,
    #[serde(rename = "ERROR")]
    error: Option<serde::de::IgnoredAny>,
    errorlist: Option<serde::de::IgnoredAny>,
}
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Id {
    identity: ClinVarVariationId,
    text: biodata::Text,
}
impl TryFrom<String> for Id {
    type Error = biodata::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let text = format!("clinvar:{value}");
        Ok(Self {
            identity: text
                .parse()
                .map_err(|_| biodata::Error::ClinVarVariationIdInvalid)?,
            text: biodata::Text::new(&text)?,
        })
    }
}
impl Reply for Search {
    fn check(&self) -> Result<(), BadReply> {
        let page = &self.esearchresult;
        let total = page.count;
        let offset = page.retstart.parse::<u32>().map_err(|_| BadReply)?;
        let ids: std::collections::BTreeSet<_> = page
            .idlist
            .iter()
            .filter_map(|id| id.as_ref().map(|id| id.identity.get()))
            .collect();
        if page.error.is_some()
            || page.errorlist.is_some()
            || page.idlist.len() > 50
            || ids.len()
                != page
                    .idlist
                    .iter()
                    .filter(|id| id.as_ref().is_some())
                    .count()
            || page.idlist.len() as u64 > total.saturating_sub(u64::from(offset))
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// Search supplied gene and variant notation without choosing a reference sequence.
pub fn search(leg: &Leg<'_>, query: &VariantSearch) -> Result<SearchPage, RequestError> {
    let term = query.source_query();
    let mut args = vec![
        ("db", Arg::Text("clinvar")),
        ("term", Arg::Text(&term)),
        ("retmode", Arg::Text("json")),
        ("retmax", Arg::Num(u64::from(query.limit()))),
        ("tool", Arg::Text("biomcp")),
    ];
    if query.offset() != 0 {
        args.push(("retstart", Arg::Num(u64::from(query.offset()))));
    }
    let reply = leg
        .call(
            &SEARCH,
            Input::Params(&args),
            Read::<Search>::json()
                .check(&|reply| {
                    let page = &reply.esearchresult;
                    let total = page.count;
                    if page.retstart != query.offset().to_string()
                        || page.idlist.len() as u64
                            != total
                                .saturating_sub(u64::from(query.offset()))
                                .min(u64::from(query.limit()))
                    {
                        return Err(BadReply);
                    }
                    Ok(())
                })
                .cache_if(&|reply| {
                    reply
                        .esearchresult
                        .idlist
                        .iter()
                        .all(|id| id.as_ref().is_some())
                }),
        )?
        .value;
    Ok(mapped_page(reply.esearchresult))
}

fn mapped_page(page: Page) -> SearchPage {
    let consumed = page.idlist.len() as u32;
    let mut problems = Problems {
        source: Source::ClinVar,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let rows = page
        .idlist
        .into_iter()
        .filter_map(|id| {
            let Some(id) = problems.required("idlist", id.value()) else {
                refusals = crate::Refusal::Refused;
                return None;
            };
            let identity = VariantId::ClinVarVariation(id.identity);
            let record = SourceRecord {
                namespace: biodata::IdentifierNamespace::Unknown,
                id: id.text,
                url: None,
            };
            Some(VariantHeader {
                identity: Variant {
                    ids: Box::new([identity]),
                    representations: Box::default(),
                    genes: Box::default(),
                },
                requested: None,
                transcripts: Box::default(),
                source: Source::ClinVar,
                record,
                myvariant: None,
                sections: None,
                component_terms: Box::default(),
            })
        })
        .collect();
    SearchPage {
        total: page.count,
        consumed,
        rows: Mapped {
            refusals,
            value: rows,
            problems: problems.list,
        },
    }
}

fn count<'de, D: serde::Deserializer<'de>>(decoder: D) -> Result<u64, D::Error> {
    String::deserialize(decoder)?
        .parse()
        .map_err(|_| serde::de::Error::custom("invalid count"))
}
