//! Exact native identities and independent bounded context collections.
use biomcp_fetch::Input;
mod raw;
pub(super) mod study;
use super::{SPEC, raw::Association};
use biodata::{GwasAssociationId, GwasStudyId};
use biomcp_fetch::{BadReply, Freshness, Leg, Reply, RequestError};
use biomcp_types::{GwasAncestry, GwasLocus, GwasStudy};
/// Checked rows and source rows rejected for an owning-identity mismatch.
#[derive(Debug)]
pub struct Rows<T> {
    /// Actual source rows refused at the ownership boundary.
    pub refusals: crate::Refusal,
    /// Rejected native metadata for retained source rows.
    pub problems: Vec<biomcp_types::Problem>,
    /// Source rows in native order.
    pub items: Vec<T>,
    /// Number of rejected source rows.
    pub rejected: usize,
}
/// Read one source association without fetching its study.
pub fn association(
    leg: &Leg<'_>,
    id: GwasAssociationId,
) -> Result<super::Association, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/associations/{}", id.get()))?
        .fresh(Freshness::Days(1));
    let (row, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<Association>::json().check(&|r| {
                if r.association_id == id {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(row.into())
}
impl Reply for Association {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Read one exact study with unchanged source sample descriptions.
pub fn study(leg: &Leg<'_>, id: &GwasStudyId) -> Result<crate::Mapped<GwasStudy>, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/studies/{id}"))?
        .fresh(Freshness::Days(1));
    let (row, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<study::Study>::json().check(&|r| {
                if &r.accession_id == id {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(row.card())
}
/// Read one unpaged ancestry collection; no invented source size selector.
pub fn ancestries(leg: &Leg<'_>, id: &GwasStudyId) -> Result<Rows<GwasAncestry>, RequestError> {
    let path = format!("/studies/{id}/ancestries");
    let prefix = format!("{}{path}/", SPEC.base);
    let endpoint = SPEC.get("").with_path(path)?.fresh(Freshness::Days(1));
    let (rows, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Ancestries>::json().cache_if(&|r| {
                r.embedded
                    .ancestries
                    .iter()
                    .all(|a| raw::owned(&a.links.current.href, &prefix))
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let total = rows.embedded.ancestries.len();
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Gwas,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let items: Vec<_> = rows
        .embedded
        .ancestries
        .into_iter()
        .filter_map(|a| {
            if raw::owned(&a.links.current.href, &prefix) {
                Some(a.row(&mut problems, &mut refusals))
            } else {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .collect();
    Ok(Rows {
        refusals,
        problems: problems.list,
        rejected: total - items.len(),
        items,
    })
}
/// Read source locus descriptions and stable reported gene identities.
pub fn loci(leg: &Leg<'_>, id: GwasAssociationId) -> Result<Rows<GwasLocus>, RequestError> {
    let path = format!("/associations/{}/loci", id.get());
    let prefix = format!("{}{path}/", SPEC.base);
    let endpoint = SPEC.get("").with_path(path)?.fresh(Freshness::Days(1));
    let (rows, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Loci>::json().cache_if(&|r| {
                r.embedded
                    .loci
                    .iter()
                    .all(|a| raw::owned(&a.links.current.href, &prefix))
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let total = rows.embedded.loci.len();
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Gwas,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let items: Vec<_> = rows
        .embedded
        .loci
        .into_iter()
        .filter_map(|a| {
            if raw::owned(&a.links.current.href, &prefix) {
                Some(a.row(&mut problems, &mut refusals))
            } else {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .collect();
    Ok(Rows {
        refusals,
        problems: problems.list,
        rejected: total - items.len(),
        items,
    })
}
