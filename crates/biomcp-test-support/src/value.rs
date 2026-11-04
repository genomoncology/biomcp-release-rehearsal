//! Shared recorded JSON mutation.
//! Features trial-search and trial-get: malformed nonidentity fields retain their sibling facts.
use serde_json::Value;

/// Replace the value at a recorded JSON pointer.
pub fn replace(record: &mut Value, path: &str, invalid: &Value) {
    if let Some((parent, name)) = path.rsplit_once('/')
        && let Some(object) = record.pointer_mut(parent).and_then(Value::as_object_mut)
    {
        object.insert(name.into(), invalid.clone());
    }
}
