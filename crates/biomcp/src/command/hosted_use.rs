//! Hosted use restrictions preserve server exclusions and caller credential scope.
use super::Context;
use biomcp_fetch::{Fetch, KeyUse};
use biomcp_types::Use;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DeclaredUse {
    Restricted,
    Commercial,
    #[default]
    Academic,
}
impl DeclaredUse {
    pub(crate) fn permits(self, usage: Self) -> bool {
        usage <= self
    }
    pub(crate) fn allowed(self) -> &'static str {
        match self {
            Self::Restricted => "restricted",
            Self::Commercial => "restricted, commercial",
            Self::Academic => "restricted, commercial, academic",
        }
    }
    pub(crate) fn parse(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("restricted") => Some(Self::Restricted),
            Some("commercial") => Some(Self::Commercial),
            Some("academic") => Some(Self::Academic),
            Some(_) => None,
        }
    }
}

pub(crate) fn restrict(fetch: Fetch, context: Option<Context<'_>>) -> Fetch {
    let Some(context) = context.filter(|c| {
        matches!(
            c.declared_use,
            Some(DeclaredUse::Restricted | DeclaredUse::Commercial)
        )
    }) else {
        return fetch;
    };
    let fetch = if context.declared_use == Some(DeclaredUse::Restricted) {
        fetch.restrict_studies()
    } else {
        fetch
    };
    fetch
        .restrict_sources(disabled(context.declared_use, context.secrets))
        .restrict_uses(
            [Use::Noncommercial, Use::Licensed, Use::PerRecord]
                .into_iter()
                .collect(),
        )
}

pub(crate) fn disabled(
    declared_use: Option<DeclaredUse>,
    secrets: Option<&biomcp_fetch::Secrets>,
) -> std::collections::BTreeSet<biomcp_types::Source> {
    if !matches!(
        declared_use,
        Some(DeclaredUse::Restricted | DeclaredUse::Commercial)
    ) {
        return Default::default();
    }
    [Use::Noncommercial, Use::Licensed]
        .into_iter()
        .flat_map(biomcp_query::sources::sources_for_use)
        .filter(|source| {
            declared_use == Some(DeclaredUse::Restricted)
                || !biomcp_query::sources::requests().any(|spec| {
                    spec.source == *source
                        && matches!(spec.key, KeyUse::Required(key)
                            if secrets.is_some_and(|secrets| secrets.fingerprint(key.secret).is_some()))
                })
        })
        .collect()
}
