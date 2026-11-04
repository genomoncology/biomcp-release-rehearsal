//! Native publication studies; association totals use bounded count requests.
use super::{SPEC, get::study::Study, raw, search::identity};
use biomcp_fetch::{Arg, BadReply, Freshness, Input, Leg, Reply, RequestError};
use biomcp_types::{GwasSearch, GwasStudyRow};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Checked study rows and source diagnostics.
#[derive(Debug)]
pub struct Studies {
    /// Native study total.
    pub total: u64,
    /// Native received slots, including rejected identities.
    pub received: u32,
    /// Source metadata with optional association totals.
    pub items: Vec<GwasStudyRow>,
    /// Refused source metadata.
    pub refusals: crate::Refusal,
    /// Rejected optional fields and count failures.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "_embedded")]
    embedded: Option<Embedded>,
    #[serde(rename = "_links")]
    links: raw::Links,
    page: raw::Page,
}
#[derive(Debug, Deserialize)]
struct Embedded {
    studies: Vec<Study>,
}
impl Reply for Response {
    fn check(&self) -> Result<(), BadReply> {
        let rows = self
            .embedded
            .as_ref()
            .map(|e| e.studies.as_slice())
            .unwrap_or_default();
        let expected = self
            .page
            .total_elements
            .saturating_sub(u64::from(self.page.number) * u64::from(self.page.size))
            .min(self.page.size.into());
        let mut ids = std::collections::BTreeSet::new();
        if self.page.size == 0
            || rows.len() as u64 != expected
            || rows.iter().any(|s| !ids.insert(s.accession_id.to_string()))
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// One native study page plus paced, bounded association count requests.
pub fn studies(leg: &Leg<'_>, q: &GwasSearch) -> Result<Studies, RequestError> {
    let values = BTreeMap::from([
        (
            "pubmed_id".to_string(),
            q.pubmed().map(ToString::to_string).unwrap_or_default(),
        ),
        ("page".into(), (q.offset() / q.limit()).to_string()),
        ("size".into(), q.limit().to_string()),
    ]);
    let params: Vec<_> = values
        .iter()
        .map(|(k, v)| (k.as_str(), Arg::Text(v)))
        .collect();
    let endpoint = SPEC.get("/studies").fresh(Freshness::Days(1));
    let response = leg
        .call(
            &endpoint,
            Input::Params(&params),
            biomcp_fetch::Read::<Response>::json().check(&|r| {
                if r.page.number != q.offset() / q.limit()
                    || r.page.size != q.limit()
                    || !identity::resource_link(r.links.current.href.as_str(), "studies", &values)
                    || r.embedded.as_ref().is_some_and(|e| {
                        e.studies.iter().any(|s| {
                            s.pubmed_id.map(|id| id.to_string())
                                != q.pubmed().map(ToString::to_string)
                        })
                    })
                {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )?
        .value;
    let rows = response.embedded.map(|e| e.studies).unwrap_or_default();
    let received = rows.len() as u32;
    let mut out = Studies {
        total: response.page.total_elements,
        received,
        items: Vec::new(),
        refusals: crate::Refusal::None,
        problems: Vec::new(),
    };
    for study in rows.into_iter().skip((q.offset() % q.limit()) as usize) {
        let mapped = study.card();
        out.refusals.include(mapped.refusals);
        out.problems.extend(mapped.problems);
        out.items.push(GwasStudyRow {
            study: mapped.value,
            association_count: None,
        });
    }
    let counts = leg.batch(&out.items, |child, row| {
        let query = GwasSearch::study_count(row.study.id.clone());
        super::search(child, &query).map(|page| page.total)
    });
    for (row, count) in out.items.iter_mut().zip(counts) {
        match count {
            Ok(total) => row.association_count = Some(total),
            Err(_) => {
                out.problems.push(biomcp_types::Problem {
                    source: biomcp_types::Source::Gwas,
                    record: Some(row.study.id.to_string().parse().map_err(|_| {
                        biomcp_fetch::bad_reply(biomcp_types::Source::Gwas, Some(200))
                    })?),
                    field: "association_count".into(),
                    code: biomcp_types::ProblemCode::ValueUnknown,
                })
            }
        }
    }
    Ok(out)
}
