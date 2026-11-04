//! Keep one outline table and mark only titles repeated on consecutive parts.
use serde_json::Value;

pub(super) fn parts(value: &Value, out: &mut String) -> bool {
    let Some(items) = value.as_array() else {
        return false;
    };
    // A source title equal to the marker must remain literal.
    if items
        .iter()
        .any(|item| item.get("title").and_then(Value::as_str) == Some("^"))
    {
        return false;
    }
    let mut rows = items.clone();
    let mut previous = None;
    let mut repeated = false;
    for row in &mut rows {
        let title = row.get("title").and_then(Value::as_str).map(str::to_owned);
        if title.is_some()
            && title == previous
            && let Some(fields) = row.as_object_mut()
        {
            fields.insert("title".into(), Value::String("^".into()));
            repeated = true;
        }
        previous = title;
    }
    if !repeated {
        return false;
    }
    let mut compact = String::new();
    super::table(&rows, &[], false, &mut compact);
    compact.push_str("^ repeats the preceding title.\n\n");
    let mut ordinary = String::new();
    super::table(items, &[], false, &mut ordinary);
    if compact.len() >= ordinary.len() {
        return false;
    }
    out.push_str(&compact);
    true
}
