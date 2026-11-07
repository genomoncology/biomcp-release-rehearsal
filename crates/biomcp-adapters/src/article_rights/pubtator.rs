//! PubTator passage text is checked before native replies reach the cache.
use super::{Licenses, is_abstract, licenses, permitted, pmid};
use biomcp_fetch::{BadReply, Leg};
use biomcp_types::Use;
use serde_json::Value;
/// Strip publisher passages, including fields ignored by the annotation mapper.
pub(crate) fn filter(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    if !leg.disabled_use(Use::PerRecord) {
        return Ok(body.to_vec());
    }
    let mut root: Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    let documents = root
        .get_mut("PubTator3")
        .and_then(Value::as_array_mut)
        .ok_or(BadReply)?;
    let ids: Vec<_> = documents
        .iter()
        .filter_map(|row| pmid(&row["id"]))
        .collect();
    let licenses = licenses(leg, &ids);
    pubtator_body(root, &licenses)
}
fn pubtator_body(mut root: Value, licenses: &Licenses) -> Result<Vec<u8>, BadReply> {
    for document in root
        .get_mut("PubTator3")
        .and_then(Value::as_array_mut)
        .ok_or(BadReply)?
    {
        let license = pmid(&document["id"]).and_then(|id| licenses.get(id));
        if let Some(passages) = document.get_mut("passages").and_then(Value::as_array_mut) {
            for passage in passages {
                let abstract_text = passage
                    .pointer("/infons/type")
                    .and_then(Value::as_str)
                    .is_some_and(is_abstract);
                if !permitted(license, abstract_text)
                    && let Some(passage) = passage.as_object_mut()
                {
                    passage.remove("text");
                    passage.remove("sentences");
                    // Annotation mentions are copied fragments of the same passage.
                    if let Some(annotations) =
                        passage.get_mut("annotations").and_then(Value::as_array_mut)
                    {
                        for annotation in annotations {
                            if let Some(annotation) = annotation.as_object_mut() {
                                annotation.remove("text");
                            }
                        }
                    }
                }
            }
        }
    }
    serde_json::to_vec(&root).map_err(|_| BadReply)
}
/// Search rows can contain provider text omitted by the public mapping.
pub(crate) fn search(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    if !leg.disabled_use(Use::PerRecord) {
        return Ok(body.to_vec());
    }
    pubtator_search_body(body)
}
fn pubtator_search_body(body: &[u8]) -> Result<Vec<u8>, BadReply> {
    let mut root: Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    if let Some(rows) = root.get_mut("results").and_then(Value::as_array_mut) {
        for row in rows {
            if let Some(row) = row.as_object_mut() {
                row.retain(|key, _| key == "pmid");
            }
        }
    }
    serde_json::to_vec(&root).map_err(|_| BadReply)
}
