//! Feature study-get: exact stored cards and bounded source metadata without member IDs.
use super::required_contract;
use super::{rows, seed};
use crate::runner::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, path::Path};
#[path = "study_get/resource_refusals.rs"]
mod resource_refusals;
#[path = "study_get_pages.rs"]
mod section_page;
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    // The test binds runtime store revisions and acquisition times while reconstructing independently owned native pages.
    vec![
        transcript_trial(required_contract::STUDY_PAGES.0, || {
            pages().map_err(Into::into)
        }),
        // Replace the native catalog after acquiring a pool and revisit its old immutable membership.
        transcript_trial(
            "study-get/resource-refusals-survive-catalog-replacement",
            || resource_refusals::run().map_err(Into::into),
        ),
    ]
}
fn profiles() -> Value {
    json!(
        (0..17)
            .map(|n| json!({"molecularProfileId":format!("native_profile_{n}"),"studyId":"synthetic_0",
        "molecularAlterationType":"MRNA_EXPRESSION","datatype":"CONTINUOUS","name":"Synthetic profile",
        "description":"Native profile description","patientLevel":false,"showProfileInAnalysisTab":true}))
            .collect::<Vec<_>>()
    )
}
fn lists() -> Value {
    json!(
        (0..13)
            .map(
                |n| json!({"sampleListId":format!("native_list_{n}"),"studyId":"synthetic_0","category":"all_cases_in_study",
        "name":"Synthetic list","description":"Native list description","sampleCount":n})
            )
            .collect::<Vec<_>>()
    )
}
fn record(replay: &Path, section: &str, body: Value, status: u16) -> Check {
    recording(replay, section, body, status, 0)
}
fn recording(replay: &Path, section: &str, body: Value, status: u16, delay: u64) -> Check {
    recording_raw(replay, section, &body.to_string(), status, delay)
}
pub(super) fn recording_raw(
    replay: &Path,
    section: &str,
    body: &str,
    status: u16,
    delay: u64,
) -> Check {
    let stub = crate::runner::network_stub::Stub::for_source(
        "www.cbioportal.org",
        "application/json",
        synthetic_reply,
    )?;
    let env = crate::runner::variant::source_environment(&stub)?;
    crate::runner::configuration::invoke(
        &env,
        vec!["sync".into(), "cbioportal-studies".into()],
        0,
    )?;
    let directory = env.root().join("recorded");
    let selected = if section == "molecular-profiles" {
        "profiles"
    } else {
        section
    };
    let acquired = crate::runner::configuration::invoke(
        &env,
        [
            "get",
            "study",
            "synthetic_0",
            selected,
            "--record",
            directory.to_str().ok_or("recording path")?,
        ]
        .map(str::to_owned)
        .to_vec(),
        0,
    )?;
    if stub.finish()?.len() != 2 {
        return Err("synthetic study recording changed request count".into());
    }
    for file in fs::read_dir(directory.join("cbioportal")).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let mut value: Value =
            serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if value.pointer("/exchanges/0/fetched") != acquired.pointer("/result/acquired")
            || acquired
                .pointer("/result/acquired")
                .and_then(Value::as_u64)
                .is_none()
        {
            return Err("live study acquisition differs from its recording receipt".into());
        }
        let exchange = json!(
            {"status":status,"fetched":42,"content_type":"application/json","delay_ms":delay,"retry_after_ms":null,"body":{"kind":"text",
            "value":body}}
        );
        let exchanges = if matches!(status, 429 | 500) {
            vec![exchange; 3]
        } else {
            vec![exchange]
        };
        *value.get_mut("exchanges").ok_or("recorded exchanges")? = json!(exchanges);
        fs::write(
            replay.join("cbioportal").join(file.file_name()),
            value.to_string(),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn synthetic_reply(request: &str) -> Result<String, String> {
    Ok(if request.contains("/molecular-profiles?") {
        profiles()
    } else if request.contains("/samples?") {
        json!([{ "sampleId": "synthetic_sample", "studyId": "synthetic_0", "sampleType": "Primary" }])
    } else if request.contains("/sample-lists?") {
        lists()
    } else {
        rows()
    }
    .to_string())
}

fn pages() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, rows(), 200)?;
    call(
        &env,
        &replay,
        &["sync", "cbioportal-studies"],
        0,
        "get/pages/1",
    )?;
    record(&replay, "molecular-profiles", profiles(), 200)?;
    record(&replay, "sample-lists", lists(), 200)?;
    section_page::limited(&env, &replay)?;
    for (section, total, id) in [
        ("profiles", 17, "molecularProfileId"),
        ("sample-lists", 13, "sampleListId"),
    ] {
        let mut count = 0;
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let offset = count.to_string();
            let value = call(
                &env,
                &replay,
                &[
                    "get",
                    "study",
                    "synthetic_0",
                    section,
                    "--section-offset",
                    &offset,
                ],
                0,
                "get/pages/2",
            )?;
            let items = value
                .pointer("/result/items")
                .and_then(Value::as_array)
                .ok_or_else(|| format!("missing section items: {value}"))?;
            if value.pointer("/page/total") != Some(&json!(total))
                || items.len() > 5
                || value.pointer("/result/acquired") != Some(&json!(42))
            {
                return Err(format!("native page bounds: {value}"));
            }
            for row in items {
                let identity = row.get(id).and_then(Value::as_str).ok_or("native ID")?;
                if !seen.insert(identity.to_owned())
                    || row.get("studyId") != Some(&json!("synthetic_0"))
                    || row.get("sampleIds").is_some()
                {
                    return Err("collection changed identity or exposed members".into());
                }
            }
            count += items.len();
            if count == total {
                break;
            }
            if items.is_empty() {
                return Err("page walk stopped early".into());
            }
        }
    }
    record(
        &replay,
        "samples",
        json!([{ "sampleId": "synthetic_sample", "studyId": "synthetic_0", "sampleType": "Primary" }]),
        200,
    )?;
    super::study_files::seed_files(&replay)?;
    let all = call(
        &env,
        &replay,
        &["get", "study", "synthetic_0", "all"],
        0,
        "get/pages/3",
    )?;
    bounded_all(&all)?;
    navigation(&env, &replay)?;
    Ok(())
}
fn bounded_all(all: &Value) -> Check {
    if all
        .pointer("/result/profiles/data/items")
        .and_then(Value::as_array)
        .map(Vec::len)
        != Some(5)
        || all
            .pointer("/result/sample_lists/data/items")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(5)
    {
        return Err(format!("all lost independent bounded sections: {all}"));
    }
    Ok(())
}

