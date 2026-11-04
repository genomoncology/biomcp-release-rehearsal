//! Original native transport declarations for the MyChem and Cellosaurus controls.
use super::{Check, NativeCase, Route};
use serde_json::Value;
pub(super) fn case(file: &str) -> Check<Option<NativeCase>> {
    match file {
        "008.json" => mychem().map(Some),
        "native-projected-k562.json" => cellosaurus().map(Some),
        _ => Ok(None),
    }
}
fn mychem() -> Check<NativeCase> {
    let text = crate::runner::launch::fixture_text("fixtures/drug/mychem/plan.json")?;
    let plan: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let request = plan
        .get(7)
        .and_then(|row| row.get("request"))
        .and_then(Value::as_str)
        .ok_or("missing original MyChem request")?;
    Ok(NativeCase {
        host: "mychem.info",
        root: "fixtures/drug/mychem",
        args: ["get", "drug", "CHEMBL3989989", "--source", "mychem"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        requests: vec![Route {
            file: "008.json".into(),
            request: request.into(),
        }],
    })
}
fn cellosaurus() -> Check<NativeCase> {
    let text = crate::runner::launch::fixture_text("fixtures/cell-line/native/requests.json")?;
    let plan: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut requests = Vec::new();
    for name in [
        "native-projected-k562",
        "portable-cellosaurus-cellosaurus-release",
    ] {
        let row = plan
            .get(name)
            .ok_or("missing original Cellosaurus declaration")?;
        let file = row
            .get("file")
            .and_then(Value::as_str)
            .ok_or("native file")?;
        let target = row
            .get("target")
            .and_then(Value::as_str)
            .ok_or("native target")?;
        requests.push(Route {
            file: file.into(),
            request: format!("GET {target} HTTP/1.1"),
        });
    }
    Ok(NativeCase {
        host: "api.cellosaurus.org",
        root: "fixtures/cell-line/native",
        args: ["get", "cell-line", "CVCL_0004"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        requests,
    })
}
