//! Expected native facts come from unchanged publisher fixtures.
use super::{Value, json, path};
fn rows(value: &Value, key: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
fn field(value: &Value, key: &str) -> Result<Value, String> {
    value
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing native {key}"))
}
fn variation(value: &Value) -> Result<Value, String> {
    let mut result = serde_json::Map::new();
    result.insert("variation_type".into(), field(value, "variation-type")?);
    for (source, target) in [
        ("mutation-description", "description"),
        ("mutation-type", "mutation_type"),
        ("zygosity-type", "zygosity"),
        ("variation-note", "note"),
    ] {
        if let Some(field) = value.get(source) {
            result.insert(target.into(), field.clone());
        }
    }
    let mut genes = Vec::new();
    let mut references = Vec::new();
    for reference in rows(value, "xref-list").into_iter().map(identifier) {
        if reference.get("namespace") == Some(&json!("HGNC")) {
            genes.push(json!({"reference":reference.clone(), "hgnc":field(&reference, "id")?}));
        } else {
            references.push(reference.clone());
        }
    }
    result.insert("genes".into(), json!(genes));
    result.insert("references".into(), json!(references));
    result.insert(
        "provenance".into(),
        json!(
            rows(value, "source-list")
                .into_iter()
                .map(|mut source| {
                    if let Some(reference) = source.get_mut("reference")
                        && let Some(object) = reference.as_object_mut()
                        && let Some(value) = object.remove("resource-internal-ref")
                    {
                        object.insert("resource_internal_ref".into(), value);
                    }
                    if let Some(xref) = source.get_mut("xref") {
                        *xref = identifier(xref.clone());
                    }
                    source
                })
                .collect::<Vec<_>>()
        ),
    );
    Ok(Value::Object(result))
}
pub(in crate::runner::cell_line) fn page(
    source: &Value,
    section: &str,
    value: &Value,
    offset: usize,
    limit: usize,
) -> Result<(), String> {
    path(value, "/result/offset", json!(offset))?;
    if section == "identity" {
        for (native, field) in [("species-list", "species"), ("disease-list", "diseases")] {
            collection(
                value,
                field,
                rows(source, native).into_iter().map(identifier).collect(),
                offset,
                limit,
            )?;
        }
        let warnings = rows(source, "comment-list")
            .into_iter()
            .filter(|r| {
                matches!(
                    r.get("category").and_then(Value::as_str),
                    Some("Caution" | "Problematic cell line")
                )
            })
            .map(|r| Ok(json!({"scope":field(&r, "category")?,"text":field(&r, "value")?})))
            .collect::<Result<Vec<_>, String>>()?;
        return collection(value, "warnings", warnings, offset, limit);
    }
    let items = match section {
        "names" => rows(source, "name-list")
            .into_iter()
            .map(|r| Ok(json!({"kind":field(&r, "type")?,"value":field(&r, "value")?})))
            .collect::<Result<Vec<_>, String>>()?,
        "xrefs" => rows(source, "xref-list")
            .into_iter()
            .map(identifier)
            .collect(),
        "variants" => rows(source, "sequence-variation-list")
            .iter()
            .map(variation)
            .collect::<Result<Vec<_>, String>>()?,
        _ => return Err("unknown section".into()),
    };
    collection(value, "items", items, offset, limit)
}
fn collection(
    value: &Value,
    field: &str,
    rows: Vec<Value>,
    offset: usize,
    limit: usize,
) -> Result<(), String> {
    path(value, &format!("/result/{field}/total"), json!(rows.len()))?;
    path(
        value,
        &format!("/result/{field}/items"),
        json!(
            rows.into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>()
        ),
    )
}

fn identifier(mut reference: Value) -> Value {
    if let Some(object) = reference.as_object_mut() {
        for (old, new) in [("database", "namespace"), ("accession", "id")] {
            if let Some(value) = object.remove(old) {
                object.insert(new.into(), value);
            }
        }
    }
    reference
}
