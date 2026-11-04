//! Complete native names used by the same MyChem search source.
use super::{links::strings, raw::Component};
use biomcp_types::DrugNameMatch;
pub(super) fn name(c: &Component) -> Option<&str> {
    c.name
        .as_deref()
        .or(c.pref_name.as_deref())
        .or(c.display_name.as_deref())
        .or(c.proprietaryname.as_deref())
        .or(c.nonproprietaryname.as_deref())
}
pub(super) fn matching(rows: &[(&str, Component)], query: &str) -> Option<DrugNameMatch> {
    let exact = |s: &str| DrugNameMatch::same_name(s, query);
    let brand = rows.iter().any(|(_, c)| {
        let real_brand = |brand: &str| {
            exact(brand)
                && !generic_names(c)
                    .into_iter()
                    .flatten()
                    .any(|g| DrugNameMatch::same_name(brand, g))
                && !c.generic_name.as_ref().is_some_and(|v| {
                    strings(v)
                        .into_iter()
                        .any(|g| DrugNameMatch::same_name(brand, g))
                })
        };
        c.proprietaryname.as_deref().is_some_and(real_brand)
            || c.brand_name
                .as_ref()
                .is_some_and(|v| strings(v).into_iter().any(real_brand))
    });
    let generic = rows.iter().any(|(_, c)| {
        generic_names(c)
            .into_iter()
            .flatten()
            .any(|name| DrugNameMatch::generic_name(name, query))
            || c.generic_name.as_ref().is_some_and(|v| {
                strings(v)
                    .into_iter()
                    .any(|name| DrugNameMatch::generic_name(name, query))
            })
            || c.synonyms
                .as_ref()
                .is_some_and(|v| strings(v).into_iter().any(exact))
            || c.molecule_synonyms
                .iter()
                .flatten()
                .any(|s| s.molecule_synonym.as_deref().is_some_and(exact))
    });
    DrugNameMatch::from_fields(brand, generic)
}

fn generic_names(c: &Component) -> [Option<&str>; 4] {
    [
        c.name.as_deref(),
        c.pref_name.as_deref(),
        c.display_name.as_deref(),
        c.nonproprietaryname.as_deref(),
    ]
}
