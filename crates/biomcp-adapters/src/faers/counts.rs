//! Native openFDA reaction counts, independent of report discovery and its totals.
use super::SPEC;
use biodata::PartialDate;
use biomcp_fetch::{Arg, BadReply, Freshness, Input, Leg, Reply, ReplyMetadata, RequestError};
use biomcp_types::{AdverseEventReactionCount, AdverseEventSearch};
use serde::Deserialize;

/// One source count request; report totals and pagination are not available here.
#[derive(Debug)]
pub struct CountPage {
    /// Exact native terms and report counts, in provider order.
    pub items: Vec<AdverseEventReactionCount>,
    /// Publisher dataset date, independent of retrieval time.
    pub updated: Option<PartialDate>,
    /// Original received metadata.
    pub metadata: ReplyMetadata,
    /// The terms' separately admitted terminology policy.
    pub components: Box<[biomcp_types::ComponentLabel]>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response {
    Page {
        meta: Meta,
        results: Vec<AdverseEventReactionCount>,
    },
    Absent {
        error: crate::openfda::Absence,
    },
}
#[derive(Debug, Deserialize)]
struct Meta {
    last_updated: Option<PartialDate>,
}
impl Reply for Response {
    fn data_date(&self) -> Option<String> {
        match self {
            Self::Page { meta, .. } => meta.last_updated.map(|date| date.to_string()),
            Self::Absent { .. } => None,
        }
    }
    fn check(&self) -> Result<(), BadReply> {
        match self {
            Self::Page { .. } => Ok(()),
            Self::Absent { error } if error.valid() => Ok(()),
            _ => Err(BadReply),
        }
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if matches!(
            (self, status),
            (Self::Page { .. }, 200) | (Self::Absent { .. }, 404)
        ) {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}

/// Count exact reaction terms over the matching reports in one native request.
pub fn reactions(leg: &Leg<'_>, query: &AdverseEventSearch) -> Result<CountPage, RequestError> {
    if query.aggregate().is_none() || query.source() != biomcp_types::AdverseEventSource::Faers {
        return Err(biomcp_fetch::InputError::Serialization.into());
    }
    crate::adverse_event_sources::components::meddra(leg)?;
    let components = crate::adverse_event_sources::components::labels(
        biomcp_types::Source::Faers,
        biomcp_types::Source::Meddra,
    )?;
    let selector = query.faers_expression();
    let endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
    let received = leg.call(
        &endpoint,
        Input::Params(&[
            ("search", Arg::Text(&selector)),
            ("count", Arg::Text("patient.reaction.reactionmeddrapt.exact")),
            ("limit", Arg::Num(query.limit().into())),
        ]),
        biomcp_fetch::Read::<Response>::json().check(&|response| {
            if matches!(response, Response::Page { results, .. } if results.len() > query.limit() as usize) {
                return Err(BadReply);
            }
            Ok(())
        }),
    )?;
    let (items, updated) = match received.value {
        Response::Page { meta, results } => (results, meta.last_updated),
        Response::Absent { .. } => (Vec::new(), None),
    };
    Ok(CountPage {
        items,
        updated,
        metadata: received.meta,
        components,
    })
}
