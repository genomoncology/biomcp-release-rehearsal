//! Prepare retained native replies for the cut-content save fixture.
use super::launch::fixtures;
use serde_json::Value;
use std::fs;

pub(super) fn cut_reply(request: &str) -> Result<String, String> {
    if request.contains(".xlsx?") {
        return Ok("PK\x03".into());
    }
    let target = request.split_whitespace().nth(1).ok_or("request target")?;
    for file in fs::read_dir(fixtures().join("replies/article/supplements/live/pmc-oa"))
        .map_err(|e| e.to_string())?
    {
        let path = file.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|v| v != "json") {
            continue;
        }
        let record: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if record
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| url.ends_with(target))
        {
            let body = record.pointer("/exchanges/0/body/value").ok_or("body")?;
            return Ok(body
                .as_str()
                .map_or_else(|| body.to_string(), str::to_owned));
        }
    }
    Err(format!("unrecorded local request {target}"))
}
