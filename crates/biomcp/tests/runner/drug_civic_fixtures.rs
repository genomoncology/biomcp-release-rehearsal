//! Original CC0 replies from the recorded source proof, projected only for declared local requests.
use serde_json::Value;
fn fixture(index: usize) -> Result<Value, String> {
    let body = match index {
        21 => &crate::runner::launch::fixture_text("fixtures/drug/civic/021.body")?,
        22 => &crate::runner::launch::fixture_text("fixtures/drug/civic/022.body")?,
        23 => &crate::runner::launch::fixture_text("fixtures/drug/civic/023.body")?,
        24 => &crate::runner::launch::fixture_text("fixtures/drug/civic/024.body")?,
        25 => &crate::runner::launch::fixture_text("fixtures/drug/civic/025.body")?,
        26 => &crate::runner::launch::fixture_text("fixtures/drug/civic/026.body")?,
        27 => &crate::runner::launch::fixture_text("fixtures/drug/civic/027.body")?,
        28 => &crate::runner::launch::fixture_text("fixtures/drug/civic/028.body")?,
        29 => &crate::runner::launch::fixture_text("fixtures/drug/civic/029.body")?,
        30 => &crate::runner::launch::fixture_text("fixtures/drug/civic/030.body")?,
        31 => &crate::runner::launch::fixture_text("fixtures/drug/civic/031.body")?,
        _ => return Err("undeclared source fixture".into()),
    };
    serde_json::from_str(body).map_err(|e| e.to_string())
}
pub(super) fn reply(request: &str) -> Result<String, String> {
    let (_, body) = request.split_once("\r\n\r\n").ok_or("missing POST body")?;
    let body: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let q = at(&body, "/query").as_str().ok_or("missing GraphQL")?;
    let v = at(&body, "/variables");
    check_request(q, v)?;
    // Synthetic connection tests candidate separation using identities observed in the source proof.
    if at(v, "/q").as_str() == Some("synthetic distinct therapy identities") {
        return serde_json::to_string(&serde_json::json!({"data":{"therapies":{
        "totalCount":2,"pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[
            {"id":84,"name":"Trastuzumab","link":"/therapies/84"},
            {"id":4287,"name":"Trastuzumab Deruxtecan","link":"/therapies/4287"}
        ]}}}))
        .map_err(|e| e.to_string());
    }
    let index = if let Some(text) = at(v, "/q").as_str() {
        match (text, at(v, "/after").as_str()) {
            ("imatinib", None) => 24,
            ("trastuzumab", None) => 25,
            ("ivosidenib", None) => 26,
            ("fam-trastuzumab deruxtecan", None) => 27,
            ("trastuzumab", Some("MQ")) => 28,
            _ => return Err(format!("undeclared candidate request {body}")),
        }
    } else if q.contains("molecularProfileId:") {
        match (
            q.contains("molecularProfileId:65"),
            at(v, "/after").as_str(),
        ) {
            (_, None) => 29,
            (false, Some("MQ")) => 30,
            (true, Some("MQ")) => 31,
            _ => return Err(format!("undeclared claim request {body}")),
        }
    } else {
        match at(v, "/id").as_u64() {
            Some(5) => 21,
            Some(84) => 22,
            Some(454) => 23,
            _ => return Err(format!("undeclared identity {body}")),
        }
    };
    let mut value = fixture(index)?;
    if at(v, "/id").is_number() {
        if at(&value, "/data/therapy").is_null() {
            let therapy = fixture(21)?
                .pointer("/data/therapy")
                .ok_or("retained therapy")?
                .clone();
            value
                .get_mut("data")
                .and_then(Value::as_object_mut)
                .ok_or("data")?
                .insert("therapy".into(), therapy);
        }
        for field in ["evidenceItems", "assertions"] {
            if !q.contains(&format!("{field}(")) {
                value
                    .get_mut("data")
                    .and_then(Value::as_object_mut)
                    .ok_or("data")?
                    .remove(field);
            }
        }
    }
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

fn at<'a>(value: &'a Value, path: &str) -> &'a Value {
    value.pointer(path).unwrap_or(&Value::Null)
}

fn check_request(query: &str, variables: &Value) -> Result<(), String> {
    if let Some(text) = at(variables, "/q").as_str() {
        let count = if text == "synthetic distinct therapy identities" {
            2
        } else {
            1
        };
        if !query.contains(&format!("therapies(name:$q,first:{count},after:$after)")) {
            return Err("candidate request changed its literal selector or page size".into());
        }
        return Ok(());
    }
    let id = at(variables, "/id").as_u64().ok_or("missing therapy ID")?;
    if !matches!(id, 5 | 84 | 454) || !query.contains("therapy(id:$id){id name link}") {
        return Err("undeclared exact therapy request".into());
    }
    let mut profile = String::new();
    for suffix in query.split("molecularProfileId:").skip(1) {
        let value = suffix.split_once(',').ok_or("missing profile delimiter")?.0;
        if id != 5 || !matches!(value, "1" | "65") {
            return Err(format!("undeclared selected profile {value}"));
        }
        let selected = format!(",molecularProfileId:{value}");
        if !profile.is_empty() && profile != selected {
            return Err("mixed selected profiles".into());
        }
        profile = selected;
    }
    let mut streams = 0;
    for field in ["evidenceItems", "assertions"] {
        if query.contains(&format!("{field}(")) {
            if !query.contains(&format!(
                "{field}(therapyId:$id{profile},status:ACCEPTED,first:1,after:$after)"
            )) {
                return Err(format!(
                    "{field} changed its therapy, status or page selector"
                ));
            }
            if !profile.is_empty() && (field == "evidenceItems") != profile.ends_with(":1") {
                return Err("undeclared profile and claim stream pair".into());
            }
            streams += 1;
        }
    }
    if streams == 0 {
        return Err("missing claim collection".into());
    }
    Ok(())
}
