use super::links::{native_xrefs, strings};
use super::names::name;
use super::raw::{Component, Hit, Page};
pub(super) use crate::mapped::text;
use biodata::{DrugIdentityLevel, DrugIdentityRelation};
use biomcp_fetch::{BadReply, Leg};
use biomcp_types::{DrugCard, DrugHeader, DrugPage, DrugSource};

pub(super) fn components(
    hit: Hit,
    leg: &Leg<'_>,
    problems: &mut crate::mapped::Problems,
) -> (String, Option<u64>, Vec<(&'static str, Component)>) {
    let mut rows = Vec::new();
    for (name, values) in [
        ("drugbank", hit.drugbank),
        ("chembl", hit.chembl),
        ("drugcentral", hit.drugcentral),
        ("gtopdb", hit.gtopdb),
        ("unii", hit.unii),
        ("ndc", hit.ndc),
        ("openfda", hit.openfda),
    ] {
        if crate::drug_sources::components::all()
            .find(|c| c.name == name)
            .is_some_and(|c| !c.disabled_on_leg(leg))
            && let Some(value) = values.and_then(|value| problems.required(name, value.value()))
        {
            for row in value.values() {
                if let Some(row) = problems.required(name, row.value()) {
                    rows.push((name, super::normalize::component(row, name, problems)));
                }
            }
        }
    }
    (hit.id, problems.field("_version", hit.version), rows)
}

fn header(id: &str, rows: &[(&str, Component)]) -> Result<DrugHeader, BadReply> {
    Ok(DrugHeader {
        name_resolution: None,
        matched_name: None,
        id: id.parse().map_err(|_| BadReply)?,
        source: DrugSource::Mychem,
        version: None,
        name: rows
            .iter()
            .find_map(|(_, c)| name(c))
            .map(crate::mapped::source_text)
            .transpose()?,
        identity_level: DrugIdentityLevel::Unknown,
        r#type: rows
            .iter()
            .find_map(|(_, c)| {
                c.molecule_type
                    .as_deref()
                    .or(c.r#type.as_deref())
                    .or(c.ingredient_type.as_deref())
                    .or(c.producttypename.as_deref())
            })
            .map(text)
            .transpose()?,
        url: Some(text(&format!("https://mychem.info/v1/chem/{id}"))?),
    })
}
/// One page, with whether each item's record has drug rows of its own, not only product rows.
pub(super) fn page(
    page: Page,
    leg: &Leg<'_>,
    query: &str,
) -> Result<(crate::Mapped<DrugPage>, Vec<bool>), BadReply> {
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Mychem,
        record: None,
        list: Vec::new(),
    };
    let mut items = Vec::new();
    let mut drug = Vec::new();
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
        let (id, _, rows) = components(hit, leg, &mut problems);
        let rows = super::names::speaking(rows);
        let mut item = header(&id, &rows)?;
        item.matched_name = super::names::matching(&rows, query);
        items.push(item);
        drug.push(super::names::has_drug_rows(&rows));
    }
    let page = crate::Mapped {
        refusals,
        value: DrugPage {
            items: items.into(),
            total: Some(page.total),
        },
        problems: problems.list,
    };
    Ok((page, drug))
}
pub(super) fn card(
    hit: Hit,
    leg: &Leg<'_>,
) -> Result<crate::Mapped<super::DrugDocument>, BadReply> {
    let mut mapped = crate::mapped::Problems {
        source: biomcp_types::Source::Mychem,
        record: crate::mapped::record(&hit.id),
        list: Vec::new(),
    };
    let (id, version, rows) = components(hit, leg, &mut mapped);
    let mut card = DrugCard {
        header: header(&id, &rows)?,
        alternate_names: Box::default(),
        official_names: Box::default(),
        cross_references: Box::default(),
        parents: Box::default(),
        products: Box::default(),
        formulations: Box::default(),
        version: version.map(|n| text(&n.to_string())).transpose()?,
        updated: None,
        retrieved: None,
        available_sections: Box::default(),
        next: Box::default(),
    };
    let mut problems = mapped.list;
    let mut aliases = Vec::new();
    let mut official = Vec::new();
    let mut refs = Vec::new();
    let mut parents = Vec::new();
    for (component, c) in rows {
        aliases.extend(aliases_for(&c));
        official.extend(super::names::official(&c));
        refs.extend(references_for(&id, component, &c, &mut problems));
        refs.extend(native_xrefs(&id, component, &c, &mut problems));
        parents.extend(parents_for(&id, component, &c, &mut problems));
    }
    let mut names = Vec::new();
    for alias in aliases {
        let alias = crate::mapped::source_text(&alias)?;
        if !names.contains(&alias) {
            names.push(alias);
        }
    }
    card.alternate_names = names.into();
    let mut names = Vec::new();
    for name in official {
        let name = crate::mapped::source_text(&name)?;
        if !names.contains(&name) {
            names.push(name);
        }
    }
    card.official_names = names.into();
    let mut distinct = Vec::new();
    for reference in refs.into_iter().chain(parents) {
        if !distinct.contains(&reference) {
            distinct.push(reference);
        }
    }
    if !distinct.is_empty() || !problems.is_empty() {
        card.available_sections = vec![biomcp_types::DrugSection::Xrefs].into();
    }
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value: super::DrugDocument {
            card,
            xrefs: distinct.into(),
        },
        problems,
    })
}

