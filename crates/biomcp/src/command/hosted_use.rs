//! Commercial hosted calls add registry restrictions without weakening server policy.
use super::Context;
use biomcp_fetch::{Fetch, KeyUse};
use biomcp_types::Use;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredUse {
    Commercial,
    Academic,
}
impl DeclaredUse {
    pub(crate) fn parse(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("commercial") => Some(Self::Commercial),
            Some("academic") => Some(Self::Academic),
            Some(_) => None,
        }
    }
}

pub(super) fn restrict(fetch: Fetch, context: Option<Context<'_>>) -> Fetch {
    let Some(context) =
        context.filter(|context| context.declared_use == Some(DeclaredUse::Commercial))
    else {
        return fetch;
    };
    let groups = ["noncommercial".into(), "restricted".into()];
    let disabled = biomcp_query::sources::disable_groups()
        .into_iter()
        .filter(|group| matches!(group.group, Use::Noncommercial | Use::Restricted))
        .flat_map(|group| group.sources)
        .filter(|source| {
            !biomcp_query::sources::requests().any(|spec| {
                spec.source == *source
                    && matches!(spec.key, KeyUse::Required(key) | KeyUse::Optional(key)
                        if context.secrets.is_some_and(|secrets| secrets.fingerprint(key.secret).is_some()))
            })
        })
        .collect();
    fetch
        .restrict_sources(disabled)
        .restrict_uses(biomcp_query::sources::disabled_uses(&groups))
}
