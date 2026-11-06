//! Match schema-owned choices once before checked query decoding.
use serde_json::{Map, Value};

pub(crate) const ENUM_CASE_GUIDANCE: &str =
    "Listed choices accept any letter case; replies use their listed spelling.";

pub(super) fn matching<'a>(value: &Value, choices: &'a [Value]) -> Option<&'a Value> {
    let text = value.as_str()?;
    choices.iter().find(|choice| *choice == value).or_else(|| {
        choices.iter().find(|choice| {
            choice
                .as_str()
                .is_some_and(|canonical| same_letters(canonical, text))
        })
    })
}

pub(super) fn canonicalize(fields: &mut Map<String, Value>, schema: &Value) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    let context = fields.clone();
    for (name, value) in fields {
        let Some(property) = properties.get(name) else {
            continue;
        };
        let node = super::validation::resolve(schema, property);
        let Some(choices) =
            scoped(node, &context).or_else(|| node.get("enum").and_then(Value::as_array))
        else {
            continue;
        };
        if let Some(values) = value.as_array_mut() {
            for value in values {
                if let Some(canonical) = matching(value, choices) {
                    *value = canonical.clone();
                }
            }
        } else if let Some(canonical) = matching(value, choices) {
            *value = canonical.clone();
        }
    }
}

// A union may share spellings across providers while retaining different canonical case.
fn scoped<'a>(node: &'a Value, fields: &Map<String, Value>) -> Option<&'a Vec<Value>> {
    let scope = node.get("x-enum-by-field")?;
    let field = scope.get("field")?.as_str()?;
    let selected = fields
        .get(field)
        .or_else(|| scope.get("default"))?
        .as_str()?;
    scope
        .get("choices")?
        .as_object()?
        .iter()
        .find(|(key, _)| same_letters(key, selected))?
        .1
        .as_array()
}

fn same_letters(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right) || left.to_lowercase() == right.to_lowercase()
}
