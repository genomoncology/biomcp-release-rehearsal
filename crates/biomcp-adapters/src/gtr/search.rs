use super::{
    bundle, map,
    raw::{Native, Object, Relation},
};
use biodata::GtrTestId;
use biomcp_fetch::{BundleInfo, Find, KeyKind, Leg, RequestError};
use biomcp_types::{DiagnosticCandidate, DiagnosticSearch};
use std::collections::{HashMap, HashSet};
/// One bounded source page and its complete distinct-test count.
#[derive(Debug)]
pub struct Search {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected optional source prose on retained candidates.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native test candidates in relation report order.
    pub items: Box<[DiagnosticCandidate]>,
    /// Distinct exact accession/version identities, including orphan relations and conflicts.
    pub total: u64,
    /// Exact local source snapshot.
    pub info: BundleInfo,
}
/// Match only complete native relation fields, retaining the source relation that matched.
pub fn search(leg: &Leg<'_>, query: &DiagnosticSearch) -> Result<Search, RequestError> {
    let key;
    let find = if let Some(symbol) = query.gene() {
        Find::Exact(KeyKind::Symbol, symbol.as_str())
    } else if let Some(id) = query.gene_id() {
        key = format!("gene:{id}");
        Find::Exact(KeyKind::Xref, &key)
    } else if let Some(id) = query.condition_id() {
        Find::Exact(KeyKind::Xref, id.as_str())
    } else if let Some(kind) = query.test_type() {
        Find::Exact(KeyKind::Alias, kind.as_str())
    } else {
        Find::Exact(KeyKind::Band, "relations")
    };
    let (rows, info) = bundle::read(leg, find)?;
    super::revision(query.bundle_revision(), &info)?;
    let mut seen = HashSet::new();
    let mut matched = Vec::new();
    let mut cx = crate::mapped::Problems {
        source: biomcp_types::Source::Gtr,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    for row in rows {
        if let Native::Refused(r) = &row.value
            && r.matches(query)
        {
            refusals = crate::Refusal::Refused;
            r.disclose(&mut cx);
        }
        if let Native::Relation(r) = row.value
            && let Some(relation) = map::relation(&r, &mut cx, &mut refusals)
            && matches(&r, query)
            && seen.insert(r.id.clone())
        {
            matched.push((r, relation));
        }
    }
    let total = matched.len() as u64;
    let page: Vec<_> = matched
        .into_iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
        .collect();
    let ids: Vec<_> = page.iter().map(|(r, _)| r.id.to_string()).collect();
    let keys: Vec<_> = ids.iter().map(String::as_str).collect();
    let (versions, other) = bundle::read(leg, Find::AnyExact(KeyKind::Id, &keys))?;
    if other.content_revision != info.content_revision {
        return Err(biomcp_fetch::bad_reply(biomcp_types::Source::Gtr, None));
    }
    let mut by_id: HashMap<GtrTestId, Vec<_>> = HashMap::new();
    for row in versions {
        if let Native::Version(v) = row.value {
            by_id
                .entry(v.test_accession_ver.clone())
                .or_default()
                .push(v);
        }
    }
    let mut problems = cx.list;
    let items = page
        .iter()
        .map(|(r, relation)| {
            candidate(
                r,
                relation.clone(),
                by_id.get(&r.id).map(Vec::as_slice).unwrap_or_default(),
                &mut problems,
            )
        })
        .collect::<Result<Box<[_]>, _>>()?;
    Ok(Search {
        refusals,
        items,
        total,
        info,
        problems,
    })
}
fn matches(r: &Relation, q: &DiagnosticSearch) -> bool {
    q.test_type()
        .is_none_or(|v| r.test_type.as_ref().and_then(crate::raw::Field::as_ref) == Some(&v))
        && q.gene()
            .is_none_or(|v| r.object == Object::Gene && &*r.gene_symbol == v.as_str())
        && q.gene_id()
            .is_none_or(|v| r.object == Object::Gene && *r.object_id == v.to_string())
        && q.condition_id().is_none_or(|v| {
            r.object == Object::Condition && format!("MedGen:{}", r.medgen) == v.as_str()
        })
}
fn candidate(
    r: &Relation,
    relation: biodata::GtrTestRelation,
    versions: &[Box<biodata::GtrTestVersion>],
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<DiagnosticCandidate, RequestError> {
    Ok(match versions {
        [v] if v.now_current.as_ref() == "1" => DiagnosticCandidate::Resolved {
            header: map::header(v, problems)?,
            matched: Some(relation),
        },
        [] | [_] => DiagnosticCandidate::MissingCard {
            id: r.id.clone(),
            matched: relation,
        },
        _ => DiagnosticCandidate::Conflict {
            id: r.id.clone(),
            problem: super::conflict(&r.id),
        },
    })
}
