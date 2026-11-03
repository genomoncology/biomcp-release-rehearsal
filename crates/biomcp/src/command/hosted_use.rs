//! Hosted use restrictions preserve server exclusions and caller credential scope.
use super::Context;
use biomcp_fetch::{Fetch, KeyUse};
use biomcp_types::Use;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredUse {
    Restricted,
    Commercial,
    Academic,
}
impl DeclaredUse {
    pub(crate) fn parse(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("restricted") => Some(Self::Restricted),
            Some("commercial") => Some(Self::Commercial),
            Some("academic") => Some(Self::Academic),
            Some(_) => None,
        }
    }
}

pub(super) fn restrict(fetch: Fetch, context: Option<Context<'_>>) -> Fetch {
    let Some(context) = context.filter(|c| {
        matches!(
            c.declared_use,
            Some(DeclaredUse::Restricted | DeclaredUse::Commercial)
        )
    }) else {
        return fetch;
    };
    let disabled = [Use::Noncommercial, Use::Licensed]
        .into_iter()
        .flat_map(biomcp_query::sources::sources_for_use)
        .filter(|source| {
            context.declared_use == Some(DeclaredUse::Restricted)
                || !biomcp_query::sources::requests().any(|spec| {
                    spec.source == *source
                        && matches!(spec.key, KeyUse::Required(key) | KeyUse::Optional(key)
                            if context.secrets.is_some_and(|secrets| secrets.fingerprint(key.secret).is_some()))
                })
        })
        .collect();
    fetch.restrict_sources(disabled).restrict_uses(
        [Use::Noncommercial, Use::Licensed, Use::PerRecord]
            .into_iter()
            .collect(),
    )
}
