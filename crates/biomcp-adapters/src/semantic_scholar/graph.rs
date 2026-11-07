//! Bounded paper sections and native citation graph windows.
use crate::{
    mapped::{Problems, record},
    raw::Field,
};
use biodata::Text;
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, PathSegment, Reply, RequestError};
use biomcp_types::ArticleInput;
use biomcp_types::Capped;
use serde::Deserialize;
mod summary;
const PAPER: Endpoint = super::SPEC.get("/paper").fresh(Freshness::Days(1));
pub use summary::PaperSummary;
/// Bind a checked article identity to the shared graph source without accepting a free URL.
pub fn paper_endpoint(input: &ArticleInput) -> Result<Endpoint, RequestError> {
    let value = super::paper_id(input)?;
    let part = PathSegment::new(&value).map_err(RequestError::Input)?;
    Ok(PAPER.clone().with_segments(&[part]))
}
/// Read the summary under the shared optional-key source declaration.
pub fn summary(leg: &Leg<'_>, input: &ArticleInput) -> Result<PaperSummary, RequestError> {
    leg.call(
        &paper_endpoint(input)?,
        biomcp_fetch::Input::Params(&[(
            "fields",
            Arg::Text("tldr,citationCount,influentialCitationCount"),
        )]),
        biomcp_fetch::Read::json(),
    )
    .map(|received| received.value)
}
/// One source graph page; `next` is a provider offset, never a client guess.
#[derive(Debug)]
pub struct Edges {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Provider continuation offset.
    pub next: Option<u32>,
    /// Native citation order.
    pub data: Vec<Edge>,
    /// Rejected independent edge identities.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Source edge metadata with missing fields preserved independently.
#[derive(Debug, Deserialize)]
#[serde(from = "RawEdge")]
pub struct Edge {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Present in a citation reply.
    pub citing_paper: Option<super::Paper>,
    /// Present in a references reply.
    pub cited_paper: Option<super::Paper>,
    /// Provider intent labels.
    pub intents: Option<Box<[biomcp_types::CitationIntent]>>,
    /// Missing is distinct from false.
    pub is_influential: Option<bool>,
    /// Missing is distinct from a reported empty list.
    pub contexts: Option<Vec<String>>,
    /// Independently resolved license of the citing paper.
    pub context_license: Option<biodata::License>,
    /// Text was removed while retaining the paper and edge metadata.
    pub text_withheld: Option<biomcp_types::WithheldArticleText>,
    /// Rejected edge metadata retained with its paper.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEdge {
    citing_paper: Option<super::Paper>,
    cited_paper: Option<super::Paper>,
    intents: Option<Field<Vec<Field<biomcp_types::CitationIntent>>>>,
    is_influential: Option<Field<bool>>,
    contexts: Option<Field<Vec<Field<String>>>>,
    #[serde(rename = "biomcpContextLicense")]
    context_license: Option<biodata::License>,
    #[serde(rename = "biomcpTextWithheld")]
    text_withheld: Option<biomcp_types::WithheldArticleText>,
}
impl From<RawEdge> for Edge {
    fn from(raw: RawEdge) -> Self {
        let paper = raw.citing_paper.as_ref().or(raw.cited_paper.as_ref());
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: paper.and_then(|r| r.paper_id.as_ref()).and_then(record),
            list: Vec::new(),
        };
        let intents = p.field("intents", raw.intents).map(|values| {
            values
                .into_iter()
                .filter_map(|v| p.field("intents", Some(v)))
                .collect()
        });
        let is_influential = p.field("influential", raw.is_influential);
        let contexts = p.field("contexts", raw.contexts).map(|values| {
            values
                .into_iter()
                .filter_map(|v| p.field("contexts", Some(v)))
                .collect()
        });
        let mut refusals = crate::Refusal::None;
        for paper in [&raw.citing_paper, &raw.cited_paper].into_iter().flatten() {
            refusals.include(paper.refusals);
        }
        Self {
            refusals,
            citing_paper: raw.citing_paper,
            cited_paper: raw.cited_paper,
            intents,
            is_influential,
            contexts,
            context_license: raw.context_license,
            text_withheld: raw.text_withheld,
            problems: p.list,
        }
    }
}
#[derive(Debug, Deserialize)]
struct RawEdges {
    next: Option<u32>,
    #[serde(deserialize_with = "edge_data")]
    data: Option<Vec<Field<Edge>>>,
}
impl Reply for RawEdges {
    fn discoveries(&self) -> Option<Vec<biomcp_fetch::Identities>> {
        let rows = self.data.as_ref()?;
        if rows.iter().any(|row| {
            row.as_ref()
                .is_none_or(|edge| edge.refusals != crate::Refusal::None)
        }) {
            return None;
        }
        super::search::paper_discoveries(
            self.data
                .as_ref()?
                .iter()
                .filter_map(Field::as_ref)
                .filter_map(|edge| edge.citing_paper.as_ref().or(edge.cited_paper.as_ref())),
        )
    }

