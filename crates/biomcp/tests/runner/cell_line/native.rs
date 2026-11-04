use super::{Environment, Value, json, parity, path, recorded_stub, setup};
pub(super) fn run(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    search(&env, measure)?;
    cards(&env, measure)?;
    reverse(&env, measure)?;
    let requests = stub.finish()?;
    if requests.iter().any(|(_, r)| r.contains("sort=")) {
        return Err("native candidate order changed".into());
    }
    Ok(())
}
fn search(env: &Environment, measure: bool) -> Result<(), String> {
    for (name, query, ids, offset) in [
        ("MOLM13", "MOLM13", vec!["CVCL_2119"], 0),
        ("MV4-semicolon", "MV4;11", vec!["CVCL_0064"], 0),
        (
            "NB4-first",
            "NB4",
            vec![
                "CVCL_8811",
                "CVCL_U085",
                "CVCL_U087",
                "CVCL_U088",
                "CVCL_U081",
            ],
            0,
        ),
        (
            "NB4-second",
            "NB4",
            vec![
                "CVCL_F5R0",
                "CVCL_U080",
                "CVCL_U084",
                "CVCL_U082",
                "CVCL_VV77",
            ],
            5,
        ),
        ("KG1", "KG1", vec!["CVCL_E3VV", "CVCL_0374", "CVCL_UD72"], 0),
        ("empty", "zzzx0196nomatch", vec![], 0),
    ] {
        let mut words = vec!["search", "cell-line", query];
        let offset_text = offset.to_string();
        if offset > 0 {
            words.extend(["--offset", &offset_text]);
        }
        let value = parity(
            &format!("native/search/{name}"),
            env,
            &words,
            if ids.is_empty() { 1 } else { 0 },
            measure,
        )?;
        headers(&value, name, query, &ids, offset)?;
        if name == "KG1" {
            for i in [0, 2] {
                path(
                    &value,
                    &format!("/result/items/{i}/species/items/0/id"),
                    json!("10090"),
                )?;
            }
        }
    }
    Ok(())
}
fn cards(env: &Environment, measure: bool) -> Result<(), String> {
    for (id, name) in [
        ("CVCL_2119", "MOLM-13"),
        ("CVCL_0004", "K-562"),
        ("CVCL_0005", "NB4"),
        ("CVCL_0002", "HL-60"),
        ("CVCL_A794", "HL-60(TB)"),
        ("CVCL_0007", "U-937"),
        ("CVCL_ZZZZ", ""),
    ] {
        let value = parity(
            &format!("native/card/{id}"),
            env,
            &["get", "cell-line", id],
            if name.is_empty() { 1 } else { 0 },
            measure,
        )?;
        if name.is_empty() {
            path(&value, "/result/status", json!("absent"))?;
            continue;
        }
        path(&value, "/result/header/accession", json!(id))?;
        path(&value, "/result/header/name", json!(name))?;
        path(&value, "/result/release/version", json!("57.0"))?;
        if value.pointer("/result/dates/version").is_none()
            || value.pointer("/result/fetched").is_none()
        {
            return Err("entry/retrieval provenance missing".into());
        }
        let actions = value
            .get("next")
            .and_then(Value::as_array)
            .ok_or("actions")?;
        if actions.len() > 5 {
            return Err("card action budget exceeded".into());
        }
        for (step, section) in actions
            .iter()
            .zip(["names", "variants", "xrefs", "chembl", "identity"])
        {
            path(step, "/args", json!(["get", "cell-line", id, section]))?;
        }
        if id == "CVCL_0007" {
            path(
                &value,
                "/result/warnings/items/0/text",
                json!(
                    "Partially contaminated. Some ATCC stocks were cross-contaminated by K-562 initially \
                        (PubMed=7759961). The problem was corrected and subsequent stocks have been confirmed to carry \
                        only U-937."
                ),
            )?;
        }
    }
    Ok(())
}
fn reverse(env: &Environment, measure: bool) -> Result<(), String> {
    for id in ["DepMap:ACH-000362", "ChEMBL:CHEMBL3706573"] {
        let value = parity(
            &format!("native/reverse/{id}"),
            env,
            &["get", "cell-line", id],
            0,
            measure,
        )?;
        path(&value, "/result/header/accession", json!("CVCL_2119"))?;
        path(&value, "/result/input", json!(id))?;
        path(&value, "/result/candidates", json!([]))?;
    }
    Ok(())
}

fn headers(
    value: &Value,
    name: &str,
    query: &str,
    ids: &[&str],
    offset: u32,
) -> Result<(), String> {
    let items = value
        .pointer("/result/items")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let actual: Vec<_> = items
        .iter()
        .filter_map(|r| r.get("accession").and_then(Value::as_str))
        .collect();
    if actual != ids || value.get("page").is_some() {
        return Err(format!("{name}: source order or unknown total changed"));
    }
    for (row, id) in items.iter().zip(ids) {
        path(row, "/accession", json!(id))?;
        if row.get("name").is_none() || row.get("category").is_none() {
            return Err("header facts missing".into());
        }
    }
    let steps = value
        .get("next")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if steps.len() != ids.len() + usize::from(ids.len() == 5) {
        return Err(format!("{name}: candidates/page actions lost"));
    }
    if ids.len() == 5 {
        path(
            steps.last().ok_or("page")?,
            "/args",
            json!([
                "search",
                "cell-line",
                query,
                "--offset",
                (offset + 5).to_string()
            ]),
        )?;
    }
    Ok(())
}

tool_only! {
pub(super) fn remaining() -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    cards(&env, true)?;
    reverse(&env, true)?;
    stub.finish()?;
    Ok(())
}
}

pub(super) fn canonical(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    let value = parity(
        "native/search/accession-CVCL_0005",
        &env,
        &["search", "cell-line", "CVCL_0005"],
        0,
        measure,
    )?;
    path(&value, "/result/items/0/accession", json!("CVCL_0005"))?;
    path(&value, "/result/items/0/name", json!("NB4"))?;
    let requests = stub.finish()?;
    if requests.len() != 2
        || !requests
            .iter()
            .any(|(_, r)| r.contains("q=ac%3A%22CVCL_0005%22") && r.contains("start=0"))
    {
        return Err("canonical accession did not use its checked native query".into());
    }
    Ok(())
}
