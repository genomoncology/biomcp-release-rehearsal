pub(super) use super::variant_subject::{member, record, subject};
use super::{
    variant::{bounds, failure},
    variant_claims, variant_raw as raw,
};
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::{Text, VariantRef};
use biomcp_fetch::RequestError;
use biomcp_types::{
    CivicClaimKind, CivicClaimPage, CivicMember, CivicMembership, CivicProfile, CivicStatement,
    Reason, Source, SourceRecord, VariantCivic, VariantGet,
};

pub(super) fn identity(value: Option<Field<u64>>, field: &str, cx: &mut Problems) -> Option<u64> {
    let value = cx.required_field(field, value)?;
    cx.optional(
        field,
        Some(if raw::id(value) { Ok(value) } else { Err(()) }),
    )
}
pub(super) fn page(
    raw: raw::Connection<raw::Profile>,
    query: &VariantGet,
    allele: &str,
) -> Result<Mapped<VariantCivic>, RequestError> {
    let consumed = raw.nodes.len();
    let mut cx = Problems {
        source: Source::Civic,
        record: None,
        list: Vec::new(),
    };
    let mut profiles = Vec::new();
    let mut refusals = crate::Refusal::None;
    for profile in raw.nodes {
        cx.record = None;
        if let Some(profile) = map(profile, query, allele, &mut cx, &mut refusals)? {
            profiles.push(profile);
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    let page = bounds(
        if query.civic_profile().is_some() {
            0
        } else {
            query.section_offset()
        },
        profiles.len(),
        raw.total_count,
    );
    let next = if query.civic_profile().is_none() && raw.page_info.has_next_page {
        Some(
            query.at_section_offset(
                query
                    .section_offset()
                    .checked_add(consumed as u32)
                    .ok_or_else(|| failure(Reason::TooLarge))?,
            ),
        )
    } else {
        None
    };
    Ok(Mapped {
        refusals,
        value: VariantCivic {
            profiles: profiles.into(),
            page,
            next,
        },
        problems: cx.list,
    })
}
fn map(
    raw: raw::Profile,
    query: &VariantGet,
    allele: &str,
    cx: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Result<Option<CivicProfile>, RequestError> {
    let Some(id) = identity(raw.id, "id", cx) else {
        return Ok(None);
    };
    cx.record = crate::mapped::record(id);
    if query
        .civic_profile()
        .is_some_and(|requested| requested != id)
    {
        cx.required::<()>("id", None);
        return Ok(None);
    }
    let start = cx.list.len();
    let label = cx.source_field("name", raw.name);
    let compound = cx.field("isComplex", raw.is_complex);
    let raw_name = cx.field("rawName", raw.raw_name);
    let native_count = raw.variants.as_ref().and_then(Field::as_ref).map(Vec::len);
    let native_members = cx.fields("variants", raw.variants);
    let (members, exact) = checked_members(native_members, allele, cx, refusals);
    let mut context_bad = cx.list.len() != start;
    let complete = native_count == Some(members.len()) && !context_bad;
    let link = cx.field("link", raw.link);
    let Some(record) = record(id, link.as_ref(), cx) else {
        return Ok(None);
    };
    let before_subject = cx.list.len();
    let subject = subject(
        &record,
        label.as_ref(),
        raw_name.as_ref(),
        compound,
        members.iter().collect(),
        complete,
        cx,
    );
    context_bad |= cx.list.len() != before_subject;
    let membership = membership(exact, context_bad, &record, cx);
    let mut evidence_page = claim_page(
        &raw.evidence_items,
        query,
        id,
        CivicClaimKind::Evidence,
        exact,
    )?;
    let mut assertion_page = claim_page(
        &raw.assertions,
        query,
        id,
        CivicClaimKind::Assertions,
        exact,
    )?;
    let mut statements = Vec::new();
    if exact {
        let (claims, evidence_returned) = claims(
            raw.evidence_items.nodes,
            raw.assertions.nodes,
            subject.as_ref(),
            context_bad,
            &record.id,
            cx,
            refusals,
        );
        statements = claims;
        evidence_page.returned = evidence_returned;
        assertion_page.returned = statements.len() as u32 - evidence_returned;
    }
    Ok(Some(CivicProfile {
        record,
        label,
        raw_name,
        subject,
        compound,
        membership,
        members: members.into(),
        evidence_count: raw.evidence_items.total_count,
        assertion_count: raw.assertions.total_count,
        statements: statements.into(),
        evidence_page,
        assertion_page,
    }))
}
fn membership(
    exact: bool,
    context_bad: bool,
    record: &SourceRecord,
    cx: &mut Problems,
) -> CivicMembership {
    if exact {
        return CivicMembership::Exact;
    }
    cx.record = Some(record.id.clone());
    if !context_bad {
        cx.required::<()>("molecular_profile.membership", None);
    }
    CivicMembership::Candidate
}
fn claims(
    evidence: Vec<raw::Evidence>,
    assertions: Vec<raw::Assertion>,
    subject: Option<&VariantRef>,
    context_bad: bool,
    record: &Text,
    cx: &mut Problems,
    refusals: &mut crate::Refusal,
) -> (Vec<CivicStatement>, u32) {
    let mut statements = Vec::new();
    for row in evidence {
        cx.record = Some(record.clone());
        if let Some(row) = variant_claims::evidence(row, subject, context_bad, cx) {
            statements.push(row.claim);
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    let evidence_returned = statements.len() as u32;
    for row in assertions {
        cx.record = Some(record.clone());
        if let Some(row) = variant_claims::assertion(row, subject, context_bad, cx) {
            statements.push(row.claim);
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    (statements, evidence_returned)
}
fn checked_members(
    raw: Box<[raw::Member]>,
    allele: &str,
    cx: &mut Problems,
    refusals: &mut crate::Refusal,
) -> (Vec<CivicMember>, bool) {
    let parent = cx.record.clone();
    let mut members = Vec::new();
    let mut exact = false;
    for value in raw.into_vec() {
        cx.record = parent.clone();
        if let Some((member, caid)) = member(value, cx) {
            exact |= caid.as_ref().is_some_and(|id| id.as_str() == allele);
            members.push(member);
        } else {
            *refusals = crate::Refusal::Refused;
        }
    }
    cx.record = parent;
    (members, exact)
}
fn claim_page<T>(
    raw: &raw::Connection<T>,
    query: &VariantGet,
    profile: u64,
    kind: CivicClaimKind,
    exact: bool,
) -> Result<CivicClaimPage, RequestError> {
    let next =
        if exact && (raw.page_info.has_next_page || raw.nodes.is_empty() && raw.total_count > 0) {
            Some(
                query
                    .with_civic_page(profile, kind, raw.page_info.end_cursor.clone())
                    .map_err(|_| failure(Reason::BadReply))?,
            )
        } else {
            None
        };
    Ok(CivicClaimPage {
        returned: if exact { raw.nodes.len() as u32 } else { 0 },
        total: raw.total_count,
        next,
    })
}
