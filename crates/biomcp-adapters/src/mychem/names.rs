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

/// Keep rows stating the name; the page alone never decides that they describe one drug.
pub(super) fn name_records(
    page: super::raw::Page,
    leg: &biomcp_fetch::Leg<'_>,
    query: &str,
) -> Result<crate::Mapped<Vec<super::NameRecord>>, biomcp_fetch::BadReply> {
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Mychem,
        record: None,
        list: Vec::new(),
    };
    let mut records = Vec::new();
    let mut refusals = crate::Refusal::None;
    for hit in page.hits {
        problems.record = None;
        let Some(hit) = problems.field("_id", Some(hit)) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        problems.record = crate::mapped::record(&hit.id);
        if problems.optional("_id", Some(hit.identity())).is_none() {
            refusals = crate::Refusal::Refused;
            continue;
        }
        let (id, _, rows) = super::map::components(hit, leg, &mut problems);
        if matching(&rows, query).is_some()
            || stated(&rows)
                .iter()
                .any(|name| DrugNameMatch::same_identity(name, query))
        {
            let names = drug_names(&rows);
            records.push(super::NameRecord {
                structure: structure(&id, &rows),
                names: names
                    .iter()
                    .filter_map(|name| problems.text("name", Some(name.as_str())))
                    .collect(),
                preferred: preferred(&rows),
                id,
            });
        }
    }
    Ok(crate::Mapped {
        refusals,
        value: records,
        problems: problems.list,
    })
}
/// The names of the drug itself. A product row can be a combination or a store brand, and even
/// its generic name cannot show one ingredient, so it speaks only when no drug row exists.
fn drug_names(rows: &[(&str, Component)]) -> Vec<String> {
    let product = |component: &str| matches!(component, "ndc" | "openfda");
    if rows.iter().all(|(component, _)| product(component)) {
        return stated(rows);
    }
    rows.iter()
        .filter(|(component, _)| !product(component))
        .flat_map(|row| stated(std::slice::from_ref(row)))
        .collect()
}
/// The main name each source gives the row.
fn preferred(rows: &[(&str, Component)]) -> Vec<String> {
    rows.iter()
        .flat_map(|(_, c)| {
            [
                c.name.as_deref(),
                c.pref_name.as_deref(),
                c.display_name.as_deref(),
            ]
        })
        .flatten()
        .map(str::to_owned)
        .collect()
}
/// Every name the rows state, with source values that join several names split apart.
fn stated(rows: &[(&str, Component)]) -> Vec<String> {
    rows.iter()
        .flat_map(|(_, c)| {
            generic_names(c)
                .into_iter()
                .chain([c.proprietaryname.as_deref()])
                .flatten()
                .flat_map(split)
                .map(str::to_owned)
                .chain(super::map::aliases_for(c))
        })
        .collect()
}

/// One source value may hold several names joined by a pipe.
pub(super) fn split(value: &str) -> impl Iterator<Item = &str> {
    value
        .split('|')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// The structure a row describes: an InChIKey's skeleton, so stereo forms of one molecule agree.
/// A row with no chemical identity of its own, such as an NDC product row, has none.
fn structure(id: &str, rows: &[(&str, Component)]) -> Option<String> {
    let blocks: Vec<_> = id.split('-').collect();
    if let [skeleton, stereo, charge] = blocks.as_slice()
        && (skeleton.len(), stereo.len(), charge.len()) == (14, 10, 1)
        && id.chars().all(|c| c == '-' || c.is_ascii_uppercase())
    {
        return Some((*skeleton).to_owned());
    }
    rows.iter()
        .any(|(component, _)| !matches!(*component, "ndc" | "openfda"))
        .then(|| id.to_owned())
}

fn generic_names(c: &Component) -> [Option<&str>; 4] {
    [
        c.name.as_deref(),
        c.pref_name.as_deref(),
        c.display_name.as_deref(),
        c.nonproprietaryname.as_deref(),
    ]
}
