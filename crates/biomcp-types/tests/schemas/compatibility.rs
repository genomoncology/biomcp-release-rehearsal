#[path = "compatibility/scalars.rs"]
mod scalars;
use super::policy::{ContractKind, Reviews};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

type Check<T> = Result<T, String>;
const STRING_FORMATS: &[&str] = &["date", "date-time", "uri", "uri-reference"];
#[derive(Debug, Default, Clone)]
struct Shape {
    redirect: Option<(String, String)>,
    referents: BTreeMap<String, Shape>,
    nodes: Vec<Value>,
    children: BTreeMap<String, String>,
    types: BTreeSet<String>,
    values: Option<BTreeSet<String>>,
    open_enum: bool,
    enum_assertion: bool,
    required: BTreeSet<String>,
    selectors: Option<Vec<BTreeSet<String>>>,
    minimum_length: u64,
    maximum_length: Option<u64>,
    minimum_items: u64,
    unique_items: bool,
    maximum_items: Option<u64>,
    pattern: Option<String>,
    minimum: Option<Bound>,
    maximum: Option<Bound>,
    closed: bool,
    markdown: bool,
    string_format: Option<String>,
    impossible: bool,
    map_values: bool,
    map_keys: bool,
    map_patterns: BTreeSet<String>,
}
pub(super) fn check(old: &Value, new: &Value) -> Check<()> {
    check_reviewed(old, new, ContractKind::Query, &Reviews::default())
}

