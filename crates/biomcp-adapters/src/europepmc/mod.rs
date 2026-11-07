//! Europe PMC's bounded metadata-only search window.
use crate::native_text::NativeText;
use biomcp_fetch::{Input, Read};
mod core;
use crate::{
    mapped::{Problems, record},
    quotas,
    raw::Field,
};
use biodata::{Doi, EuropePmcId, PartialDate, Pmcid, Pmid};

use biomcp_fetch::{
    Arg, BadReply, Endpoint, Freshness, KeyUse, Leg, Reply, RequestError, SourceSpec,
};
use biomcp_types::ArticleHeader;
use biomcp_types::{ArticleItem, ArticleKey, ArticleSort, Source};
pub use core::{core, read};
use serde::Deserialize;
use std::time::Duration;
/// Public search policy with a conservative local quota.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "Europe PMC copyright; record-specific licenses",
        url: "https://europepmc.org/help#copyright",
        usage: biomcp_types::Use::PerRecord,
    },
    source: Source::EuropePmc,
    base: "https://www.ebi.ac.uk/europepmc/webservices/rest",
    key: KeyUse::None,
    quota: &quotas::EUROPEPMC,
    deadline: Duration::from_secs(15),
};
const SEARCH: Endpoint = SPEC.get("/search").fresh(Freshness::Days(1));
/// Original row positions and metadata, before any duplicate suppression.
#[derive(Debug)]
pub struct Search {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Source-reported whole result count.
    pub count: u64,
    /// The actual fetched rows, including duplicates in their original positions.
    pub items: Vec<ArticleItem>,
    /// Rejected source metadata retained beside the page.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Fetch one metadata page in the requested provider order.
pub fn search(leg: &Leg<'_>, term: &str, sort: ArticleSort) -> Result<Search, RequestError> {
    let mut args = vec![
        ("query", Arg::Text(term)),
        ("format", Arg::Text("json")),
        ("resultType", Arg::Text("lite")),
        ("pageSize", Arg::Num(100)),
    ];
    if sort == ArticleSort::Date {
        args.push(("sort", Arg::Text("FIRST_PDATE_D desc")));
    }
    let raw = leg
        .call(&SEARCH, Input::Params(&args), Read::<Raw>::json())?
        .value;
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let items = raw
        .result_list
        .result
        .into_iter()
        .filter_map(|row| {
            if let Some(row) = row.value() {
                row.item(&mut problems, &mut refusals)
            } else {
                refusals = crate::Refusal::Refused;
                crate::mapped::push_problem(&mut problems, Source::EuropePmc, None, "id");
                None
            }
        })
        .collect();
    Ok(Search {
        refusals,
        count: raw.hit_count,
        items,
        problems,
    })
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Raw {
    hit_count: u64,
    result_list: Rows,
}
#[derive(Deserialize)]
struct Rows {
    result: Vec<Field<Row>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    id: String,
    source: String,
    pmid: Option<Field<Pmid>>,
    pmcid: Option<Field<Pmcid>>,
    doi: Option<Field<Doi>>,
    title: Option<Field<NativeText>>,
    journal_title: Option<Field<NativeText>>,
    first_publication_date: Option<Field<PartialDate>>,
    author_string: Option<Field<NativeText>>,
}
impl Reply for Raw {
    fn check(&self) -> Result<(), BadReply> {
        if self.result_list.result.len() > 100
            || self.result_list.result.len() as u64 > self.hit_count
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl Row {
    fn local_id(&self) -> Result<EuropePmcId, biodata::Error> {
        let id = if self.id.starts_with(&self.source) {
            self.id.clone()
        } else {
            format!("{}{}", self.source, self.id)
        };
        id.parse()
    }
    fn item(
        self,
        problems: &mut Vec<biomcp_types::Problem>,
        refusals: &mut crate::Refusal,
    ) -> Option<ArticleItem> {
        let mut p = Problems {
            source: Source::EuropePmc,
            record: record(&self.id),
            list: Vec::new(),
        };
        let local = p.optional("id", Some(self.local_id()));
        let Some(local) = local else {
            *refusals = crate::Refusal::Refused;
            problems.extend(p.list);
            return None;
        };
        let pmid = p.field("pmid", self.pmid).or_else(|| {
            (self.source == "MED")
                .then(|| p.optional("pmid", Some(self.id.parse())))
                .flatten()
        });
        let pmcid = p.field("pmcid", self.pmcid);
        let doi = p.field("doi", self.doi);
        let key = if let Some(id) = pmid {
            ArticleKey::Pubmed(id)
        } else if let Some(id) = pmcid {
            ArticleKey::Pmc(id)
        } else if let Some(id) = doi.clone() {
            ArticleKey::Doi(id)
        } else {
            ArticleKey::Europepmc(local.clone())
        };
        let title = p.source_field("title", self.title);
        let journal = p.source_field("journal", self.journal_title);
        let date = p.field("date", self.first_publication_date);
        let authors = p.source_field("authors", self.author_string);
        let header = if pmid.is_none() {
            title.map(|title| ArticleHeader {
                pmid: None,
                europepmc_id: Some(local),
                doi,
                pmcid,
                title: Some(title),
                journal,
                date,
                electronic_date: None,
                types: Box::default(),
                types_unknown: true,
                first_author: authors
                    .as_ref()
                    .and_then(|s| s.as_str().split(',').next())
                    .and_then(|s| p.checked_text("first_author", Some(s.trim()))),
                first_author_id: None,
                author_count: authors
                    .as_ref()
                    .map_or(0, |s| s.as_str().split(',').count() as u32),
            })
        } else {
            None
        };
        problems.extend(p.list);
        Some(ArticleItem {
            text_withheld: None,
            context_license: None,
            key: Some(key),
            pmid,
            header,
            cited_by: None,
            intents: None,
            influential: None,
            contexts: None,
            found_by: Default::default(),
            sentence: None,
        })
    }
}

mod query;
pub use query::search_query;

/// Complete source policy and command declaration.
pub(crate) mod decl;
