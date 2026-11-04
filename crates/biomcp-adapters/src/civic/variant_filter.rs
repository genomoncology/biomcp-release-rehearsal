//! Filtered profile evidence uses CIViC's root collection, whose arguments support filters.
use super::{GENES, evidence_filter, variant::failure, variant_raw as raw};
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{BadReply, Input, Leg, Read, Reply, RequestError};
use biomcp_types::{CivicClaimKind, Reason, VariantGet};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Request<'a> {
    query: String,
    variables: Variables<'a>,
}
#[derive(Serialize)]
struct Variables<'a> {
    id: u64,
    after: Option<&'a Text>,
}
#[derive(Deserialize)]
struct Response {
    data: Data,
    errors: Option<Vec<serde::de::IgnoredAny>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    evidence_items: raw::Connection<raw::Evidence>,
}
impl Reply for Response {
    fn check(&self) -> Result<(), BadReply> {
        if self
            .errors
            .as_ref()
            .is_some_and(|errors| !errors.is_empty())
        {
            return Err(BadReply);
        }
        self.data.evidence_items.check()
    }
}
pub(super) fn apply(
    leg: &Leg<'_>,
    query: &VariantGet,
    profiles: &mut raw::Connection<raw::Profile>,
) -> Result<(), RequestError> {
    for profile in &mut profiles.nodes {
        let id = profile
            .id
            .as_ref()
            .and_then(Field::as_ref)
            .copied()
            .filter(|id| raw::id(*id))
            .ok_or_else(|| failure(Reason::BadReply))?;
        let count = match query.civic_kind() {
            Some(CivicClaimKind::Evidence) => query.section_limit(),
            Some(CivicClaimKind::Assertions) => 0,
            None => query.section_limit().min(2),
        };
        profile.evidence_items = evidence_filter::page(
            query.evidence_filter(),
            if query.civic_kind() == Some(CivicClaimKind::Evidence) {
                query.civic_cursor()
            } else {
                None
            },
            count,
            |level, after, count| {
                let arguments = evidence_filter::arguments(
                    query.evidence_filter(),
                    level,
                    "status:NON_REJECTED",
                );
                let body = Request {
                    query: format!(
                        "query($id:Int!,$after:String){{evidenceItems(molecularProfileId:$id,{arguments},first:{count},after:$after){{totalCount \
                        pageInfo{{hasNextPage endCursor}} nodes{{id status evidenceType evidenceLevel evidenceDirection \
                        significance disease{{name doid}} therapies{{name}} therapyInteractionType source{{citationId sourceType}}}}}}}}"
                    ),
                    variables: Variables { id, after },
                };
                let reply: Response = leg
                    .call(
                        &GENES,
                        Input::json(&[], &body),
                        Read::json().cache_if(&|reply: &Response| {
                            count == 0 || reply.data.evidence_items.progresses(after)
                        }),
                    )?
                    .value;
                let rows = reply.data.evidence_items;
                if !rows.progresses(after) && count > 0 {
                    return Err(failure(Reason::BadReply));
                }
                Ok(rows)
            },
        )?;
    }
    Ok(())
}