pub(super) fn check_reviewed(
    old: &Value,
    new: &Value,
    kind: ContractKind,
    reviews: &Reviews,
) -> Check<()> {
    let baseline = reviews.enum_baseline(old, new);
    let old = &baseline;
    let old_shapes = shapes(old)?;
    let new_shapes = shapes(new)?;
    let (old_name, old_major) = identity(old)?;
    let (new_name, new_major) = identity(new)?;
    if new_major < old_major
        || old_name != new_name
            && !(new_major > old_major
                && matches!(kind, ContractKind::Reply)
                && reviews.successor(old, new))
    {
        return Err("schema identity changed or major version decreased".into());
    }
    if new_major > old_major {
        return Ok(());
    }
    let mut pending = vec![(String::new(), String::new())];
    let mut assumed = BTreeSet::new();
    while let Some((path, new_path)) = pending.pop() {
        // A repeated pair is an assumption, discharged only by the whole loop.
        if !assumed.insert((path.clone(), new_path.clone())) {
            continue;
        }
        let old = old_shapes.get(&path).ok_or("missing old structural path")?;
        let new = new_shapes
            .get(&new_path)
            .ok_or("missing new structural path")?;
        if !compatible(old, new, kind, reviews, (old_name, old_major, &path)) {
            return Err(format!("incompatible contract at {path}"));
        }
        for (edge, child) in &old.children {
            let new_child = new
                .children
                .get(edge)
                .ok_or_else(|| format!("removed {path}/{edge}"))?;
            pending.push((child.clone(), new_child.clone()));
        }
    }
    Ok(())
}
fn compatible(
    old: &Shape,
    new: &Shape,
    kind: ContractKind,
    reviews: &Reviews,
    identity: (&str, u64, &str),
) -> bool {
    let (old_name, old_major, path) = identity;
    !(old.map_patterns != new.map_patterns
        || old.impossible != new.impossible
        || new.map_values && !old.map_values
        || new.map_keys && !old.map_keys
        || old.markdown && !new.markdown
        || old.string_format.is_some() && old.string_format != new.string_format
        || old.pattern != new.pattern
            && !(matches!(kind, ContractKind::Reply)
                && reviews.pattern(
                    old_name,
                    old_major,
                    path,
                    old.pattern.as_deref(),
                    new.pattern.as_deref(),
                )
                || reviews.narrowing(
                    old_name,
                    old_major,
                    path,
                    old.pattern.as_deref(),
                    new.pattern.as_deref(),
                ))
        || new
            .minimum
            .is_some_and(|n| old.minimum.is_none_or(|o| n > o))
        || lowered(old.maximum, new.maximum)
        || new.selectors.as_ref().is_some_and(|new| {
            old.selectors.as_ref().map_or_else(
                || !new.iter().any(|branch| branch.is_subset(&old.required)),
                |alternatives| {
                    alternatives.iter().any(|branch| {
                        !new.iter().any(|required| {
                            required
                                .iter()
                                .all(|field| old.required.contains(field) || branch.contains(field))
                        })
                    })
                },
            )
        })
        || old.open_enum && !new.open_enum
        || old.types != new.types
        || match kind {
            ContractKind::Query => !new.required.is_subset(&old.required),
            ContractKind::Reply => !old.required.is_subset(&new.required),
        }
        || lowered(old.maximum_length, new.maximum_length)
        || new.minimum_length > old.minimum_length
        || new.minimum_items > old.minimum_items
        || new.unique_items && !old.unique_items
        || lowered(old.maximum_items, new.maximum_items)
        || (!old.closed && new.closed)
        || match (&old.values, &new.values) {
            (Some(old), Some(new)) => !old.is_subset(new),
            (None, Some(_)) => true,
            _ => false,
        })
}
fn lowered<T: PartialOrd>(old: Option<T>, new: Option<T>) -> bool {
    new.is_some_and(|n| old.is_none_or(|o| n < o))
}
fn identity(schema: &Value) -> Check<(&str, u64)> {
    let id = schema["$id"].as_str().ok_or("missing schema $id")?;
    let (name, major) = id.rsplit_once('/').ok_or("missing major version")?;
    Ok((name, major.parse().map_err(|_| "invalid major version")?))
}
fn shapes(root: &Value) -> Check<BTreeMap<String, Shape>> {
    let mut shapes = BTreeMap::new();
    visit(root, root, "", &mut shapes, &mut BTreeMap::new())?;
    let redirects: Vec<_> = shapes
        .iter()
        .filter_map(|(path, s)| Some((path.clone(), s.redirect.clone()?, s.nodes.clone())))
        .collect();
    for (path, (target, reference), nodes) in redirects {
        // Replay the whole path, including nullable branches before the backedge.
        let mut shape = shapes
            .get(&target)
            .and_then(|s| s.referents.get(&reference))
            .ok_or("missing recursive target")?
            .clone();
        shape.redirect = None;
        shapes.insert(path.clone(), shape);
        for node in nodes {
            collect(&node, &path, &mut shapes)?;
        }
    }
    Ok(shapes)
}
fn visit(
    root: &Value,
    node: &Value,
    path: &str,
    shapes: &mut BTreeMap<String, Shape>,
    refs: &mut BTreeMap<String, String>,
) -> Check<()> {
    if let Some(boolean) = node.as_bool() {
        shapes.entry(path.into()).or_default().impossible = !boolean;
        return Ok(());
    }
    if let Some(negated) = node.get("not") {
        let object = node.as_object().ok_or("invalid negation")?;
        if negated.as_object().is_none_or(|value| !value.is_empty())
            || object
                .keys()
                .any(|key| !["not", "title", "description"].contains(&key.as_str()))
        {
            return Err("unsupported negation".into());
        }
        shapes.entry(path.into()).or_default().impossible = true;
        return Ok(());
    }
    let object = node
        .as_object()
        .ok_or_else(|| format!("unsupported schema at {path}"))?;
    validate_keywords(object, path)?;
    if let Some(reference) = node.get("$ref") {
        let reference = reference.as_str().ok_or("invalid reference")?;
        let pointer = reference
            .strip_prefix("#/$defs/")
            .map(|_| &reference[1..])
            .ok_or_else(|| format!("unsupported reference {reference}"))?;
        if let Some(target) = refs.get(reference) {
            if target == path {
                return Err(format!("nonproductive reference {reference}"));
            }
            shapes.entry(path.into()).or_default().redirect =
                Some((target.clone(), reference.into()));
            return collect(node, path, shapes);
        }
        let prior = shapes.remove(path).unwrap_or_default();
        refs.insert(reference.into(), path.into());
        let target = root
            .pointer(pointer)
            .ok_or_else(|| format!("unresolved {reference}"))?;
        visit(root, target, path, shapes, refs)?;
        refs.remove(reference);
        let shape = shapes.get_mut(path).ok_or("missing reference target")?;
        let target = shape.clone();
        shape.referents.insert(reference.into(), target);
        for node in prior.nodes {
            collect(&node, path, shapes)?;
        }
    }
    if let Some(branches) = node.get("anyOf")
        && selectors(node)?.is_none()
    {
        if let Ok(branches) = optional_branches(branches) {
            for branch in branches {
                visit(root, branch, path, shapes, refs)?;
            }
        } else {
            for (kind, branch) in scalars::branches(root, branches)
                .map_err(|error| format!("{error} at {path}: {branches}"))?
            {
                descend(root, branch, path, &format!("@type={kind}"), shapes, refs)?;
            }
        }
    }
    if let Some(branches) = node.get("oneOf") {
        for (tag, branch) in tagged_branches(branches)? {
            descend(root, branch, path, &format!("@{tag}"), shapes, refs)?;
        }
    }
    collect(node, path, shapes)?;
    visit_children(root, node, path, shapes, refs)
}
fn visit_children(
    root: &Value,
    node: &Value,
    path: &str,
    shapes: &mut BTreeMap<String, Shape>,
    refs: &mut BTreeMap<String, String>,
) -> Check<()> {
    if let Some(patterns) = node.get("patternProperties") {
        for (pattern, child) in patterns.as_object().ok_or("invalid patternProperties")? {
            let key = pattern.replace('~', "~0").replace('/', "~1");
            let edge = format!("{{pattern={key}}}");
            descend(root, child, path, &edge, shapes, refs)?;
        }
    }
    for (field, suffix) in [
        ("additionalProperties", "{value}"),
        ("propertyNames", "{key}"),
    ] {
        if let Some(child) = node
            .get(field)
            .filter(|value| field == "propertyNames" || !value.is_boolean())
        {
            descend(root, child, path, suffix, shapes, refs)?;
        }
    }
    if let Some(items) = node.get("items") {
        descend(root, items, path, "[]", shapes, refs)?;
    }
    if let Some(properties) = node.get("properties") {
        for (name, child) in properties.as_object().ok_or("invalid properties")? {
            let name = name.replace('~', "~0").replace('/', "~1");
            descend(root, child, path, &name, shapes, refs)?;
        }
    }
    Ok(())
}
fn descend(
    root: &Value,
    node: &Value,
    path: &str,
    edge: &str,
    shapes: &mut BTreeMap<String, Shape>,
    refs: &mut BTreeMap<String, String>,
) -> Check<()> {
    let child = format!("{path}/{edge}");
    shapes
        .entry(path.into())
        .or_default()
        .children
        .insert(edge.into(), child.clone());
    visit(root, node, &child, shapes, refs)
}
fn collect(node: &Value, path: &str, shapes: &mut BTreeMap<String, Shape>) -> Check<()> {
    let shape = shapes.entry(path.into()).or_default();
    shape.nodes.push(node.clone());
    if shape.redirect.is_some() {
        return Ok(());
    }
    if let Some(selectors) = selectors(node)? {
        shape.selectors = Some(selectors);
    }
    if let Some(kind) = node.get("type") {
        let kinds = kind
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![kind.clone()]);
        for kind in kinds {
            let kind = kind.as_str().ok_or("invalid type")?;
            if !"string object null integer array boolean number"
                .split_whitespace()
                .any(|t| t == kind)
            {
                return Err(format!("unsupported type {kind} at {path}"));
            }
            shape.types.insert(kind.into());
        }
    }
    if let Some(unique) = node.get("uniqueItems") {
        shape.unique_items |= unique.as_bool().ok_or("invalid uniqueItems")?;
    }
    if let Some(pattern) = node.get("pattern") {
        shape.pattern = Some(pattern.as_str().ok_or("invalid pattern")?.into());
    }
    if let Some(minimum) = node.get("minimum") {
        shape.minimum = Some(bound(node, minimum)?);
    }
    if let Some(maximum) = node.get("maximum") {
        shape.maximum = Some(bound(node, maximum)?);
    }
    if let Some(maximum) = node.get("maxLength") {
        shape.maximum_length = Some(maximum.as_u64().ok_or("invalid maxLength")?);
    }
    collect_format(node, shape)?;
    shape.open_enum |= node.get("x-extensible-enum").is_some();
    shape.enum_assertion |= node.get("enum").is_some() || node.get("const").is_some();
    if shape.open_enum && shape.enum_assertion {
        return Err(format!("closed assertion on extensible enum at {path}"));
    }
    if let Some(values) = node.get("enum").or_else(|| node.get("x-extensible-enum")) {
        shape.values = Some(
            values
                .as_array()
                .ok_or("invalid enum")?
                .iter()
                .map(Value::to_string)
                .collect(),
        );
    }
    if let Some(value) = node.get("const") {
        if !value.is_string() {
            return Err("unsupported non-string const".into());
        }
        shape.values = Some(BTreeSet::from([value.to_string()]));
    }
    if let Some(required) = node.get("required") {
        for name in required.as_array().ok_or("invalid required")? {
            let name = name.as_str().ok_or("invalid required name")?;
            shape.required.insert(name.into());
        }
    }
    if let Some(bound) = node.get("minItems") {
        shape.minimum_items = bound.as_u64().ok_or("invalid minItems")?;
    }
    if let Some(bound) = node.get("maxItems") {
        shape.maximum_items = Some(bound.as_u64().ok_or("invalid maxItems")?);
    }
    if let Some(minimum) = node.get("minLength") {
        shape.minimum_length = minimum.as_u64().ok_or("invalid minLength")?;
    }
    collect_map(node, shape)
}
fn collect_format(node: &Value, shape: &mut Shape) -> Check<()> {
    if node.get("format").is_some_and(|format| {
        format != "uint32"
            && format != "uint64"
            && format != "int64"
            && format != "double"
            && format != "markdown"
            && !format
                .as_str()
                .is_some_and(|format| STRING_FORMATS.contains(&format))
    }) {
        return Err("unsupported format".into());
    }
    if let Some(format) = node.get("format").and_then(Value::as_str)
        && STRING_FORMATS.contains(&format)
    {
        if !shape.types.contains("string") {
            return Err("string format requires a string schema".into());
        }
        shape.string_format = Some(format.into());
    }
    shape.markdown |= node.get("format").is_some_and(|v| v == "markdown");
    Ok(())
}
fn collect_map(node: &Value, shape: &mut Shape) -> Check<()> {
    if let Some(additional) = node.get("additionalProperties") {
        shape.closed = additional == &Value::Bool(false);
        shape.map_values = additional.is_object();
    }
    shape.map_keys = node.get("propertyNames").is_some();
    if let Some(patterns) = node.get("patternProperties") {
        shape.map_patterns = patterns
            .as_object()
            .ok_or("invalid patternProperties")?
            .keys()
            .cloned()
            .collect();
    }
    Ok(())
}
// The only object-level union permits at least one named selector.
pub(super) fn selectors(node: &Value) -> Check<Option<Vec<BTreeSet<String>>>> {
    if node["type"] != "object" || node.get("anyOf").is_none() {
        return Ok(None);
    }
    let properties = node["properties"]
        .as_object()
        .ok_or("missing selector properties")?;
    let branches = node["anyOf"].as_array().ok_or("invalid selector anyOf")?;
    if branches.is_empty() {
        return Err("empty selector anyOf".into());
    }
    let mut result = Vec::new();
    for branch in branches {
        let object = branch.as_object().ok_or("invalid selector branch")?;
        if object.len() != 1 {
            return Err("selector branches contain only required".into());
        }
        let names = branch["required"]
            .as_array()
            .ok_or("missing selector required")?;
        let mut required = BTreeSet::new();
        for name in names {
            let name = name.as_str().ok_or("invalid selector name")?;
            if !properties.contains_key(name) || !required.insert(name.to_owned()) {
                return Err("unknown or repeated selector".into());
            }
        }
        if required.is_empty() || result.contains(&required) {
            return Err("empty or repeated selector branch".into());
        }
        result.push(required);
    }
    Ok(Some(result))
}
// Nullable generated references and strings remain distinct from tagged unions.
fn optional_branches(branches: &Value) -> Check<&[Value]> {
    let branches = branches.as_array().ok_or("invalid anyOf")?;
    if branches.len() != 2
        || branches
            .iter()
            .filter(|branch| **branch == serde_json::json!({"type":"null"}))
            .count()
            != 1
    {
        return Err("unsupported anyOf: expected one value and null".into());
    }
    let value = branches
        .iter()
        .find(|branch| branch["type"] != "null")
        .ok_or("missing optional value")?;
    if value.get("$ref").is_none() && value["type"] != "string" {
        return Err("unsupported anyOf value".into());
    }
    Ok(branches)
}

