//! Abstract refusal metadata follows the same paper through native and saved replies.
use biodata::{License, Pmid};
use biomcp_fetch::BadReply;
use serde_json::{Map, Value};

const MARKER: &str = "biomcpAbstractWithheld";
pub(super) fn filter(
    object: &mut Map<String, Value>,
    own: Option<Pmid>,
    license: Option<License>,
) -> Result<(), BadReply> {
    let provided = ["abstract", "abstract_text"].iter().any(|key| {
        object
            .get(*key)
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty())
    });
    if provided {
        object.remove(MARKER);
    }
    if super::permitted(license, true) {
        return Ok(());
    }
    object.remove("abstract");
    object.remove("abstract_text");
    if provided && let Some(id) = own {
        object.insert(
            MARKER.into(),
            serde_json::to_value(super::withheld(id)).map_err(|_| BadReply)?,
        );
    }
    Ok(())
}
/// Author snapshots wrap the pinned detail value; keep diagnostics on that wrapper.
pub(super) fn lift(object: &mut Map<String, Value>) {
    if let Some(notice) = object
        .get_mut("detail")
        .and_then(Value::as_object_mut)
        .and_then(|detail| detail.remove(MARKER))
    {
        object.insert("text_withheld".into(), notice);
    }
}
