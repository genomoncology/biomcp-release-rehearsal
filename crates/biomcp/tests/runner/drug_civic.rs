//! Feature drug-civic: source bodies fetched 2026-10-25 are delivered locally, recorded and replayed.
//! Original fetch dates and request hashes remain in fixtures/drug/civic/provenance.json.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<Trial> {
    vec![transcript_trial(
        crate::runner::pool_contract::DRUG_CIVIC_CURSOR.0,
        || check(9).map_err(Into::into),
    )]
}
fn arguments(n: usize) -> Result<Vec<String>, String> {
    let mut args = if n == 14 {
        vec![
            "search",
            "drug",
            "synthetic distinct therapy identities",
            "--source",
            "civic",
            "--limit",
            "2",
        ]
    } else if n < 5 {
        let q = [
            "imatinib",
            "trastuzumab",
            "ivosidenib",
            "fam-trastuzumab deruxtecan",
            "trastuzumab",
        ]
        .get(n)
        .ok_or("query index")?;
        vec!["search", "drug", q, "--source", "civic", "--limit", "1"]
    } else {
        let id = match n {
            5 | 8..=13 => "5",
            6 => "84",
            7 => "454",
            _ => return Err("unknown case".into()),
        };
        vec!["get", "drug", id, "civic", "--source", "civic"]
    };
    if n == 4 {
        args.extend(["--offset", "1"]);
    }
    if (8..=13).contains(&n) {
        let (profile, kind) = if matches!(n, 10 | 11 | 13) {
            ("65", "assertions")
        } else {
            ("1", "evidence")
        };
        args.extend([
            "--section-limit",
            "1",
            "--civic-profile",
            profile,
            "--civic-kind",
            kind,
        ]);
        if matches!(n, 9 | 11) {
            args.extend(["--civic-cursor", "MQ"]);
        }
    }
    if matches!(n, 12 | 13) {
        args.extend(["--section-offset", "1"]);
    }
    Ok(args.into_iter().map(String::from).collect())
}

fn check(n: usize) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let stub = Stub::for_source(
        "civicdb.org",
        "application/json",
        super::drug_civic_fixtures::reply,
    )?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    let args = arguments(n)?;
    let recorded = env.root().join("recorded");
    let mut record_args = args.clone();
    record_args.extend(["--record".into(), recorded.display().to_string()]);
    let exit = 0;
    let value = invoke(&env, record_args, exit)?;
    super::schema::Registry::embedded()?.validate(&value, &args, true)?;
    verify(n, &value)?;
    let proxy_address = stub.address.to_string();
    let requests = stub.finish()?;
    let expected = if matches!(n, 4 | 12) { 2 } else { 1 };
    if requests.len() != expected {
        return Err(format!(
            "expected {expected} local requests, got {}",
            requests.len()
        ));
    }
    let mut replay_args = args.clone();
    replay_args.extend([
        "--replay".into(),
        recorded.display().to_string(),
        "--fresh".into(),
    ]);
    let replay = invoke(&env, replay_args.clone(), exit)?;
    let globals = vec![
        "--replay".into(),
        recorded.display().to_string(),
        "--fresh".into(),
    ];
    if n == 5 {
        disabled(&env, &proxy_address, &ca, &replay_args)?;
    }
    let mcp = super::mcp::call_with_globals(&env, &args, false, &globals)?;
    for key in ["result", "status", "query"] {
        if replay.get(key) != mcp.get(key) || replay.get(key) != value.get(key) {
            return Err(format!("CLI, recorder and MCP differ for {key}"));
        }
    }
    let full = serde_json::to_vec(&value).map_err(|e| e.to_string())?.len();
    let comparable = super::source_terms::size(&value, "")?;
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "CIVIC_SIZE case={n} full_bytes={full} full_tokens={} comparable_bytes={comparable} comparable_tokens={}",
        full.div_ceil(4),
        comparable.div_ceil(4)
    )
    .map_err(|e| e.to_string())?;
    let bound = [
        701, 726, 704, 304, 739, 3621, 3637, 4002, 2088, 2113, 2176, 577, 2160, 575, 850,
    ]
    .get(n)
    .copied()
    .ok_or("budget index")?;
    let measured_full: [usize; 15] = [
        693, 715, 696, 348, 726, 3313, 3320, 3638, 1943, 1965, 2013, 585, 2006, 584, 823,
    ];
    if comparable > bound
        || full > (*measured_full.get(n).ok_or("full budget index")? * 115).div_ceil(100)
    {
        return Err(format!(
            "case {n} exceeds CIViC section ceiling: {comparable}"
        ));
    }
    Ok(())
}
fn verify(n: usize, value: &Value) -> Result<(), String> {
    if (5..14).contains(&n) {
        missing_native_build(value)?;
    }
    let expected = match n {
        0..=4 | 14 => search_expected(n)?,
        5..=7 => drug_expected(n)?,
        _ => stream_expected(n)?,
    };
    for (path, expected) in expected {
        if value.pointer(path) != Some(&expected) {
            return Err(format!("{path}: expected {expected}, received {value}"));
        }
    }
    if matches!(n, 10 | 11 | 13)
        && value
            .pointer("/result/civic/data/assertions/page/next")
            .is_some()
    {
        return Err("invented a positive assertion continuation".into());
    }
    if n == 5
        && value
            .pointer("/result/civic/data/evidence/page/next/civic_profile")
            .is_some()
    {
        return Err("therapy-wide total relabeled as a selected profile".into());
    }
    Ok(())
}