    fn check(&self) -> Result<(), BadReply> {
        if self.data.as_ref().is_some_and(|data| {
            data.len() > 1000
                || data
                    .iter()
                    .filter_map(Field::as_ref)
                    .any(|e| e.citing_paper.is_some() == e.cited_paper.is_some())
        }) {
            return Err(BadReply);
        }
        Ok(())
    }
}
fn edge_data<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<Vec<Field<Edge>>>, D::Error> {
    Option::<Vec<Field<Edge>>>::deserialize(decoder)
}
impl Edge {
    /// Keep the selected direction's paper and first whole context.
    pub fn item(
        self,
        references: bool,
        problems: &mut Vec<biomcp_types::Problem>,
    ) -> Result<biomcp_types::ArticleEdgeItem, BadReply> {
        problems.extend(self.problems);
        let mut item = if references {
            self.cited_paper
        } else {
            self.citing_paper
        }
        .ok_or(BadReply)?
        .item(problems);
        item.intents = self.intents;
        item.influential = self.is_influential;
        item.context_license = self.context_license;
        item.text_withheld = self.text_withheld.map(|notice| {
            if notice.url.is_some() {
                notice
            } else {
                crate::article_rights::withheld_item(&item)
            }
        });
        item.contexts = self
            .contexts
            .map(|contexts| {
                Ok(Capped {
                    total: contexts.len() as u64,
                    items: contexts
                        .into_iter()
                        .take(1)
                        .filter_map(|text| {
                            let mut p = Problems {
                                source: biomcp_types::Source::SemanticScholar,
                                record: item.pmid.and_then(record),
                                list: Vec::new(),
                            };
                            let value = p.optional("contexts", Some(Text::from_source(&text)));
                            problems.extend(p.list);
                            value
                        })
                        .collect(),
                })
            })
            .transpose()?;
        Ok(item)
    }
}
/// Read at most one thousand edges without sorting the provider's rows.
pub fn edges(
    leg: &Leg<'_>,
    input: &ArticleInput,
    references: bool,
    offset: Option<u32>,
) -> Result<Edges, RequestError> {
    let suffix = PathSegment::new(if references {
        "references"
    } else {
        "citations"
    })
    .map_err(RequestError::Input)?;
    let endpoint = paper_endpoint(input)?.with_segments(&[suffix]);
    let mut args = vec![
        ("limit", Arg::Num(1000)),
        (
            "fields",
            Arg::Text("contexts,intents,isInfluential,externalIds,title,year"),
        ),
    ];
    if let Some(offset) = offset {
        args.push(("offset", Arg::Num(u64::from(offset))));
    }
    let raw = leg
        .call(
            &endpoint,
            biomcp_fetch::Input::Params(&args),
            biomcp_fetch::Read::<RawEdges>::json().filter_body(&|body| {
                let parent = match input {
                    ArticleInput::Pmid(id) => Some(*id),
                    _ => None,
                };
                crate::article_rights::semantic::filter(leg, body, parent)
            }),
        )?
        .value;
    let data = raw.data.ok_or(biomcp_fetch::SourceError {
        source: biomcp_types::Source::SemanticScholar,
        reason: biomcp_types::Reason::Unavailable,
        status: Some(200),
        tls_cause: None,
        retry_after: None,
    })?;
    let mut p = Problems {
        source: biomcp_types::Source::SemanticScholar,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let data = data
        .into_iter()
        .filter_map(|v| {
            let mut row = p.record("paper_id", v, &mut refusals)?;
            if references && let Some(notice) = &mut row.text_withheld {
                notice.url = crate::article_rights::text_withheld(input).url;
            }
            refusals.include(row.refusals);
            Some(row)
        })
        .collect();
    Ok(Edges {
        refusals,
        next: raw.next,
        data,
        problems: p.list,
    })
}
