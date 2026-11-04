//! Preserve native identity relations and their component record attribution.
use super::{map::text, raw::Component};
use biodata::{Attribution, DrugIdentity, DrugIdentityLevel, DrugIdentityRelation};
use biomcp_fetch::BadReply;
pub(super) fn strings(values: &crate::raw::OneOrMany<String>) -> Vec<&str> {
    match values {
        crate::raw::OneOrMany::One(v) => vec![v],
        crate::raw::OneOrMany::Many(v) => v.iter().map(String::as_str).collect(),
    }
}
pub(super) fn reference(
    record: &str,
    component: &str,
    c: &Component,
    relation: &str,
    target: &str,
    name: Option<&str>,
) -> Result<DrugIdentityRelation, BadReply> {
    let source = relation.split('.').next().unwrap_or(component);
    let target_source = relation
        .split_once(".xrefs.")
        .map(|(_, key)| key)
        .unwrap_or(match relation {
            "unii.rxcui" => "rxnorm",
            "unii.ncit" => "ncit",
            "ndc.applicationnumber" | "openfda.application_number" => "drugsfda",
            "openfda.spl_set_id" => "openfda-label",
            "gtopdb.chembl_id" => "chembl",
            _ => source,
        });
    Ok(DrugIdentityRelation {
        identity: DrugIdentity {
            id: target.parse().map_err(|_| BadReply)?,
            namespace: text(target_source)?,
            name: name.map(text).transpose()?,
            identity_level: DrugIdentityLevel::Unknown,
            r#type: None,
            url: None,
        },
        relation: text(relation)?,
        attribution: Attribution {
            approved: None,
            published: None,
            source: text(component)?,
            record: native_record(component, c).map(text).transpose()?,
            version: None,
            release: None,
            submitter: None,
            evaluated: None,
            url: Some(text(&format!("https://mychem.info/v1/chem/{record}"))?),
        },
    })
}
pub(super) fn native_xrefs(
    record: &str,
    component: &str,
    c: &Component,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<DrugIdentityRelation> {
    let mut rows = Vec::new();
    for (namespace, ids) in c.xrefs.as_ref().into_iter().flatten() {
        let relation = format!("{component}.xrefs.{namespace}");
        for id in strings(ids) {
            rows.extend(mapped_reference(
                record, component, c, &relation, id, None, problems,
            ));
        }
    }
    rows
}

fn native_record<'a>(component: &str, c: &'a Component) -> Option<&'a str> {
    match component {
        "drugbank" | "drugcentral" => c.id.as_deref(),
        "chembl" => c.molecule_chembl_id.as_deref(),
        "gtopdb" => c.ligand_id.as_deref(),
        "unii" => c.unii.as_deref(),
        "ndc" => c.product_id.as_deref().or(c.productndc.as_deref()),
        _ => None,
    }
}

pub(super) fn mapped_reference(
    record: &str,
    component: &str,
    c: &Component,
    relation: &str,
    target: &str,
    name: Option<&str>,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<DrugIdentityRelation> {
    let mut context = crate::mapped::Problems {
        source: biomcp_types::Source::Mychem,
        record: crate::mapped::record(record),
        list: Vec::new(),
    };
    let value = context.optional(
        relation,
        Some(reference(record, component, c, relation, target, name)),
    );
    for problem in context.list {
        crate::mapped::push_problem(problems, problem.source, problem.record, &problem.field);
    }
    value
}
