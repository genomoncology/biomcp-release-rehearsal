//! Only reviewed MyChem fields may enter a source projection.
use crate::registry::Component;
use biomcp_fetch::{Fetch, Leg, Terms};
use biomcp_types::Source;
pub(crate) const fn component(
    name: &'static str,
    native_source: Source,
    terms: Terms,
) -> Component {
    Component {
        source: Source::Mychem,
        native_source: Some(native_source),
        name,
        terms,
    }
}
/// Look up independent terms without granting rights to undeclared fields.
pub fn terms(name: &str) -> Option<Terms> {
    all().find(|c| c.name == name).map(|c| c.terms)
}
/// Inspect source and component policy before reading or constructing a response.
pub fn disabled(fetch: &Fetch, name: &str) -> bool {
    all()
        .find(|c| c.name == name)
        .is_none_or(|c| c.disabled(fetch))
}
/// Filter the exact source projection before its request/cache/replay identity exists.
/// Identity and provider result metadata are the only unlabeled fields retained.
pub fn allowed_fields(leg: &Leg<'_>, fields: &str) -> String {
    fields
        .split(',')
        .filter(|field| {
            let name = field.split('.').next().unwrap_or(field);
            matches!(*field, "_id" | "_score")
                || all()
                    .find(|c| c.name == name)
                    .is_some_and(|c| !c.disabled_on_leg(leg))
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Enumerate independently labeled fields from their adapter declarations.
pub fn all() -> impl Iterator<Item = &'static Component> {
    crate::mychem::decl::DECL.components.iter()
}