// These unions are internally tagged objects. Branch identity includes both
// discriminator field and value, so another branch cannot hide a removal.
fn tagged_branches(branches: &Value) -> Check<Vec<(String, &Value)>> {
    let mut result = Vec::new();
    let mut tags = BTreeSet::new();
    let mut discriminator = None;
    for branch in branches.as_array().ok_or("invalid oneOf")? {
        if branch["type"] != "object" {
            return Err("unsupported oneOf branch".into());
        }
        let properties = branch["properties"]
            .as_object()
            .ok_or("missing tagged properties")?;
        let candidates: Vec<_> = properties
            .iter()
            .filter_map(|(key, value)| {
                value
                    .get("const")
                    .and_then(Value::as_str)
                    .map(|value| (key, value))
            })
            .collect();
        let [(field, value)] = candidates.as_slice() else {
            return Err("expected one discriminator".into());
        };
        if !branch["required"]
            .as_array()
            .is_some_and(|required| required.contains(&Value::String((*field).clone())))
        {
            return Err("optional discriminator".into());
        }
        if discriminator.is_some_and(|old| old != field.as_str()) {
            return Err("inconsistent discriminator".into());
        }
        discriminator = Some(field.as_str());
        let tag = format!("{field}={value}");
        if !tags.insert(tag.clone()) {
            return Err("duplicate discriminator".into());
        }
        result.push((tag, branch));
    }
    if result.is_empty() {
        return Err("empty tagged union".into());
    }
    Ok(result)
}

