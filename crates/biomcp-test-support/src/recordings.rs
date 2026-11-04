//! Copy isolated recordings and opt positive exchanges into synthetic cache setup.
use serde_json::Value;
use std::{fs, path::Path};
type Check = Result<(), Box<dyn std::error::Error>>;
/// Copy transport-owned identities unchanged and mutate only response exchanges.
pub fn copy_capture(
    original: &Path,
    recordings: &Path,
    mut change: impl FnMut(&str, &mut Value),
) -> Check {
    for source in fs::read_dir(original)? {
        let source = source?;
        if !source.file_type()?.is_dir() {
            continue;
        }
        let name = source.file_name().to_string_lossy().into_owned();
        fs::create_dir_all(recordings.join(&name))?;
        for file in fs::read_dir(source.path())? {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }
            if file.path().extension().is_some_and(|v| v == "bin") {
                fs::copy(file.path(), recordings.join(&name).join(file.file_name()))?;
                continue;
            }
            if file.path().extension().is_none_or(|v| v != "json") {
                continue;
            }
            let mut record = serde_json::from_slice(&fs::read(file.path())?)?;
            change(&name, &mut record);
            fs::write(
                recordings.join(&name).join(file.file_name()),
                serde_json::to_vec(&record)?,
            )?;
        }
    }
    Ok(())
}

/// Assign a synthetic acquisition date to successful exchanges.
pub fn cache_acquisition(record: &mut Value, acquired: i64) {
    if let Some(exchanges) = record.get_mut("exchanges").and_then(Value::as_array_mut) {
        for exchange in exchanges {
            if exchange
                .get("status")
                .and_then(Value::as_u64)
                .is_some_and(|v| (200..300).contains(&v))
                && let Some(exchange) = exchange.as_object_mut()
            {
                exchange.entry("fetched").or_insert(Value::from(acquired));
            }
        }
    }
}
