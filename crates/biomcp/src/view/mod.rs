//! One structural view of serialized JSON and its schema presentation hints.
mod article;
mod escape;
mod help;
mod hints;
pub(crate) mod protein;
use biomcp_types::{Help, ResponseBody};
use escape::{escape, source_link};
use serde_json::Value;
use std::{collections::BTreeSet, io};

pub(crate) fn markdown(value: &Value) -> io::Result<Vec<u8>> {
    let identity = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let schema: Value = crate::schema::find(identity)
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or(Value::Null);
    let hint = hints::find(identity);
    let mut out = String::new();
    if let Some(note) = protein::note(value) {
        out.push_str(&note);
        out.push_str("\n\n");
    }
    if value.get("status").and_then(Value::as_str) == Some("absent") {
        for source in value
            .get("sources")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if source.get("state").and_then(Value::as_str) == Some("ok") {
                no_data(&source["source"], &mut out);
            }
        }
    }
    for source in value
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for notice in source
            .get("notices")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if notice.get("kind").and_then(Value::as_str) == Some("litvar-rsid-hint") {
                out.push_str(&scalar(&notice["message"]));
                out.push_str("\n\n");
            }
            if notice.get("kind").and_then(Value::as_str) == Some("pubmed-warning") {
                out.push_str("**PubMed warning ");
                out.push_str(&scalar(&notice["name"]));
                out.push_str(":** ");
                out.push_str(&scalar(&notice["phrases"]));
                out.push_str("\n\n");
            } else if notice.get("kind").and_then(Value::as_str) == Some("not-found-words") {
                out.push_str("**PubMed dropped supplied words:** ");
                out.push_str(&scalar(&notice["words"]));
                out.push_str("\n\n");
            }
        }
    }
    if value.get("error").is_some() {
        out.push_str("# Error\n\n");
    }
    let view = View {
        root: &schema,
        hint,
        source_links: false,
    };
    if identity == Help::SCHEMA {
        help::render(value, &mut out);
    } else {
        view.object(value, &schema, 2, &mut out);
    }
    Ok(out.into_bytes())
}
struct View<'a> {
    root: &'a Value,
    hint: Option<&'a hints::Hint>,
    source_links: bool,
}
impl View<'_> {
    fn object(&self, value: &Value, schema: &Value, depth: usize, out: &mut String) {
        let Some(fields) = value.as_object() else {
            return;
        };
        state_note(value, out);
        for (name, value) in fields {
            if name == "schema"
                || (name == "query"
                    && (value.is_null()
                        || value.as_object().is_some_and(serde_json::Map::is_empty)))
            {
                continue;
            }
            let node = property(schema, name, self.root).unwrap_or(&Value::Null);
            if (value.is_object() || value.is_array()) && nonfinite(value).is_none() {
                heading(name, depth, out);
                if typed_step(value, node, self.root, out) {
                    continue;
                } else if name == "next" {
                    steps(value, node, self.root, out);
                } else if !(name == "parts"
                    && self
                        .hint
                        .is_some_and(|hint| hint.schema == biomcp_types::ArticleCards::SCHEMA)
                    && article::parts(value, out))
                {
                    View {
                        root: self.root,
                        hint: self.hint,
                        source_links: depth == 2 && name == "sources",
                    }
                    .render(value, node, depth + 1, out);
                }
            } else if self.source_links && name == "terms_url" {
                out.push_str(&format!(
                    "**{}:** {}\n\n",
                    escape(name),
                    source_link(value.as_str().unwrap_or_default())
                ));
            } else {
                out.push_str(&format!("**{}:** {}\n\n", escape(name), scalar(value)));
            }
        }
    }
    fn render(&self, value: &Value, schema: &Value, depth: usize, out: &mut String) {
        match value {
            Value::Object(_) => self.object(value, schema, depth, out),
            Value::Array(items) => self.array(items, schema, depth, out),
            _ => {
                out.push_str(&scalar(value));
                out.push_str("\n\n");
            }
        }
    }
    fn array(&self, items: &[Value], schema: &Value, depth: usize, out: &mut String) {
        let item_schema = member(schema, "items", self.root).unwrap_or(&Value::Null);
        if !items.is_empty()
            && items
                .iter()
                .all(|value| projected_step(value, item_schema, self.root))
        {
            for item in items {
                typed_step(item, item_schema, self.root, out);
            }
            return;
        }
        if items.is_empty() {
            out.push_str("None\n\n");
            return;
        }
        if let Some(hint) = self.hint.filter(|h| !h.title.is_empty())
            && items.iter().any(|item| item.pointer(hint.title).is_some())
        {
            for item in items {
                let title = item.pointer(hint.title).or_else(|| item.get("input"));
                out.push_str(&format!("# {}\n\n", title.map_or_else(String::new, scalar)));
                self.object(item, item_schema, depth, out);
            }
        } else if items.iter().all(Value::is_object)
            && !items.iter().any(nested_records)
            && !contains_markdown(item_schema, self.root)
        {
            for item in items {
                table_notes(item, out);
            }
            table(
                items,
                self.hint.map_or(&[], |h| h.columns),
                self.source_links,
                out,
            );
        } else {
            for item in items {
                if item.is_object() {
                    self.object(item, item_schema, depth, out);
                } else {
                    out.push_str(&format!("- {}\n", scalar(item)));
                }
            }
            out.push('\n');
        }
    }
}
// Tables flatten nested objects, so visit their source states before flattening.
fn table_notes(value: &Value, out: &mut String) {
    state_note(value, out);
    if let Some(fields) = value.as_object() {
        for field in fields.values().filter(|field| field.is_object()) {
            table_notes(field, out);
        }
    }
}
fn no_data(source: &Value, out: &mut String) {
    out.push_str(&format!("{} returned no data.\n\n", scalar(source)));
}
fn state_note(value: &Value, out: &mut String) {
    let Some(source) = value.get("source").filter(|source| source.is_string()) else {
        return;
    };
    match value.get("state").and_then(Value::as_str) {
        Some("empty") => no_data(source, out),
        Some("unavailable" | "failed") => {
            if let Some(reason) = value.get("reason") {
                out.push_str(&format!(
                    "{} failed (reason: {}); resolve this reason and retry the request.\n\n",
                    scalar(source),
                    scalar(reason)
                ));
            }
        }
        _ => {}
    }
}
fn heading(name: &str, depth: usize, out: &mut String) {
    out.push_str(&format!(
        "{} {}\n\n",
        "#".repeat(depth.min(6)),
        escape(&label(name))
    ));
}
fn label(name: &str) -> String {
    let name = name.rsplit('/').next().unwrap_or(name).replace('_', " ");
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}
fn scalar(value: &Value) -> String {
    if let Some(label) = nonfinite(value) {
        return label.to_owned();
    }
    escape(
        &value
            .as_str()
            .map_or_else(|| value.to_string(), str::to_owned),
    )
}
// The native scalar union has exactly one field. Do not reinterpret source metadata objects.
fn nonfinite(value: &Value) -> Option<&'static str> {
    let object = value.as_object().filter(|object| object.len() == 1)?;
    match object.get("kind")?.as_str()? {
        "nan" => Some("NaN"),
        "positive-infinity" => Some("+Inf"),
        "negative-infinity" => Some("-Inf"),
        _ => None,
    }
}
fn leaf_paths(value: &Value, prefix: &str, fields: &mut BTreeSet<String>) {
    if nonfinite(value).is_some() {
        fields.insert(prefix.to_owned());
        return;
    }
    if let Some(object) = value.as_object().filter(|o| !o.is_empty()) {
        for (key, value) in object {
            let key = key.replace('~', "~0").replace('/', "~1");
            let path = if prefix.is_empty() {
                key
            } else {
                format!("{prefix}/{key}")
            };
            leaf_paths(value, &path, fields);
        }
    } else {
        fields.insert(prefix.to_owned());
    }
}
// Nested record collections need their own headings and tables, rather than
// escaped JSON inside one cell of their parent's table.
fn nested_records(value: &Value) -> bool {
    match value {
        Value::Object(fields) => fields.values().any(nested_records),
        Value::Array(items) => items.iter().any(|item| item.is_object() || item.is_array()),
        _ => false,
    }
}
fn table(items: &[Value], preferred: &[&str], source_links: bool, out: &mut String) {
    let mut fields = BTreeSet::new();
    for item in items {
        leaf_paths(item, "", &mut fields);
    }
    let mut columns: Vec<_> = preferred
        .iter()
        .copied()
        .filter(|k| fields.contains(*k))
        .collect();
    columns.extend(
        fields
            .iter()
            .map(String::as_str)
            .filter(|k| !preferred.contains(k)),
    );
    out.push_str("| ");
    out.push_str(
        &columns
            .iter()
            .map(|name| escape(&label(name)))
            .collect::<Vec<_>>()
            .join(" | "),
    );
    out.push_str(" |\n| ");
    out.push_str(&vec!["---"; columns.len()].join(" | "));
    out.push_str(" |\n");
    for item in items {
        out.push_str("| ");
        out.push_str(
            &columns
                .iter()
                .map(|name| {
                    item.pointer(&format!("/{name}"))
                        .map_or_else(String::new, |value| {
                            if source_links && *name == "terms_url" {
                                source_link(value.as_str().unwrap_or_default())
                            } else {
                                scalar(value)
                            }
                        })
                })
                .collect::<Vec<_>>()
                .join(" | "),
        );
        out.push_str(" |\n");
    }
    out.push('\n');
}
// Fenced command text retains shell syntax while making terminal controls visible.
fn controls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if escape::control(ch) {
            out.push_str(&escape::visible(ch));
        } else {
            out.push(ch);
        }
    }
    out
}
fn projected_step(value: &Value, schema: &Value, root: &Value) -> bool {
    value.get("name").is_some()
        && value.get("arguments").is_some()
        && crate::schema::steps::paths(root, schema, value)
            .iter()
            .any(String::is_empty)
}
fn typed_step(value: &Value, schema: &Value, root: &Value, out: &mut String) -> bool {
    if !projected_step(value, schema, root) {
        return false;
    }
    if let Some(kind) = value.get("kind") {
        out.push_str(&format!("**{}:**\n\n", scalar(kind)));
    }
    let call = serde_json::json!({"name":value["name"],"arguments":value["arguments"]}).to_string();
    let longest = call.split(|ch| ch != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(longest + 1));
    out.push_str(&format!("{fence}json\n{call}\n{fence}\n\n"));
    true
}