fn follow(env: &Environment, replay: &Path, value: &Value, kind: &str) -> Result<Value, String> {
    let next = value
        .get("next")
        .and_then(Value::as_array)
        .and_then(|steps| {
            steps
                .iter()
                .find(|s| s.get("kind").and_then(Value::as_str) == Some(kind))
        })
        .ok_or("next step")?;
    let args = next
        .get("args")
        .and_then(Value::as_array)
        .ok_or("step arguments")?
        .iter()
        .map(|a| a.as_str().ok_or("argument"))
        .collect::<Result<Vec<_>, _>>()?;
    let cli = call(env, replay, &args, 0, "get/follow/1")?;
    let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
    let mcp = crate::runner::mcp::call_with_globals(
        env,
        &args,
        false,
        &env.globals(Some(replay), false),
    )?;
    if cli != mcp {
        return Err("emitted study step differs between CLI and MCP".into());
    }
    Ok(cli)
}
fn navigation(env: &Environment, replay: &Path) -> Check {
    let card = call(
        env,
        replay,
        &["get", "study", "synthetic_0"],
        0,
        "get/navigation/1",
    )?;
    let section = follow(env, replay, &card, "section")?;
    let next = follow(env, replay, &section, "page")?;
    if next.pointer("/page/offset") != Some(&json!(5))
        || next.pointer("/result/acquired") != Some(&json!(42))
    {
        return Err("emitted section or page lost offset or acquisition".into());
    }
    call(
        env,
        replay,
        &["get", "study", "synthetic_absent"],
        1,
        "get/navigation/2",
    )?;
    fs::remove_dir_all(replay.join("cbioportal")).map_err(|e| e.to_string())?;
    call(
        env,
        replay,
        &["get", "study", "../escape"],
        2,
        "get/navigation/3",
    )?;
    Ok(())
}

fn call(
    env: &Environment,
    replay: &Path,
    args: &[&str],
    exit: i32,
    phase: &str,
) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|s| (*s).to_owned()));
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &command)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) || !output.stderr.is_empty() {
        return Err(format!(
            "study command {args:?}: expected exit {exit}, actual {:?}; stdout {}; stderr {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    super::budget(&value, phase, "cli")?;
    let words = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let mcp = crate::runner::mcp::call_with_globals(
        env,
        &words,
        exit > 1,
        &env.globals(Some(replay), false),
    )?;
    super::budget(&mcp, phase, "mcp")?;
    Ok(value)
}
