//! Only object shape and declared keys belong to the typed adapter.
use serde_json::Value;
pub(super) fn valid(schema: &Value, value: &Value) -> bool {
    value.as_object().is_some_and(|fields| {
        fields
            .keys()
            .all(|key| schema.get("properties").and_then(|p| p.get(key)).is_some())
    })
}
