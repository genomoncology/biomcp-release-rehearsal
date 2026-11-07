//! Feature study-get: exact stored cards and bounded source metadata without member IDs.
use super::required_contract;
use super::{rows, seed};
use crate::runner::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use sha2::Digest;
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
        transcript_trial("study-get/oversized-list-keeps-its-row", || {
            oversized_list().map_err(Into::into)
        }),
        transcript_trial("study-get/failed-list-count-keeps-the-section", || {
            failed_count().map_err(Into::into)
        }),
        transcript_trial("study-get/count-deadline-marks-the-rest", || {
            count_deadline().map_err(Into::into)
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
            "--section-limit",
            "50",
            "--record",
            directory.to_str().ok_or("recording path")?,
        ]
        .map(str::to_owned)
        .to_vec(),
        0,
    )?;
    // One catalog read and one section read; sample lists add one count read per list.
    let counts = if selected == "sample-lists" { 13 } else { 0 };
    if stub.finish()?.len() != 2 + counts {
        return Err("synthetic study recording changed request count".into());
    }
    for file in fs::read_dir(directory.join("cbioportal")).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let mut value: Value =
            serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if copied_count(replay, &file, &value)? {
            continue;
        }
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
/// Copies a sample-list count recording as recorded; returns whether the file was one.
fn copied_count(replay: &Path, file: &fs::DirEntry, value: &Value) -> Result<bool, String> {
    let count = value
        .get("url")
        .and_then(Value::as_str)
        .is_some_and(|url| url.contains("/api/sample-lists/"));
    if count {
        fs::copy(
            file.path(),
            replay.join("cbioportal").join(file.file_name()),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(count)
}
fn synthetic_reply(request: &str) -> Result<String, String> {
    Ok(if request.contains("/molecular-profiles?") {
        profiles()
    } else if request.contains("/samples?") {
        json!([{ "sampleId": "synthetic_sample", "studyId": "synthetic_0", "sampleType": "Primary" }])
    } else if request.contains("/sample-lists?") {
        lists()
    } else if let Some(list) = request
        .strip_prefix("GET /api/sample-lists/")
        .and_then(|path| path.split([' ', '?']).next())
    {
        let count = list.rsplit('_').next().and_then(|n| n.parse::<u64>().ok());
        json!({ "sampleListId": list, "studyId": "synthetic_0", "sampleCount": count })
    } else {
        rows()
    }
    .to_string())
}

/// One list above the read bound leaves its row and says its count is missing.
fn oversized_list() -> Check {
    let stub = crate::runner::network_stub::Stub::for_source(
        "www.cbioportal.org",
        "application/json",
        |request| {
            Ok(if request.contains("/sample-lists?") {
                json!([{"sampleListId":"native_list_big","studyId":"synthetic_0",
                    "category":"all_cases_in_study","name":"Synthetic list",
                    "description":"Synthetic list (200000 samples)"}])
            } else if request.starts_with("GET /api/sample-lists/") {
                json!({"sampleListId":"native_list_big","studyId":"synthetic_0","sampleCount":200_000,
                    "sampleIds":vec!["synthetic_member"; 200_000]})
            } else {
                rows()
            }
            .to_string())
        },
    )?;
    let env = crate::runner::variant::source_environment(&stub)?;
    let invoke = |args: &[&str]| {
        let args = args.iter().map(|arg| (*arg).to_owned()).collect();
        crate::runner::configuration::invoke(&env, args, 0)
    };
    invoke(&["sync", "cbioportal-studies"])?;
    let reply = invoke(&["get", "study", "synthetic_0", "sample-lists"])?;
    let row = reply.pointer("/result/items/0").ok_or("list row missing")?;
    if reply.get("status") != Some(&json!("partial"))
        || row.get("sampleListId") != Some(&json!("native_list_big"))
        || row.get("sampleCount").is_some()
        || reply.pointer("/problems/0/field") != Some(&json!("sampleCount: too-large"))
        || reply.pointer("/problems/0/code") != Some(&json!("collection.incomplete"))
    {
        return Err(format!("oversized list reply changed: {reply}"));
    }
    Ok(())
}

/// The second list's count reply names another study; both rows stay, and only it lacks a count.
fn failed_count() -> Check {
    let stub = crate::runner::network_stub::Stub::for_source(
        "www.cbioportal.org",
        "application/json",
        |request| {
            Ok(if request.contains("/sample-lists?") {
                json!(
                    ["native_list_1", "native_list_2", "native_list_3"].map(|id| json!({
                    "sampleListId": id, "studyId": "synthetic_0", "category": "all_cases_in_study",
                    "name": "Synthetic list", "description": "Synthetic list"}))
                )
            } else if let Some(list) = request
                .strip_prefix("GET /api/sample-lists/")
                .and_then(|path| path.split([' ', '?']).next())
            {
                let study = if list == "native_list_2" {
                    "other_study"
                } else {
                    "synthetic_0"
                };
                json!({ "sampleListId": list, "studyId": study, "sampleCount": 7 })
            } else {
                rows()
            }
            .to_string())
        },
    )?;
    let env = crate::runner::variant::source_environment(&stub)?;
    let invoke = |args: &[&str]| {
        let args = args.iter().map(|arg| (*arg).to_owned()).collect();
        crate::runner::configuration::invoke(&env, args, 0)
    };
    invoke(&["sync", "cbioportal-studies"])?;
    let reply = invoke(&["get", "study", "synthetic_0", "sample-lists"])?;
    let counts: Vec<_> = (0..3)
        .map(|i| {
            reply
                .pointer(&format!("/result/items/{i}/sampleCount"))
                .cloned()
        })
        .collect();
    if reply.get("status") != Some(&json!("partial"))
        || counts != [Some(json!(7)), None, Some(json!(7))]
        || reply.pointer("/problems/0/record") != Some(&json!("native_list_2"))
        || reply.pointer("/problems/0/field") != Some(&json!("sampleCount: bad-reply"))
        || reply.pointer("/problems/0/code") != Some(&json!("collection.incomplete"))
    {
        return Err(format!("failed list count reply changed: {reply}"));
    }
    Ok(())
}

/// The second count read runs out of time, so the third is not attempted and both lack counts.
fn count_deadline() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, rows(), 200)?;
    // Direct calls: these replies have no stored byte budget phase.
    let run = |args: &[&str]| {
        let mut words = env.globals(Some(&replay), false);
        words.extend(args.iter().map(|arg| (*arg).to_owned()));
        crate::runner::configuration::invoke(&env, words, 0)
    };
    run(&["sync", "cbioportal-studies"])?;
    let lists = ["native_list_1", "native_list_2", "native_list_3"];
    let summary = json!(lists.map(|id| json!({"sampleListId": id, "studyId": "synthetic_0",
        "category": "all_cases_in_study", "name": "Synthetic list", "description": "Synthetic list"})));
    replayed(
        &replay,
        "https://www.cbioportal.org/api/studies/synthetic_0/sample-lists?projection=SUMMARY",
        &summary,
        0,
    )?;
    // The third list has no recording: a read of it would fail the command as a replay miss.
    for (id, delay) in [("native_list_1", 0), ("native_list_2", 30_001)] {
        let count = json!({"sampleListId": id, "studyId": "synthetic_0", "sampleCount": 7});
        replayed(
            &replay,
            &format!("https://www.cbioportal.org/api/sample-lists/{id}"),
            &count,
            delay,
        )?;
    }
    let reply = run(&["get", "study", "synthetic_0", "sample-lists"])?;
    let counts: Vec<_> = (0..3)
        .map(|i| {
            reply
                .pointer(&format!("/result/items/{i}/sampleCount"))
                .cloned()
        })
        .collect();
    let missing: Vec<_> = reply
        .get("problems")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|p| p.get("field") == Some(&json!("sampleCount: deadline-exceeded")))
        .filter_map(|p| p.get("record").cloned())
        .collect();
    if reply.get("status") != Some(&json!("partial"))
        || counts != [Some(json!(7)), None, None]
        || missing != [json!("native_list_2"), json!("native_list_3")]
    {
        return Err(format!("count deadline reply changed: {reply}"));
    }
    Ok(())
}
fn replayed(replay: &Path, url: &str, body: &Value, delay: u64) -> Check {
    let key: String = sha2::Sha256::digest(format!("GET\0{url}").as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let recording = json!({"source":"cbioportal","method":"GET","url":url,"exchanges":[{"status":200,
        "fetched":42,"content_type":"application/json","delay_ms":delay,"retry_after_ms":null,
        "body":{"kind":"text","value":body.to_string()}}]});
    fs::write(
        replay.join("cbioportal").join(format!("{key}.json")),
        recording.to_string(),
    )
    .map_err(|e| e.to_string())
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
