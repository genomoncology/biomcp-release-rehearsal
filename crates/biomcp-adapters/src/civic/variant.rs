//! Molecular-profile lookups use source allele identities, never name-derived equivalence.
use super::{GENES, variant_map as map, variant_raw as raw};
use crate::Mapped;
use biodata::{Text, VariantId};
use biomcp_fetch::Input;

use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{
    CivicClaimKind, PageInfo, Reason, Source, VariantCivic, VariantGet, VariantHeader,
};
use serde::Serialize;

#[derive(Serialize)]
struct Query<'a> {
    query: String,
    variables: Variables<'a>,
}
#[derive(Serialize)]
struct Variables<'a> {
    allele: &'a str,
    after: Option<&'a Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<[u64; 1]>,
    evidence_after: Option<&'a Text>,
    assertion_after: Option<&'a Text>,
}
/// Read one bounded profile page or a selected native evidence/assertion cursor page.
pub fn variant(
    leg: &Leg<'_>,
    query: &VariantGet,
    candidate: &VariantHeader,
) -> Result<Mapped<VariantCivic>, RequestError> {
    let allele = candidate
        .identity
        .ids
        .iter()
        .chain(candidate.requested.iter())
        .find(|id| matches!(id, VariantId::Ca(_)))
        .map(ToString::to_string)
        .ok_or_else(|| failure(Reason::NoJoinId))?;
    let exact_query = query
        .for_candidate(allele.parse().map_err(|_| failure(Reason::BadReply))?)
        .map_err(|_| failure(Reason::BadReply))?;
    let query = &exact_query;
    let mut after = None;
    let mut offset = 0;
    let mut partial = false;
    loop {
        let body = request(query, &allele, after.as_ref());
        let reply: raw::Profiles = leg
            .call(
                &GENES,
                Input::json(&[], &body),
                biomcp_fetch::Read::json().cache_if(&|page: &raw::Profiles| {
                    !page.has_errors()
                        && page.matches(&allele, query.civic_profile())
                        && page.progresses(query, after.as_ref())
                }),
            )?
            .value;
        if !reply.progresses(query, after.as_ref()) {
            return Err(failure(Reason::BadReply));
        }
        partial |= reply.has_errors();
        let connection = reply.data.molecular_profiles;
        if query.civic_profile().is_some()
            || offset >= query.section_offset()
            || !connection.page_info.has_next_page
        {
            let mut mapped = map::page(connection, query, &allele)?;
            if partial {
                mapped.problems.push(biomcp_types::Problem {
                    source: Source::Civic,
                    record: None,
                    field: "graphql.errors".into(),
                    code: biomcp_types::ProblemCode::ValueUnknown,
                });
            }
            return Ok(mapped);
        }
        if connection.nodes.is_empty() || connection.page_info.end_cursor == after {
            return Err(failure(Reason::BadReply));
        }
        offset += connection.nodes.len() as u32;
        after = connection.page_info.end_cursor;
    }
}
fn request<'a>(query: &'a VariantGet, allele: &'a str, after: Option<&'a Text>) -> Query<'a> {
    let limit = query.section_limit();
    let (evidence, assertions) = match query.civic_kind() {
        Some(CivicClaimKind::Evidence) => (limit, 0),
        Some(CivicClaimKind::Assertions) => (0, limit),
        None => (limit.min(2), 1),
    };
    let (ids_variable, ids_argument) = if query.civic_profile().is_some() {
        (",$ids:[Int!]", ",ids:$ids")
    } else {
        ("", "")
    };
    Query {
        query: format!(
            "query($allele:String!,$after:String{ids_variable},$evidence_after:String,\
                    $assertion_after:String){{molecularProfiles(alleleRegistryId:$allele{ids_argument},first:1,after:$after){{totalCount \
                pageInfo{{hasNextPage endCursor}} nodes{{id name rawName link isComplex variants{{id name link \
                feature{{id name}} ... on GeneVariant{{alleleRegistryId hgvsDescriptions}}}} \
                evidenceItems(first:{evidence},after:$evidence_after,includeRejected:false){{totalCount \
                pageInfo{{hasNextPage endCursor}} nodes{{id status evidenceType evidenceLevel evidenceDirection \
                significance disease{{name doid}} therapies{{name}} therapyInteractionType source{{citationId sourceType}}}}}} \
                assertions(first:{assertions},after:$assertion_after,includeRejected:false){{totalCount \
                pageInfo{{hasNextPage endCursor}} nodes{{id status assertionType assertionDirection significance \
                ampLevel disease{{name doid}} therapies{{name}} therapyInteractionType variantOrigin}}}}}}}}}}"
        ),
        variables: Variables {
            allele,
            after,
            ids: query.civic_profile().map(|id| [id]),
            evidence_after: if query.civic_kind() == Some(CivicClaimKind::Evidence) {
                query.civic_cursor()
            } else {
                None
            },
            assertion_after: if query.civic_kind() == Some(CivicClaimKind::Assertions) {
                query.civic_cursor()
            } else {
                None
            },
        },
    }
}
pub(super) fn bounds(offset: u32, returned: usize, total: u64) -> PageInfo {
    PageInfo {
        offset,
        returned: returned as u32,
        total,
        pool: None,
        truncated: None,
    }
}
pub(super) fn failure(reason: Reason) -> RequestError {
    biomcp_fetch::SourceError {
        source: Source::Civic,
        reason,
        status: None,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
