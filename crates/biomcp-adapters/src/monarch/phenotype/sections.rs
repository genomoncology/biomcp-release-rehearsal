use super::raw;
use biodata::{
    Attribution, PhenotypeHierarchyLink, PhenotypeHierarchyRelation, PhenotypeSynonym,
    PhenotypeSynonymClass, PhenotypeXref,
};
type Rows = (
    Option<Box<[PhenotypeSynonym]>>,
    Option<Box<[PhenotypeXref]>>,
    Option<Box<[PhenotypeHierarchyLink]>>,
);
pub(super) fn rows(
    node: &mut raw::Node,
    attribution: &Attribution,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Rows {
    let mut synonyms = Vec::new();
    let mut available = false;
    for (rows, relationship) in [
        (&node.synonym, None),
        (&node.exact_synonym, Some(PhenotypeSynonymClass::Exact)),
        (&node.broad_synonym, Some(PhenotypeSynonymClass::Broad)),
        (&node.narrow_synonym, Some(PhenotypeSynonymClass::Narrow)),
        (&node.related_synonym, Some(PhenotypeSynonymClass::Related)),
    ] {
        if let Some(rows) = rows {
            available = true;
            synonyms.extend(rows.iter().map(|value| PhenotypeSynonym {
                value: value.clone(),
                relationship,
                attribution: attribution.clone(),
            }));
        }
    }
    let xrefs = node.xref.as_ref().map(|rows| {
        rows.iter()
            .map(|id| PhenotypeXref {
                id: id.clone(),
                predicate: None,
                attribution: attribution.clone(),
            })
            .collect()
    });
    let hierarchy = node.node_hierarchy.take().and_then(|hierarchy| {
        if hierarchy.super_classes.is_none() && hierarchy.sub_classes.is_none() {
            return None;
        }
        let mut links = Vec::new();
        for (field, rows, relation) in [
            (
                "node_hierarchy.super_classes",
                hierarchy.super_classes,
                PhenotypeHierarchyRelation::Parent,
            ),
            (
                "node_hierarchy.sub_classes",
                hierarchy.sub_classes,
                PhenotypeHierarchyRelation::Child,
            ),
        ] {
            for row in problems.records(field, rows, refusals) {
                links.push(PhenotypeHierarchyLink {
                    id: row.id,
                    label: problems.required_field(&format!("{field}.name"), row.name),
                    relation,
                    attribution: attribution.clone(),
                });
            }
        }
        Some(links.into())
    });
    (available.then(|| synonyms.into()), xrefs, hierarchy)
}
