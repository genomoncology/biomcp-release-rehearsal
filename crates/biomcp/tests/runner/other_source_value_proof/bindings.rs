//! Native request bindings remain strict even when a returned row identity is bad.
use super::super::{launch, schema::Registry};
use super::{Check, Mutation, NativeCase, Route, acquire_at_exit, native_case, save};
use serde_json::{Value, json};
use std::path::Path;
pub(super) fn mechanism(directory: &Path, registry: &Registry) -> Check {
    let case = native_case("015.json")?;
    let text = launch::fixture_text("fixtures/drug/native/015.json")?;
    let mut body: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let row = body
        .pointer_mut("/mechanisms/0")
        .ok_or("native mechanism")?;
    *row.get_mut("mec_id").ok_or("native mechanism identity")? = json!({"bad_shape":true});
    *row.get_mut("molecule_chembl_id")
        .ok_or("native molecule binding")? = json!("CHEMBL941");
    let mutation = Mutation {
        file: "015.json",
        pointer: "/mechanisms/0",
        field: "request-scope",
        absent: "",
        present: "",
    };
    let value = acquire_at_exit(&case, Some((&mutation, row.clone())), registry, 4)?;
    save(directory, "contradictory-scope-and-bad-identity", &value)?;
    if value.pointer("/sources/0/reason") != Some(&json!("bad-reply")) {
        return Err("contradictory native request scope was admitted".into());
    }
    Ok(())
}

pub(super) fn cell(directory: &Path, registry: &Registry) -> Check {
    let mut case = NativeCase {
        host: "www.ebi.ac.uk",
        root: "fixtures/cell-line/chembl",
        args: ["get", "cell-line", "CVCL_2119", "chembl"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        requests: vec![
            Route {
                file: "CVCL_2119.json".into(),
                request: concat!(
                    "GET /chembl/api/data/cell_line.json?cellosaurus_id=CVCL_2119",
                    "&limit=5&offset=0 HTTP/1.1"
                )
                .into(),
            },
            Route {
                file: "status.json".into(),
                request: "GET /chembl/api/data/status.json HTTP/1.1".into(),
            },
        ],
    };
    let positive = super::acquire(&case, None, registry)?;
    if positive.pointer("/result/items/items/0/id") != Some(&json!("CHEMBL3706573")) {
        return Err("native ChEMBL cell positive control lost identity".into());
    }
    save(directory, "cell-binding-positive", &positive)?;
    let text = launch::fixture_text("fixtures/cell-line/chembl/CVCL_2119.json")?;
    let mut body: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let row = body.pointer_mut("/cell_lines/0").ok_or("native cell")?;
    *row.get_mut("cell_chembl_id")
        .ok_or("native cell identity")? = json!({"bad_shape":true});
    *row.get_mut("cellosaurus_id").ok_or("native cell binding")? = json!("CVCL_0004");
    case.requests.retain(|route| route.file == "CVCL_2119.json");
    let mutation = Mutation {
        file: "CVCL_2119.json",
        pointer: "/cell_lines/0",
        field: "cell-request-scope",
        absent: "",
        present: "",
    };
    let value = acquire_at_exit(&case, Some((&mutation, row.clone())), registry, 4)?;
    save(
        directory,
        "cell-contradictory-scope-and-bad-identity",
        &value,
    )?;
    if !value
        .get("sources")
        .and_then(Value::as_array)
        .is_some_and(|sources| {
            sources
                .iter()
                .any(|source| source.get("reason") == Some(&json!("bad-reply")))
        })
    {
        return Err("contradictory native cell request scope was admitted".into());
    }
    Ok(())
}
