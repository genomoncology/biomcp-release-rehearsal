//! Public iCite metadata; no abstracts or provider-specific scoring blend.
use crate::{quotas, raw::Field};
use biodata::{FiniteNumber, Pmid, Text};
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{
    Arg, BadReply, Endpoint, Freshness, KeyUse, Leg, Reply, RequestError, SourceSpec,
};
use biomcp_types::{Citations, Problem, ProblemCode, Source};
use serde::Deserialize;
use std::{collections::BTreeSet, time::Duration};
/// Maximum documented PMID request size.
pub const BATCH_LIMIT: usize = 200;
/// Conservative application quota; no provider rate is claimed.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "NIH public-domain data",
        url: "https://support.icite.nih.gov/hc/en-us/articles/9066501755291-Citations-Module-Overview",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Icite,
    base: "https://icite.od.nih.gov/api",
    key: KeyUse::None,
    quota: &quotas::ICITE,
    deadline: Duration::from_secs(15),
};
const PUBS: Endpoint = SPEC.get("/pubs").fresh(Freshness::Days(1));
const FIELDS: &str = "pmid,pubYear,citedByPmidCount,citedByPmids,citedPmids,citedByClinicalArticle,citingClinicalPmids,rcr,rcrIsProvisional";
/// One checked, independently identified iCite record.
#[derive(Debug)]
pub struct Publication {
    /// Native PubMed identity.
    pub pmid: Pmid,
    /// Optional fields rejected independently of the retained paper.
    pub problems: Vec<Problem>,
    /// Counts with absent lists distinguished from confirmed empty lists.
    pub counts: Citations,
    /// Distinct citing paper IDs when the source supplied the list.
    pub citing: Option<Vec<Pmid>>,
    /// Distinct referenced paper IDs when the source supplied the list.
    pub references: Option<Vec<Pmid>>,
}
/// Request one documented metadata batch, joining only requested identities.
pub fn publications(
    leg: &Leg<'_>,
    ids: &[Pmid],
) -> Result<crate::Mapped<Vec<Publication>>, RequestError> {
    request::<Raw>(leg, ids)
}
/// Request metadata while observing the native reference selection.
pub fn references(
    leg: &Leg<'_>,
    ids: &[Pmid],
) -> Result<crate::Mapped<Vec<Publication>>, RequestError> {
    request::<References>(leg, ids)
}
fn request<T: Reply + Into<Raw> + serde::de::DeserializeOwned>(
    leg: &Leg<'_>,
    ids: &[Pmid],
) -> Result<crate::Mapped<Vec<Publication>>, RequestError> {
    if ids.is_empty() || ids.len() > BATCH_LIMIT {
        return Err(biomcp_fetch::bad_reply(Source::Icite, Some(200)));
    }
    let names: Vec<_> = ids
        .iter()
        .map(|id| id.to_string().into_boxed_str())
        .collect();
    let raw: Raw = leg
        .call(
            &PUBS,
            Input::Params(&[
                ("pmids", Arg::Ids(&names)),
                ("legacy", Arg::Text("false")),
                ("fl", Arg::Text(FIELDS)),
            ]),
            Read::<T>::json(),
        )?
        .value
        .into();
    if raw
        .data
        .iter()
        .filter_map(Field::as_ref)
        .any(|r| !ids.contains(&r.pmid.0))
    {
        return Err(biomcp_fetch::bad_reply(Source::Icite, Some(200)));
    }
    let mut p = crate::mapped::Problems {
        source: Source::Icite,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let value = raw
        .data
        .into_iter()
        .filter_map(|r| match p.required("pmid", r.value()) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .map(Row::map)
        .collect();
    Ok(crate::Mapped {
        refusals,
        value,
        problems: p.list,
    })
}

#[derive(Deserialize)]
#[serde(try_from = "u32")]
struct Id(Pmid);
impl TryFrom<u32> for Id {
    type Error = biodata::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Pmid::new(value)
            .map(Self)
            .map_err(|_| biodata::Error::ArticleIdInvalid)
    }
}
#[derive(Deserialize)]
struct Raw {
    data: Vec<Field<Row>>,
}
impl Raw {
    fn selection(&self, references: bool) -> Option<biomcp_fetch::Identities> {
        let [row] = self.data.as_slice() else {
            return None;
        };
        let row = row.as_ref()?;
        let ids = if references {
            &row.cited_pmids
        } else {
            &row.cited_by_pmids
        };
        let ids = ids.as_ref()?.as_ref()?;
        let mut seen = BTreeSet::new();
        biomcp_fetch::Identities::bounded(
            "pmid",
            ids.iter()
                .map(|id| id.0)
                .filter(|id| seen.insert(*id))
                .map(|id| id.to_string())
                .collect(),
        )
    }
}
#[derive(Deserialize)]
#[serde(transparent)]
struct References(Raw);
impl From<References> for Raw {
    fn from(value: References) -> Self {
        value.0
    }
}
impl Reply for References {
    fn discovery(&self) -> Option<biomcp_fetch::Identities> {
        self.0.selection(true)
    }
    fn input_parameter() -> Option<(&'static str, &'static str)> {
        Raw::input_parameter()
    }
    fn check(&self) -> Result<(), BadReply> {
        self.0.check()
    }
}
impl Reply for Raw {
    fn discovery(&self) -> Option<biomcp_fetch::Identities> {
        self.selection(false)
    }
    fn input_parameter() -> Option<(&'static str, &'static str)> {
        Some(("pmid", "pmids"))
    }
    fn check(&self) -> Result<(), BadReply> {
        let ids: BTreeSet<_> = self
            .data
            .iter()
            .filter_map(Field::as_ref)
            .map(|r| r.pmid.0)
            .collect();
        if self.data.len() > BATCH_LIMIT
            || ids.len() != self.data.iter().filter_map(Field::as_ref).count()
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    pmid: Id,
    cited_by_pmid_count: Option<Field<u64>>,
    cited_by_pmids: Option<Field<Vec<Id>>>,
    cited_pmids: Option<Field<Vec<Id>>>,
    citing_clinical_pmids: Option<Field<Vec<Id>>>,
    rcr: Option<Field<FiniteNumber>>,
    rcr_is_provisional: Option<Field<bool>>,
}
fn distinct(ids: Vec<Id>) -> Vec<Pmid> {
    let mut seen = BTreeSet::new();
    ids.into_iter()
        .map(|id| id.0)
        .filter(|id| seen.insert(*id))
        .collect()
}
impl Row {
    fn map(self) -> Publication {
        let mut problems = Vec::new();
        let id = self.pmid.0;
        let count = field(self.cited_by_pmid_count, id, "cited_by", &mut problems);
        let citing = field(self.cited_by_pmids, id, "cited_by", &mut problems).map(distinct);
        let references = field(self.cited_pmids, id, "references", &mut problems).map(distinct);
        let clinical = field(
            self.citing_clinical_pmids,
            id,
            "cited_by_clinical",
            &mut problems,
        )
        .map(distinct);
        let rcr = field(self.rcr, id, "rcr", &mut problems);
        let provisional = field(self.rcr_is_provisional, id, "rcr", &mut problems);
        Publication {
            pmid: id,
            problems,
            counts: Citations {
                cited_by: citing.as_ref().map(|ids| ids.len() as u64).or(count),
                cited_by_clinical: clinical.map(|ids| ids.len() as u64),
                references: references.as_ref().map(|ids| ids.len() as u64),
                rcr: rcr.filter(|_| provisional == Some(false)),
            },
            citing,
            references,
        }
    }
}

fn field<T>(
    value: Option<Field<T>>,
    id: Pmid,
    name: &str,
    problems: &mut Vec<Problem>,
) -> Option<T> {
    match value {
        Some(Field::Value(value)) => Some(value),
        Some(Field::Invalid(_)) => {
            let problem = Problem {
                source: Source::Icite,
                record: Text::new(&id.to_string()).ok(),
                field: name.into(),
                code: ProblemCode::ValueUnknown,
            };
            if !problems.contains(&problem) {
                problems.push(problem);
            }
            None
        }
        None => None,
    }
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
