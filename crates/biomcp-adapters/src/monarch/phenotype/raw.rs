use biodata::{FiniteNumber, HpId, PhenotypeAssociationCount};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedNode")]
pub(super) struct Node {
    pub problems: Vec<biomcp_types::Problem>,
    pub id: Option<HpId>,
    pub name: Option<Box<str>>,
    pub category: Box<str>,
    pub namespace: Option<Box<str>>,
    pub description: Option<Box<str>>,
    pub score: Option<FiniteNumber>,
    pub matched_field: Option<Box<str>>,
    pub match_type: Option<Box<str>>,
    pub deprecated: Option<bool>,
    pub has_descendant_count: Option<u64>,
    pub association_counts: Option<crate::strict_input::Input<Box<[PhenotypeAssociationCount]>>>,
    pub synonym: Option<Box<[Box<str>]>>,
    pub exact_synonym: Option<Box<[Box<str>]>>,
    pub broad_synonym: Option<Box<[Box<str>]>>,
    pub narrow_synonym: Option<Box<[Box<str>]>>,
    pub related_synonym: Option<Box<[Box<str>]>>,
    pub xref: Option<Box<[biodata::Curie]>>,
    pub node_hierarchy: Option<Hierarchy>,
}
#[derive(Debug, Deserialize)]
pub(super) struct ReceivedNode {
    pub id: Option<crate::raw::Field<HpId>>,
    pub name: Option<crate::raw::Field<Box<str>>>,
    pub category: Box<str>,
    pub namespace: Option<Box<str>>,
    pub description: Option<crate::raw::Field<Box<str>>>,
    pub score: Option<crate::raw::Field<FiniteNumber>>,
    pub matched_field: Option<crate::raw::Field<Box<str>>>,
    pub match_type: Option<crate::raw::Field<Box<str>>>,
    pub deprecated: Option<crate::raw::Field<bool>>,
    pub has_descendant_count: Option<crate::raw::Field<u64>>,
    pub association_counts: Option<crate::raw::Field<Vec<crate::raw::Field<super::counts::Count>>>>,
    pub synonym: Option<crate::raw::Field<Vec<crate::raw::Field<Box<str>>>>>,
    pub exact_synonym: Option<crate::raw::Field<Vec<crate::raw::Field<Box<str>>>>>,
    pub broad_synonym: Option<crate::raw::Field<Vec<crate::raw::Field<Box<str>>>>>,
    pub narrow_synonym: Option<crate::raw::Field<Vec<crate::raw::Field<Box<str>>>>>,
    pub related_synonym: Option<crate::raw::Field<Vec<crate::raw::Field<Box<str>>>>>,
    pub xref: Option<crate::raw::Field<Vec<crate::raw::Field<biodata::Curie>>>>,
    pub node_hierarchy: Option<crate::raw::Field<Hierarchy>>,
}
impl From<ReceivedNode> for Node {
    fn from(row: ReceivedNode) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Monarch,
            record: row
                .id
                .as_ref()
                .and_then(crate::raw::Field::as_ref)
                .and_then(crate::mapped::record),
            list: Vec::new(),
        };
        Self {
            id: problems.required_field("id", row.id),
            category: row.category,
            namespace: row.namespace,
            name: problems.required_field("name", row.name),
            description: problems.field("description", row.description),
            score: problems.field("score", row.score),
            matched_field: problems.field("matched_field", row.matched_field),
            match_type: problems.field("match_type", row.match_type),
            deprecated: problems.field("deprecated", row.deprecated),
            has_descendant_count: problems.field("has_descendant_count", row.has_descendant_count),
            association_counts: row.association_counts.map(|v| {
                crate::strict_input::Input(
                    problems
                        .fields("association_counts", Some(v))
                        .into_vec()
                        .into_iter()
                        .map(|v| v.mapped(&mut problems))
                        .collect(),
                )
            }),
            synonym: row.synonym.map(|v| problems.fields("synonym", Some(v))),
            exact_synonym: row
                .exact_synonym
                .map(|v| problems.fields("exact_synonym", Some(v))),
            broad_synonym: row
                .broad_synonym
                .map(|v| problems.fields("broad_synonym", Some(v))),
            narrow_synonym: row
                .narrow_synonym
                .map(|v| problems.fields("narrow_synonym", Some(v))),
            related_synonym: row
                .related_synonym
                .map(|v| problems.fields("related_synonym", Some(v))),
            xref: row.xref.map(|v| problems.fields("xref", Some(v))),
            node_hierarchy: problems.field("node_hierarchy", row.node_hierarchy),
            problems: problems.list,
        }
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Hierarchy {
    pub super_classes: Option<crate::raw::Field<Vec<crate::raw::Field<Link>>>>,
    pub sub_classes: Option<crate::raw::Field<Vec<crate::raw::Field<Link>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Link {
    pub id: biodata::Curie,
    pub name: Option<crate::raw::Field<Box<str>>>,
}
impl Reply for Node {
    fn check(&self) -> Result<(), BadReply> {
        if self.category.as_ref() != "biolink:PhenotypicFeature"
            || self.namespace.as_deref().is_some_and(|v| v != "HP")
        {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Page {
    pub limit: u32,
    pub offset: u32,
    pub total: u64,
    pub items: Vec<Node>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.items.len() > self.limit as usize
            || (!self.items.is_empty()
                && u64::from(self.offset) + self.items.len() as u64 > self.total)
        {
            return Err(BadReply);
        }
        for node in &self.items {
            node.check()?;
        }
        Ok(())
    }
}