fn steps(value: &Value, schema: &Value, root: &Value, out: &mut String) {
    let item = member(schema, "items", root).unwrap_or(schema);
    for step in value.as_array().into_iter().flatten() {
        if typed_step(step, item, root, out) {
            continue;
        }
        let Some(args) = step.get("args").and_then(Value::as_array) else {
            continue;
        };
        if let Some(kind) = step.get("kind") {
            out.push_str(&format!("**{}:**\n\n", scalar(kind)));
        }
        let args = args
            .iter()
            .filter_map(Value::as_str)
            .map(controls)
            .map(|arg| {
                if arg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_./:-".contains(c))
                    && !arg.is_empty()
                {
                    arg
                } else {
                    format!("'{}'", arg.replace('\'', "'\\''"))
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let longest = args.split(|c| c != '`').map(str::len).max().unwrap_or(0);
        let fence = "`".repeat(3.max(longest + 1));
        out.push_str(&format!("{fence}sh\nbiomcp {args}\n{fence}\n\n"));
    }
}
fn resolve<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    node.get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| r.strip_prefix('#'))
        .and_then(|p| root.pointer(p))
        .map_or(node, |v| resolve(v, root))
}
fn member<'a>(node: &'a Value, name: &str, root: &'a Value) -> Option<&'a Value> {
    let node = resolve(node, root);
    node.get(name).or_else(|| {
        ["anyOf", "oneOf"]
            .into_iter()
            .filter_map(|k| node.get(k).and_then(Value::as_array))
            .flatten()
            .find_map(|n| member(n, name, root))
    })
}
fn alternatives(node: &Value) -> impl Iterator<Item = &Value> {
    ["anyOf", "oneOf"]
        .into_iter()
        .filter_map(|key| node.get(key).and_then(Value::as_array))
        .flatten()
}
fn property<'a>(node: &'a Value, name: &str, root: &'a Value) -> Option<&'a Value> {
    let node = resolve(node, root);
    node.get("properties")
        .and_then(|properties| properties.get(name))
        .or_else(|| alternatives(node).find_map(|branch| property(branch, name, root)))
}
fn contains_markdown(node: &Value, root: &Value) -> bool {
    markdown_in_graph(node, root, &mut Vec::new())
}
// Recursive schema definitions describe finite response values. Visit each schema
// node once so presentation discovery also terminates on recursive definitions.
fn markdown_in_graph<'a>(node: &'a Value, root: &'a Value, seen: &mut Vec<&'a Value>) -> bool {
    let node = resolve(node, root);
    if seen.iter().any(|previous| std::ptr::eq(*previous, node)) {
        return false;
    }
    seen.push(node);
    node.get("format").and_then(Value::as_str) == Some("markdown")
        || node
            .get("properties")
            .and_then(Value::as_object)
            .is_some_and(|properties| {
                properties
                    .values()
                    .any(|p| markdown_in_graph(p, root, seen))
            })
        || node
            .get("items")
            .is_some_and(|item| markdown_in_graph(item, root, seen))
        || alternatives(node).any(|branch| markdown_in_graph(branch, root, seen))
}
