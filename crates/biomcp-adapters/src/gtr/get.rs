use super::{bundle, map, raw::Native};
use biodata::{GtrMethod, GtrTestRelation};
use biomcp_fetch::{BundleInfo, Find, KeyKind, Leg, RequestError};
use biomcp_types::{DiagnosticGet, GtrCard, Problem};
/// Exact source identity and independent relation collections under one local snapshot.
#[derive(Debug)]
pub struct Record {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected optional source prose, without discarding the native identity.
    pub problems: Vec<Problem>,
    /// Selected card, absent for missing or conflicting identities.
    pub card: Option<GtrCard>,
    /// Conflict supplied before any current flag selection.
    pub conflict: Option<Problem>,
    /// Native relation rows, including rows without a current card.
    pub relations: Vec<GtrTestRelation>,
    /// Independent method lists with original indices.
    pub methods: Vec<GtrMethod>,
    /// Stored snapshot identity.
    pub info: BundleInfo,
}
/// Read one exact accession/version; no latest-version fallback or implicit synchronization.
pub fn get(
    leg: &Leg<'_>,
    query: &DiagnosticGet,
    id: &biodata::GtrTestId,
) -> Result<Record, RequestError> {
    let (rows, info) = bundle::read(leg, Find::Exact(KeyKind::Id, query.id().as_str()))?;
    super::revision(query.bundle_revision(), &info)?;
    let mut versions = Vec::new();
    let mut relations = Vec::new();
    let mut cx = crate::mapped::Problems {
        source: biomcp_types::Source::Gtr,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    for row in rows {
        match row.value {
            Native::Version(v) => versions.push(v),
            Native::Refused(r) => {
                refusals = crate::Refusal::Refused;
                r.disclose(&mut cx);
            }
            Native::Relation(r) => {
                if let Some(relation) = map::relation(&r, &mut cx, &mut refusals) {
                    relations.push(relation);
                }
            }
        }
    }
    let conflict = if versions.len() > 1 {
        Some(super::conflict(id))
    } else {
        None
    };
    let mut problems = cx.list;
    let card = if conflict.is_none() {
        versions
            .first()
            .map(|v| map::card(v, &mut problems))
            .transpose()?
    } else {
        None
    };
    let methods = if conflict.is_none() {
        versions
            .first()
            .map(|v| map::methods(v))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(Record {
        refusals,
        problems,
        card,
        conflict,
        relations,
        methods,
        info,
    })
}