fn search_expected(n: usize) -> Result<Vec<(&'static str, Value)>, String> {
    Ok(match n {
        14 => vec![
            ("/result/total", json!(2)),
            ("/result/items/0/id", json!("84")),
            ("/result/items/1/id", json!("4287")),
        ],
        0 => vec![
            ("/result/total", json!(11)),
            ("/result/items/0/id", json!("27062")),
        ],
        1 => vec![
            ("/result/total", json!(104)),
            ("/result/items/0/id", json!("2986")),
        ],
        2 => vec![
            ("/result/total", json!(3)),
            ("/result/items/0/id", json!("25758")),
        ],
        3 => vec![("/result/total", json!(0)), ("/status", json!("complete"))],
        4 => vec![
            ("/result/total", json!(104)),
            ("/result/items/0/id", json!("2988")),
        ],
        _ => return Err("unknown verification".into()),
    })
}

fn drug_expected(n: usize) -> Result<Vec<(&'static str, Value)>, String> {
    Ok(match n {
        5 => vec![
            ("/result/identity/id", json!("5")),
            ("/result/civic/data/evidence/page/total", json!(169)),
            ("/result/civic/data/assertions/page/total", json!(1)),
            (
                "/result/civic/data/evidence/records/0/profile/record/id",
                json!("1"),
            ),
            (
                "/result/civic/data/assertions/records/0/profile/record/id",
                json!("65"),
            ),
            (
                "/result/civic/data/evidence/page/next/civic_kind",
                json!("evidence"),
            ),
        ],
        6 => vec![
            ("/result/identity/id", json!("84")),
            (
                "/result/civic/data/evidence/records/0/therapies/0/id",
                json!("52"),
            ),
            (
                "/result/civic/data/evidence/records/0/therapies/1/id",
                json!("84"),
            ),
            (
                "/result/civic/data/evidence/records/0/therapy_interaction_type",
                json!("SEQUENTIAL"),
            ),
        ],
        7 => vec![("/result/identity/id", json!("454"))],
        _ => return Err("unknown verification".into()),
    })
}

fn stream_expected(n: usize) -> Result<Vec<(&'static str, Value)>, String> {
    Ok(match n {
        8 => vec![
            ("/result/civic/data/evidence/page/total", json!(4)),
            (
                "/result/civic/data/evidence/page/next/civic_profile",
                json!(1),
            ),
            (
                "/result/civic/data/evidence/page/next/civic_cursor",
                json!("MQ"),
            ),
        ],
        9 | 12 => vec![
            (
                "/result/civic/data/evidence/records/0/claim/record/id",
                json!("EID260"),
            ),
            ("/result/civic/data/evidence/page/total", json!(4)),
        ],
        10 => vec![
            ("/result/civic/data/assertions/page/total", json!(1)),
            (
                "/result/civic/data/assertions/records/0/profile/record/id",
                json!("65"),
            ),
        ],
        11 | 13 => vec![
            ("/result/civic/data/assertions/page/total", json!(1)),
            ("/result/civic/data/assertions/page/returned", json!(0)),
        ],
        _ => return Err("unknown verification".into()),
    })
}

fn disabled(
    env: &Environment,
    proxy: &str,
    ca: &std::path::Path,
    args: &[String],
) -> Result<(), String> {
    env.config(
        &json!({"ca_bundle":ca,"proxy":format!("http://{proxy}"),"disable_sources":["civic"]}),
    )
    .map_err(|e| e.to_string())?;
    let blocked = invoke(env, args.to_vec(), 4)?;
    if blocked.pointer("/sources/0/state") != Some(&json!("skipped"))
        || blocked.pointer("/sources/0/reason") != Some(&json!("disabled"))
        || blocked.get("result").is_some_and(|v| !v.is_null())
    {
        return Err(format!(
            "disabled CIViC leaked cached or replayed content: {blocked}"
        ));
    }
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{proxy}")}))
        .map_err(|e| e.to_string())
}

// Original requests did not select coordinates.referenceBuild. Never assign an assembly.
fn missing_native_build(value: &Value) -> Result<(), String> {
    for stream in ["evidence", "assertions"] {
        if let Some(rows) = value
            .pointer(&format!("/result/civic/data/{stream}/records"))
            .and_then(Value::as_array)
        {
            for row in rows {
                if !row
                    .get("oncokb_missing")
                    .and_then(Value::as_array)
                    .is_some_and(|fields| fields.contains(&json!("build")))
                {
                    return Err("retained source claim lost its missing-build disclosure".into());
                }
            }
        }
    }
    if value
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| {
            steps.iter().any(|s| {
                s.get("args")
                    .and_then(Value::as_array)
                    .is_some_and(|args| args.contains(&json!("oncokb")))
            })
        })
    {
        return Err("retained build-free claim gained an OncoKB command".into());
    }
    Ok(())
}
