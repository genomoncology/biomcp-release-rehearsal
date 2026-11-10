//! A valid empty mutation page is evidence only after the cohort selector is admitted.
use crate::runner::{launch::Environment, mcp, network_stub::Stub};
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    for mode in [
        "unknown",
        "wrong-study",
        "wrong-list",
        "malformed",
        "empty",
        "uncounted",
    ] {
        scenario(mode)?;
    }
    Ok(())
}

fn scenario(mode: &'static str) -> Result<(), String> {
    let stub = serve(mode)?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    super::config(&env, &stub)?;
    let mut args = super::args();
    if mode == "unknown" {
        let index = args
            .iter()
            .position(|word| word == "--sample-set")
            .ok_or("sample selector")?;
        *args.get_mut(index + 1).ok_or("sample value")? = "all".into();
    }
    let admitted = matches!(mode, "empty" | "uncounted");
    let cli = super::call(&env, &args, if admitted { 0 } else { 4 })?;
    assert_reply(&cli, mode)?;
    let meta = mcp::call_with_globals(&env, &args, !admitted, &[])?;
    assert_reply(&meta, mode)?;
    let requests = stub.finish()?;
    let mutations = requests
        .iter()
        .filter(|(_, request)| request.contains("/mutations?"))
        .count();
    if mutations != usize::from(admitted) {
        return Err(format!(
            "{mode}: expected {} mutation reads, got {mutations}",
            usize::from(admitted)
        ));
    }
    Ok(())
}

fn assert_reply(reply: &Value, mode: &str) -> Result<(), String> {
    if !matches!(mode, "empty" | "uncounted") {
        let reason = if mode == "unknown" {
            "not-found"
        } else {
            "bad-reply"
        };
        if reply.get("status") != Some(&json!("failed"))
            || reply.get("result").is_some()
            || reply.get("next").is_some()
            || !reply
                .get("sources")
                .and_then(Value::as_array)
                .is_some_and(|sources| {
                    sources.iter().any(|source| {
                        source.get("source") == Some(&json!("cbioportal"))
                            && source.get("state") == Some(&json!("failed"))
                            && source.get("reason") == Some(&json!(reason))
                    })
                })
        {
            return Err(format!(
                "{mode} yielded a cohort observation instead of refusal: {reply}"
            ));
        }
    } else {
        let count = u64::from(mode == "uncounted");
        if reply.get("status") != Some(&json!("complete"))
            || reply.pointer("/result/observed_sample_count") != Some(&json!(count))
            || reply.pointer("/result/examined_rows") != Some(&json!(count))
            || reply.pointer("/result/all_pages_read") != Some(&json!(true))
            || reply.pointer("/result/more") != Some(&json!(false))
        {
            return Err(format!("{mode} lost its admitted observation: {reply}"));
        }
        if mode == "uncounted" {
            if reply.pointer("/result/selected_sample_count").is_some()
                || !reply
                    .to_string()
                    .contains("sample_set.selected_sample_count")
            {
                return Err(format!(
                    "missing optional count became a denominator: {reply}"
                ));
            }
        } else if reply.pointer("/result/selected_sample_count") != Some(&json!(100)) {
            return Err(format!("true zero lost the admitted list count: {reply}"));
        }
    }
    Ok(())
}

fn serve(mode: &'static str) -> Result<Stub, String> {
    Stub::with_headers(
        &[
            "www.cbioportal.org",
            "reg.genome.network",
            "storage.googleapis.com",
        ],
        |request| {
            if request.contains("hgnc_complete_set") {
                "text/tab-separated-values"
            } else {
                "application/json"
            }
        },
        move |request| {
            if request.starts_with("GET /api/sample-lists/") {
                let mut list = json!({"studyId":"msk_impact_2017", "sampleListId":"msk_impact_2017_all", "sampleCount":100});
                match mode {
                    "unknown" => list = json!({"error":"Unknown sample list"}),
                    "wrong-study" => {
                        *list.get_mut("studyId").ok_or("study id")? = json!("another_study")
                    }
                    "wrong-list" => {
                        *list.get_mut("sampleListId").ok_or("list id")? = json!("another_list")
                    }
                    "malformed" => *list.get_mut("sampleListId").ok_or("list id")? = json!({}),
                    "uncounted" => {
                        list.as_object_mut()
                            .ok_or("list fields")?
                            .remove("sampleCount");
                    }
                    _ => (),
                }
                return Ok(list.to_string());
            }
            if request.contains("/mutations?") {
                return Ok(if mode == "uncounted" {
                    json!([super::row("matching", "T")])
                } else {
                    json!([])
                }
                .to_string());
            }
            super::cbio(request)
        },
        |_, bytes| bytes,
        move |request| {
            if mode == "unknown" && request.starts_with("GET /api/sample-lists/") {
                404
            } else {
                200
            }
        },
        |_| String::new(),
    )
}