pub(super) fn aliases_for(c: &Component) -> Vec<String> {
    let mut aliases = Vec::new();
    aliases.extend(
        c.synonyms
            .as_ref()
            .map(strings)
            .unwrap_or_default()
            .into_iter()
            .flat_map(super::names::split)
            .map(str::to_owned),
    );
    aliases.extend(
        c.molecule_synonyms
            .as_ref()
            .into_iter()
            .flatten()
            .filter_map(|s| s.molecule_synonym.as_deref())
            .flat_map(super::names::split)
            .map(str::to_owned),
    );
    for values in [&c.generic_name, &c.brand_name] {
        aliases.extend(
            values
                .as_ref()
                .map(strings)
                .unwrap_or_default()
                .into_iter()
                .flat_map(super::names::split)
                .map(str::to_owned),
        );
    }
    aliases
}
fn references_for(
    id: &str,
    component: &str,
    c: &Component,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<DrugIdentityRelation> {
    let mut refs = Vec::new();
    for (relation, target) in [
        (
            "drugbank.id",
            c.id.as_deref().filter(|_| component == "drugbank"),
        ),
        ("chembl.molecule_chembl_id", c.molecule_chembl_id.as_deref()),
        (
            "drugcentral.id",
            c.id.as_deref().filter(|_| component == "drugcentral"),
        ),
        ("gtopdb.ligand_id", c.ligand_id.as_deref()),
        ("gtopdb.chembl_id", c.chembl_id.as_deref()),
        ("unii.unii", c.unii.as_deref()),
        ("unii.rxcui", c.rxcui.as_deref()),
        ("unii.ncit", c.ncit.as_deref()),
        ("ndc.product_id", c.product_id.as_deref()),
        ("ndc.productndc", c.productndc.as_deref()),
        ("ndc.applicationnumber", c.applicationnumber.as_deref()),
    ] {
        if let Some(target) = target {
            let native_name = match relation {
                "drugbank.id"
                | "chembl.molecule_chembl_id"
                | "gtopdb.ligand_id"
                | "unii.unii"
                | "ndc.product_id"
                | "ndc.productndc" => name(c),
                _ => None,
            };
            refs.extend(super::links::mapped_reference(
                id,
                component,
                c,
                relation,
                target,
                native_name,
                problems,
            ));
        }
    }
    for (relation, values) in [
        ("openfda.spl_set_id", &c.spl_set_id),
        ("openfda.application_number", &c.application_number),
    ] {
        for target in values.as_ref().map(strings).unwrap_or_default() {
            refs.extend(super::links::mapped_reference(
                id, component, c, relation, target, None, problems,
            ));
        }
    }
    refs
}
fn parents_for(
    id: &str,
    component: &str,
    c: &Component,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<DrugIdentityRelation> {
    let mut parents = Vec::new();
    if let Some(h) = &c.molecule_hierarchy {
        for (relation, target) in [
            (
                "chembl.molecule_hierarchy.parent_chembl_id",
                &h.parent_chembl_id,
            ),
            (
                "chembl.molecule_hierarchy.active_chembl_id",
                &h.active_chembl_id,
            ),
        ] {
            if let Some(target) = target {
                parents.extend(super::links::mapped_reference(
                    id, component, c, relation, target, None, problems,
                ));
            }
        }
    }
    parents
}
