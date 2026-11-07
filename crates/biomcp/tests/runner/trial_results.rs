//! Feature trial-results: posted source values, group relationships and intact event pages.
#![expect(
    clippy::indexing_slicing,
    reason = "Source-quoted fixture assertions fail on shape drift"
)]
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript_trial(super::pool_contract::TRIAL_RESULTS.0, || {
            posted().map_err(|e| e.to_string().into())
        }),
        super::transcript_trial("trial-results/no-posted-results", || {
            empty().map_err(|e| e.to_string().into())
        }),
    ]
}

type Check = Result<(), Box<dyn std::error::Error>>;
fn posted() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("trial/results/NCT01844505", &replay, |_, _| {})?;
    let first = command(
        &env,
        &["get", "trial", "NCT01844505", "results", "--limit", "50"].map(str::to_owned),
        &env.globals(Some(&replay), false),
        0,
    )?;
    if first["result"]["items"][0]["card"]["results_first_posted"] != "2017-09-26" {
        return Err("results first posted date differs from protocol status".into());
    }
    posted_primary(&first)?;
    let mut rows = Vec::new();
    let mut current = first;
    // Continuations are served offline from the same stored record, never a new HTTP request.
    loop {
        let page = &current["result"]["items"][0]["sections"]["results"]["data"];
        if page["offset"].as_u64() != Some(rows.len() as u64)
            || page["items"].as_array().is_none_or(|r| r.len() > 50)
        {
            return Err("result page offset or section limit changed".into());
        }
        rows.extend(
            page["items"]
                .as_array()
                .ok_or("missing result records")?
                .iter()
                .cloned(),
        );
        if rows.len() as u64 == page["total"].as_u64().ok_or("missing total")? {
            break;
        }
        let next = super::action(&current, "page")?;
        current = command(&env, &next, &env.globals(None, true), 0)?;
    }
    let serious: Vec<_> = rows
        .iter()
        .filter(|r| r["record"] == "serious-event")
        .map(|r| r["data"].clone())
        .collect();
    if serious.len() != 393
        || serious[0]["term"] != "Anaemia"
        || serious[0]["stats"]
            != json!([
            {"groupId":"EG000","numAffected":5,"numAtRisk":313},
            {"groupId":"EG001","numAffected":4,"numAtRisk":313},
            {"groupId":"EG002","numAffected":5,"numAtRisk":311}])
    {
        return Err("serious adverse-event terms or group counts changed".into());
    }
    let groups: Vec<_> = rows
        .iter()
        .filter(|r| r["record"] == "adverse-event-group")
        .collect();
    if groups.len() != 3
        || groups[0]["data"]["id"] != "EG000"
        || groups[0]["data"]["seriousNumAtRisk"] != 313
    {
        return Err("adverse-event group denominator changed".into());
    }
    native_rows(&rows, &serious)
}

/// Compares every whole native row, including later pages, to the independent captured reply.
fn native_rows(rows: &[Value], serious: &[Value]) -> Check {
    let source = plain(&source()?)?;
    let native = &source["resultsSection"];
    let outcomes: Vec<_> = rows
        .iter()
        .filter(|r| r["record"] == "outcome")
        .map(|r| r["data"].clone())
        .collect();
    let other: Vec<_> = rows
        .iter()
        .filter(|r| r["record"] == "other-event")
        .map(|r| r["data"].clone())
        .collect();
    if json!([&outcomes, &serious, &other])
        .to_string()
        .contains(r"\\")
    {
        return Err("a Markdown escape reached posted result text".into());
    }
    if json!(outcomes) != native["outcomeMeasuresModule"]["outcomeMeasures"]
        || json!(serious) != native["adverseEventsModule"]["seriousEvents"]
        || json!(other) != native["adverseEventsModule"]["otherEvents"]
    {
        return Err("posted native record was lost, reordered or changed during paging".into());
    }
    Ok(())
}
fn posted_primary(first: &Value) -> Check {
    let primary = &first["result"]["items"][0]["sections"]["results"]["data"]["items"][0];
    if primary["record"] != "outcome"
        || primary["data"]["title"] != "Progression Free Survival (PFS)"
        || primary["data"]["paramType"] != "MEDIAN"
        || primary["data"]["dispersionType"] != "95% Confidence Interval"
        || primary["data"]["unitOfMeasure"] != "months"
    {
        return Err("reported PFS labels changed".into());
    }
    for (index, id, title, value, lower, upper) in [
        (0, "OG000", "Nivolumab", "6.87", "4.34", "9.46"),
        (
            1,
            "OG001",
            "Nivolumab + Ipilimumab",
            "11.50",
            "8.90",
            "16.72",
        ),
        (2, "OG002", "Ipilimumab", "2.89", "2.79", "3.42"),
    ] {
        let data = &primary["data"];
        if data["groups"][index]["id"] != id
            || data["groups"][index]["title"] != title
            || data["classes"][0]["categories"][0]["measurements"][index]
                != json!({"groupId":id,"value":value,"lowerLimit":lower,"upperLimit":upper})
        {
            return Err("reported median/CI or outcome group changed".into());
        }
    }
    if primary["data"]["analyses"][0]["groupIds"] != json!(["OG000", "OG002"])
        || primary["data"]["analyses"][0]["paramValue"] != "0.57"
        || primary["data"]["analyses"][0]["ciLowerLimit"] != "0.43"
        || primary["data"]["analyses"][0]["ciUpperLimit"] != "0.76"
    {
        return Err("posted analysis or comparison groups changed".into());
    }
    Ok(())
}

/// The captured reply as readers get it: the source's own `\<` and `\>` escapes removed.
fn plain(source: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let text = source.to_string();
    Ok(serde_json::from_str(
        &text.replace(r"\\<", "<").replace(r"\\>", ">"),
    )?)
}
fn source() -> Result<Value, Box<dyn std::error::Error>> {
    for file in std::fs::read_dir(
        super::launch::fixtures().join("replies/trial/results/NCT01844505/ctgov"),
    )? {
        let record: Value = serde_json::from_slice(&std::fs::read(file?.path())?)?;
        if record["url"]
            .as_str()
            .is_some_and(|u| u.ends_with("NCT01844505"))
        {
            return Ok(record["exchanges"][0]["body"]["value"].clone());
        }
    }
    Err("source reply missing".into())
}
fn empty() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("trial/results/NCT07041320", &replay, |_, _| {})?;
    let reply = command(
        &env,
        &["get", "trial", "NCT07041320", "results"].map(str::to_owned),
        &env.globals(Some(&replay), false),
        0,
    )?;
    if reply["result"]["items"][0]["sections"]["results"]["state"] != "empty"
        || reply["result"]["items"][0]["card"]
            .get("results_first_posted")
            .is_some()
    {
        return Err("trial without posted results must have an empty section and no date".into());
    }
    Ok(())
}
