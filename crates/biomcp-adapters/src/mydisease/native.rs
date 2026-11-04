//! Native annotation facts before response-specific paging.
use biodata::{Curie, Text};

/// One exact received annotation with independently attributed component facts.
#[derive(Debug)]
pub struct Record {
    /// Rejected top-level metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Exact caller-selected identifier.
    pub requested_id: Curie,
    /// Native top-level identifier, checked against the request or its own alias.
    pub returned_id: Curie,
    /// Native record revision, independent of ontology release.
    pub version: Option<u64>,
    /// Present, admitted components; no component is fetched separately.
    pub components: Box<[Component]>,
    /// Original transport retrieval time for this record.
    pub metadata: biomcp_fetch::ReplyMetadata,
}
/// A single component with its own native identity and complete annotation.
#[derive(Debug)]
pub struct Component {
    /// Declared component name used by the shared terms registry.
    pub name: &'static str,
    /// Native component identity, distinct from the aggregate identity.
    pub id: Curie,
    /// Native primary display label when present and valid.
    pub label: Option<Text>,
    /// Complete optional source definition, without borrowing another component's text.
    pub definition: Option<Text>,
    /// Native synonym scope and text, preserving component-local multiplicity.
    pub synonyms: Box<[Value]>,
    /// Native namespace field and original code, without merging namespaces.
    pub xrefs: Box<[Reference]>,
    /// Native direct parents; ancestors and consider are never added.
    pub parents: Box<[Curie]>,
    /// Publisher obsolete state, absent when not supplied.
    pub obsolete: Option<bool>,
    /// Publisher alternatives, never parent relations.
    pub consider: Option<Box<[Curie]>>,
    /// Publisher replacements, never selected automatically.
    pub replaced_by: Option<Box<[Curie]>>,
    /// Rejected optional provider values, attributed to this component.
    pub problems: Vec<biomcp_types::Problem>,
}
/// One original component synonym value.
#[derive(Debug)]
pub struct Value {
    /// Native synonym scope spelling.
    pub field: String,
    /// Complete source synonym text.
    pub value: Text,
}
/// One native reference whose code bytes have not passed through display text.
#[derive(Debug)]
pub struct Reference {
    /// Original namespace field.
    pub field: String,
    /// Exact source code, checked by the reference mapper.
    pub value: String,
}
pub(super) fn component(name: &'static str, raw: super::raw::Component) -> Component {
    let root = if name == "doid" {
        "disease_ontology"
    } else {
        name
    };
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::MyDisease,
        record: crate::mapped::record(&raw.id),
        list: Vec::new(),
    };
    let definition = problems.optional(
        &format!(
            "{root}.{}",
            if name == "doid" { "def" } else { "definition" }
        ),
        raw.definition.map(|v| v.value().map(Into::into).ok_or(())),
    );
    let synonyms = values(
        raw.synonym,
        &format!(
            "{root}.{}",
            if name == "doid" {
                "synonyms"
            } else {
                "synonym"
            }
        ),
        &mut problems,
    );
    let xrefs = references(raw.xrefs, &format!("{root}.xrefs"), &mut problems);
    let parents = ids(raw.parents, &format!("{root}.parents"), &mut problems).unwrap_or_default();
    let consider = ids(raw.consider, &format!("{root}.consider"), &mut problems);
    let replaced_by = ids(
        raw.replaced_by,
        &format!("{root}.replaced_by"),
        &mut problems,
    );
    Component {
        name,
        id: raw.id,
        label: problems.optional(
            &format!("{root}.{}", if name == "doid" { "name" } else { "label" }),
            raw.label.map(|v| v.value().map(Into::into).ok_or(())),
        ),
        definition,
        synonyms,
        xrefs,
        parents,
        obsolete: problems.field(&format!("{root}.is_obsolete"), raw.is_obsolete),
        consider,
        replaced_by,
        problems: problems.list,
    }
}
fn ids(
    values: Option<crate::raw::Field<Box<[crate::raw::Field<Curie>]>>>,
    field: &str,
    problems: &mut crate::mapped::Problems,
) -> Option<Box<[Curie]>> {
    values
        .and_then(|values| problems.required(field, values.value()))
        .map(|values| {
            values
                .into_vec()
                .into_iter()
                .filter_map(|v| problems.optional(field, Some(v.value().ok_or(()))))
                .collect()
        })
}
fn values(
    fields: Option<
        crate::raw::Field<
            std::collections::BTreeMap<
                String,
                crate::raw::OneOrMany<crate::raw::Field<crate::native_text::NativeText>>,
            >,
        >,
    >,
    root: &str,
    problems: &mut crate::mapped::Problems,
) -> Box<[Value]> {
    let mut result = Vec::new();
    for (field, values) in fields
        .and_then(|fields| problems.required(root, fields.value()))
        .unwrap_or_default()
    {
        for value in values.values() {
            if let Some(value) =
                problems.optional(&format!("{root}.{field}"), Some(value.value().ok_or(())))
            {
                result.push(Value {
                    field: field.clone(),
                    value: value.into(),
                });
            }
        }
    }
    result.into()
}

fn references(
    fields: Option<
        crate::raw::Field<
            std::collections::BTreeMap<String, crate::raw::OneOrMany<crate::raw::Field<String>>>,
        >,
    >,
    root: &str,
    problems: &mut crate::mapped::Problems,
) -> Box<[Reference]> {
    let mut result = Vec::new();
    for (field, values) in fields
        .and_then(|fields| problems.required(root, fields.value()))
        .unwrap_or_default()
    {
        for value in values.values() {
            if let Some(value) =
                problems.optional(&format!("{root}.{field}"), Some(value.value().ok_or(())))
            {
                result.push(Reference {
                    field: field.clone(),
                    value,
                });
            }
        }
    }
    result.into()
}
