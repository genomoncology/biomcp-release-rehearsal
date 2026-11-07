//! One bounded license lookup for the papers in a text-bearing reply.
use biodata::Pmid;
use biomcp_fetch::Leg;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

/// At most twenty record lookups and three seconds per text-bearing response.
const MAX_RECORDS: usize = 20;
const LOOKUP_BUDGET: Duration = Duration::from_secs(3);
const FINISH_BUDGET: Duration = Duration::from_millis(200);

/// One bounded lookup: the licenses found and the papers it actually settled.
#[derive(Debug, Default)]
pub struct Licenses {
    known: BTreeMap<Pmid, biodata::License>,
    checked: BTreeSet<Pmid>,
}
impl Licenses {
    /// The paper's explicit license, when the lookup found one.
    pub fn get(&self, id: Pmid) -> Option<biodata::License> {
        self.known.get(&id).copied()
    }
    /// A failed or capped lookup settles nothing, which is not a refusal.
    pub fn unchecked(&self, id: Pmid) -> bool {
        !self.checked.contains(&id)
    }
}

/// Resolve explicit PMC commercial licenses; unavailable metadata grants no rights.
/// Any paper left unchecked keeps this leg's reply out of the source cache.
pub fn licenses(leg: &Leg<'_>, ids: &[Pmid]) -> Licenses {
    let mut seen = BTreeSet::new();
    let ids: Vec<_> = ids.iter().copied().filter(|id| seen.insert(*id)).collect();
    let capped: Vec<_> = ids.iter().copied().take(MAX_RECORDS).collect();
    let found = bounded(leg, &capped);
    if ids.iter().any(|id| found.unchecked(*id)) {
        leg.note_license_unchecked();
    }
    found
}
fn bounded(leg: &Leg<'_>, ids: &[Pmid]) -> Licenses {
    let mut found = Licenses::default();
    if ids.is_empty() {
        return found;
    }
    let deadline = leg.local_deadline();
    let mut remaining = LOOKUP_BUDGET.min(leg.remaining().saturating_sub(FINISH_BUDGET));
    if remaining.is_zero() {
        return found;
    }
    let before = leg.remaining();
    let headers = leg.dependency(&crate::pubmed::SPEC, remaining, |dependency| {
        crate::pubmed::summaries(dependency, ids, None)
    });
    remaining = remaining.saturating_sub(before.saturating_sub(leg.remaining()));
    // Failed metadata never grants record text rights; the caller retains the paper and reason.
    let headers = match headers {
        Ok(headers) => headers,
        Err(_) => return found,
    };
    for header in headers.value {
        let Some(id) = header.pmid else {
            continue;
        };
        let Some(pmcid) = header.pmcid else {
            found.checked.insert(id);
            continue;
        };
        if remaining.is_zero() || deadline.expired() {
            break;
        }
        let before = leg.remaining();
        let license = leg.dependency(&crate::pmc_oa::SPEC, remaining, |dependency| {
            crate::pmc_oa::record_license(dependency, pmcid, Some(id))
        });
        remaining = remaining.saturating_sub(before.saturating_sub(leg.remaining()));
        // Failed metadata cannot supply a license; the paper still reaches the caller.
        let license = match license {
            Ok(license) => license,
            Err(_) => continue,
        };
        found.checked.insert(id);
        if let Some(license) = license {
            found.known.insert(id, license);
        }
    }
    found
}