const ANNOTATIONS: &str = concat!(
    "$schema $id $defs title description default x-enum-descriptions x-extensible-enum ",
    "x-positional x-trailing-section x-comma-separated x-cli-flags x-reply-query-titles x-identifier"
);
fn validate_keywords(object: &serde_json::Map<String, Value>, path: &str) -> Check<()> {
    for key in object.keys() {
        if !concat!(
            "$ref type enum properties required additionalProperties propertyNames patternProperties ",
            "anyOf oneOf const items minLength maxLength pattern minimum maximum format ",
            "minItems maxItems uniqueItems"
        ).split_whitespace().chain(ANNOTATIONS.split_whitespace()).any(|allowed| allowed == key) {
            return Err(format!("unsupported keyword {key} at {path}"));
        }
    }
    super::annotations::validate(object)?;
    // Assertion siblings would intersect, but Shape stores one constraint per path.
    for composition in ["$ref", "anyOf", "oneOf"] {
        if object.contains_key(composition)
            && !(composition == "anyOf" && selectors(&Value::Object(object.clone()))?.is_some())
            && object.keys().any(|key| {
                key != composition
                    && !(key == "format"
                        && object
                            .get(key)
                            .and_then(Value::as_str)
                            .is_some_and(|format| STRING_FORMATS.contains(&format)))
                    && !ANNOTATIONS.split_whitespace().any(|allowed| allowed == key)
            })
        {
            return Err(format!(
                "unsupported assertion sibling of {composition} at {path}"
            ));
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Bound {
    Integer(i128),
    Decimal(f64),
}
fn bound(node: &Value, value: &Value) -> Check<Bound> {
    if node["type"] == "integer" {
        return value
            .as_i64()
            .map(i128::from)
            .or_else(|| value.as_u64().map(i128::from))
            .map(Bound::Integer)
            .ok_or_else(|| "unsupported integer bound".into());
    }
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .map(Bound::Decimal)
        .ok_or_else(|| "unsupported decimal bound".into())
}
