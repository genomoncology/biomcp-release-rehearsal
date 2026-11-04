//! Exact LitVar2 candidate selection and source-ordered publication identifiers.
use crate::quotas;
use biodata::{Pmid, Text};
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, PathSegment, RequestError, SourceSpec};
use biomcp_types::{LitVarCandidate, Source};
mod raw;
use raw::{Candidates, Publications};
use std::time::Duration;
/// Public NCBI LitVar2 policy, independent of E-utilities credentials.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "NCBI public-domain metadata policy",
        url: "https://www.ncbi.nlm.nih.gov/home/about/policies/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::LitVar,
    base: "https://www.ncbi.nlm.nih.gov/research/litvar2-api",
    key: KeyUse::None,
    quota: &quotas::LITVAR,
    deadline: Duration::from_secs(15),
};
pub(crate) const AUTOCOMPLETE: Endpoint =
    SPEC.get("/variant/autocomplete/").fresh(Freshness::Days(1));
const PUBLICATIONS: Endpoint = SPEC.get("/variant/get").fresh(Freshness::Days(1));
/// Exact source identity retained between autocomplete and publications.
#[derive(Debug)]
pub struct Variant {
    id: String,
    term: Text,
}
impl Variant {
    /// Source rsID, or its variant name when it has no rsID.
    pub fn term(&self) -> &Text {
        &self.term
    }
}
/// Result of exact autocomplete selection, before any publication search.
#[derive(Debug)]
pub enum Selection {
    /// Exactly one source variant matched.
    Exact(Variant),
    /// No publications call may be made.
    Candidates(Vec<LitVarCandidate>),
}
/// One selected variant's whole source count and bounded native sequence.
#[derive(Debug)]
pub struct Search {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Selected source rsID or variant name.
    pub term: Text,
    /// Whole source publication count.
    pub count: Option<u64>,
    /// Native publication sequence capped at the pool limit.
    pub ids: Vec<Pmid>,
    /// Rejected native fields and publication identities.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Select by exact case-insensitive gene/change or rsID, never by hit position.
pub fn select(
    leg: &Leg<'_>,
    gene: Option<&str>,
    variant: &str,
) -> Result<crate::Mapped<Selection>, RequestError> {
    let rsid = is_rsid(variant);
    let change = variant
        .get(..2)
        .filter(|s| s.eq_ignore_ascii_case("p."))
        .map_or(variant, |_| variant.get(2..).unwrap_or(variant));
    let term = if rsid {
        variant.to_owned()
    } else {
        format!("{} {change}", gene.unwrap_or_default())
    };
    let candidates = leg
        .call(
            &AUTOCOMPLETE,
            Input::Params(&[("query", Arg::Text(&term))]),
            Read::<Candidates>::json().cache_if(&Candidates::eligible),
        )
        .map(|received| received.value)?
        .mapped();
    let mut matching = candidates.value.iter().filter_map(|row| {
        let matches = if rsid {
            row.rsid
                .as_ref()
                .is_some_and(|rsid| rsid.as_str().eq_ignore_ascii_case(variant))
        } else {
            gene.is_some_and(|gene| {
                row.gene
                    .iter()
                    .any(|g| g.as_str().eq_ignore_ascii_case(gene))
            }) && row
                .name
                .as_ref()
                .is_some_and(|name| name.as_str().eq_ignore_ascii_case(&format!("p.{change}")))
        };
        if !matches {
            return None;
        }
        row.rsid
            .as_ref()
            .or(row.name.as_ref())
            .map(|term| (row, term))
    });
    let Some((selected, term)) = matching.next().filter(|_| matching.next().is_none()) else {
        return Ok(crate::Mapped {
            refusals: candidates.refusals,
            value: Selection::Candidates(
                candidates
                    .value
                    .into_iter()
                    .take(5)
                    .map(Candidate::public)
                    .collect(),
            ),
            problems: candidates.problems,
        });
    };
    Ok(crate::Mapped {
        refusals: candidates.refusals,
        value: Selection::Exact(Variant {
            id: selected.id.clone(),
            term: term.clone(),
        }),
        problems: candidates.problems,
    })
}
/// Fetch publications only after exact source identity selection.
pub fn publications(leg: &Leg<'_>, selected: Variant) -> Result<Search, RequestError> {
    let segments = [&selected.id, "publications"]
        .into_iter()
        .map(PathSegment::new)
        .collect::<Result<Vec<_>, _>>()?;
    let reply = leg
        .call(
            &PUBLICATIONS.clone().with_segments(&segments),
            Input::Params(&[]),
            Read::<Publications>::json().cache_if(&Publications::eligible),
        )?
        .value;
    let mut problems = crate::mapped::Problems {
        source: Source::LitVar,
        record: crate::mapped::record(&selected.id),
        list: Vec::new(),
    };
    let count = problems
        .field("pmids_count", reply.pmids_count)
        .and_then(|count| {
            if count < reply.pmids.len() as u64 {
                problems.required("pmids_count", None)
            } else {
                Some(count)
            }
        });
    let mut refusals = crate::Refusal::None;
    let ids = reply
        .pmids
        .into_iter()
        .take(9_999)
        .filter_map(|raw| {
            let id = problems.record("pmids", raw, &mut refusals)?;
            match problems.optional("pmids", Some(Pmid::new(id))) {
                Some(id) => Some(id),
                None => {
                    refusals = crate::Refusal::Refused;
                    None
                }
            }
        })
        .collect();
    Ok(Search {
        refusals,
        term: selected.term,
        count,
        ids,
        problems: problems.list,
    })
}
struct Candidate {
    id: String,
    rsid: Option<Text>,
    gene: Vec<Text>,
    name: Option<biodata::Text>,
    pmids_count: Option<u64>,
}
impl Candidate {
    fn public(self) -> LitVarCandidate {
        LitVarCandidate {
            rsid: self.rsid,
            gene: self.gene,
            name: self.name,
            count: self.pmids_count,
        }
    }
}
/// Recognize the exact rsID spelling accepted by variant lookup.
pub fn is_rsid(value: &str) -> bool {
    value
        .get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("rs"))
        && value.get(2..).is_some_and(|digits| {
            !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
