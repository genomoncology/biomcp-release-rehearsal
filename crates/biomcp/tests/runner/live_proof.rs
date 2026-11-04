use super::{launch::Environment, live, response_cases};
use crate::runner::transcript::trial as transcript_trial;
use biomcp_fetch::Observation;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

#[path = "live_witness_proof.rs"]
mod witness_proof;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        #[cfg(unix)]
        transcript_trial("live-source-checks/report-is-one-json-value", || {
            stdout_report().map_err(|e| e.to_string().into())
        }),
        transcript_trial("live-source-checks/cache-retries-and-request-plans", || {
            comparison().map_err(|e| e.to_string().into())
        }),
        transcript_trial("live-source-checks/summary-retains-late-drift", || {
            summary().map_err(|e| e.to_string().into())
        }),
        transcript_trial("live-source-checks/retained-drift-witnesses", || {
            witness_proof::retained().map_err(Into::into)
        })
        .with_kind(super::proof_requirements::REPOSITORY_PYTHON),
        transcript_trial("live-source-checks/credential-free-witness-urls", || {
            witness_proof::credentials().map_err(Into::into)
        }),
        transcript_trial("live-source-checks/offline-planted-drift", || {
            check().map_err(|e| e.to_string().into())
        }),
        transcript_trial("live-source-checks/offline-eligibility", || {
            plan().map_err(|e| e.to_string().into())
        }),
        transcript_trial("live-source-checks/raw-wire-formats-and-retries", || {
            formats().map_err(|e| e.to_string().into())
        }),
    ]
}

#[cfg(unix)]
fn stdout_report() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let environment = Environment::new()?;
    let root = environment.root().canonicalize()?;
    let fixtures = root.join("fixtures");
    fs::create_dir_all(fixtures.join("cases"))?;
    fs::copy(
        super::launch::fixtures().join("cases/mcp/research-search.yaml"),
        fixtures.join("cases/research-search.yaml"),
    )?;
    let replay = fixtures.join("replies/trial/condition");
    response_cases::copy_capture("trial/condition", &replay, |_, _| {})?;
    let quote = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
    let binary = quote(
        &super::launch::executable(Path::new(env!("CARGO_BIN_EXE_biomcp"))).canonicalize()?,
    );
    let wrapper = root.join("recorded-biomcp");
    // Both sides use recorded provider replies. The existing installed-binary
    // boundary lets this exercise the real live-check entry point offline.
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nfor arg do\n  if [ \"$arg\" = --replay ]; then\n    exec {binary} \"$@\"\n  fi\ndone\nexec {binary} \"$@\" --replay {}\n",
            quote(&replay)
        ),
    )?;
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))?;
    let output = environment
        .command(
            std::env::current_exe()?,
            &["--live-check".into(), "--details".into()],
        )
        .env("TMPDIR", &root)
        .env("BIOMCP_TEST_EXECUTABLE", &wrapper)
        .env("BIOMCP_TEST_FIXTURES", &fixtures)
        .output()?;
    let report: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "live-check stdout is not one JSON report: {error}; stdout={}",
            String::from_utf8_lossy(&output.stdout)
        )
    })?;
    if !output.status.success()
        || report.get("checked") != Some(&json!(1))
        || report.get("error_cases") != Some(&json!(0))
        || report.get("drift_cases") != Some(&json!(0))
        || !String::from_utf8_lossy(&output.stderr).contains("research search compact_bytes=")
    {
        return Err(format!("recorded live-check failed: {output:?}").into());
    }
    Ok(())
}
fn check() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let before = env.root().join("before");
    let after = env.root().join("after");
    for (root, edited) in [(&before, false), (&after, true)] {
        response_cases::copy_capture("gene/main", root, |source, record| {
            if source != "mygene" {
                return;
            }
            let Some(exchanges) = record.get_mut("exchanges").and_then(Value::as_array_mut) else {
                return;
            };
            for exchange in exchanges {
                let Some(items) = exchange
                    .pointer_mut("/body/value")
                    .and_then(Value::as_array_mut)
                else {
                    continue;
                };
                for item in items.iter_mut().filter_map(Value::as_object_mut) {
                    item.insert("count".into(), json!(if edited { 0 } else { 1 }));
                    if edited && let Some(summary) = item.remove("summary") {
                        item.insert("renamed_summary".into(), summary);
                    }
                }
            }
        })?;
    }
    let before = observe(&env, &before, "before", &["get", "gene", "KRAS"])?;
    let after = observe(&env, &after, "after", &["get", "gene", "KRAS"])?;
    let mut changes = Vec::new();
    live::compare(&before, &after, &mut changes, &mut BTreeMap::new());
    if !changes
        .iter()
        .any(|s| s.contains("field removed: /*/summary"))
        || !changes
            .iter()
            .any(|s| s.contains("count dropped to zero: /*/count"))
    {
        return Err(format!("edited recording drift missing: {changes:?}").into());
    }
    // The same bounded observer also sees XML fields, bundle headers and attempt metadata.
    if !before.iter().any(|o| o.shape.header.is_some())
        || !before.iter().any(|o| o.bytes > 0 && o.status == Some(200))
    {
        return Err("missing bundle header or attempt metadata".into());
    }
    Ok(())
}
fn observe(
    env: &Environment,
    replay: &Path,
    name: &str,
    args: &[&str],
) -> Result<Vec<Observation>, Box<dyn std::error::Error>> {
    let report = env.root().join(format!("{name}.jsonl"));
    let mut args = args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    args.extend([
        "--replay".into(),
        replay.display().to_string(),
        "--observe".into(),
        report.display().to_string(),
    ]);
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &args).output()?;
    if !output.status.success() {
        return Err(format!(
            "observed replay {name} failed with {:?}",
            output.status.code()
        )
        .into());
    }
    let text = fs::read_to_string(report)?;
    if text.is_empty() || text.contains("summary\":\"") {
        return Err("wire report empty or contains payload".into());
    }
    Ok(text
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?)
}

fn plan() -> Result<(), Box<dyn std::error::Error>> {
    let environment = Environment::new()?;
    let output = environment
        .command(
            std::env::current_exe()?,
            &["--live-check".into(), "--list".into()],
        )
        .output()?;
    if !output.status.success() {
        return Err("offline eligibility listing failed".into());
    }
    let report: Value = serde_json::from_slice(&output.stdout)?;
    let cases = report
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("case list missing")?;
    if !cases.iter().any(|c| {
        c.get("case") == Some(&json!("article/sections/grammar.yaml"))
            && c.get("eligible") == Some(&json!(true))
    }) || !cases.iter().any(|c| {
        c.get("case") == Some(&json!("article/sections/failure-500.yaml"))
            && c.get("skipped").is_some()
    }) {
        return Err("eligibility list omitted public capture or failure skip".into());
    }
    for name in [
        "gene/retired.yaml",
        "drug/label-controls.yaml",
        "article/litsense/licensed-subset.yaml",
    ] {
        if !cases.iter().any(|case| {
            case.get("case") == Some(&json!(name)) && case.get("skipped").is_some()
        }) {
            return Err(format!("recorded-only control entered live checks: {name}").into());
        }
    }
    for name in [
        "mcp/research-search.yaml",
        "mcp/research-fetch.yaml",
        "pathway-sections/examples.yaml",
        "cell-line/assay-examples.yaml",
    ] {
        if !cases.iter().any(|case| {
            case.get("case") == Some(&json!(name)) && case.get("eligible") == Some(&json!(true))
        }) {
            return Err(format!("eligible source operation omitted: {name}").into());
        }
    }
    Ok(())
}

fn formats() -> Result<(), Box<dyn std::error::Error>> {
    let environment = Environment::new()?;
    let root = super::launch::fixtures().join("replies");
    let case: super::Case = serde_saphyr::from_str(&fs::read_to_string(
        super::launch::fixtures().join("cases/article/sections/grammar.yaml"),
    )?)?;
    let args = case
        .steps
        .first()
        .and_then(|step| step.run.as_ref())
        .ok_or("native XML command absent")?;
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    let replay = case.replay.as_deref().ok_or("native XML replay absent")?;
    let xml = observe(&environment, &root.join(replay), "xml", &args)?;
    if !xml
        .iter()
        .any(|o| o.shape.fields.contains_key("PubmedArticleSet"))
    {
        return Err("raw XML names were discarded".into());
    }
    let bundles = observe(
        &environment,
        &root.join("gene/clingen"),
        "csv",
        &["sync", "clingen"],
    )?;
    if !bundles.iter().all(|o| {
        o.shape
            .header
            .as_ref()
            .is_some_and(|h| h.contains("GENE SYMBOL") && !h.contains("FILE CREATED"))
    }) {
        return Err("CSV banner replaced actual header".into());
    }
    let retries = observe(
        &environment,
        &retry_capture(&environment)?,
        "retries",
        &["get", "gene", "BRAF"],
    )?;
    if retries.iter().filter(|o| o.status == Some(429)).count() < 2 {
        return Err("retry HTTP attempts were discarded".into());
    }
    let report = environment.root().join("xml.jsonl");
    let original = fs::read(&report)?;
    let output = environment
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &[
                "get".into(),
                "gene".into(),
                "BRAF".into(),
                "--observe".into(),
                report.display().to_string(),
            ],
        )
        .output()?;
    if output.status.success() || fs::read(report)? != original {
        return Err("existing observation report was replaced".into());
    }
    Ok(())
}

fn retry_capture(
    environment: &Environment,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let path = environment.root().join("retry-recordings");
    response_cases::copy_capture("gene/synthetic-429", &path, |_, record| {
        if let Some(exchanges) = record.get_mut("exchanges").and_then(Value::as_array_mut) {
            for exchange in exchanges {
                if let Some(wait) = exchange.get_mut("retry_after_ms") {
                    *wait = json!(0);
                }
            }
        }
    })?;
    Ok(path)
}

fn summary() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = (0..25)
        .map(|index| json!({"case": format!("public-{index}"), "result": {"drift": []}}))
        .collect::<Vec<_>>();
    cases.push(json!({"case": "late-drift", "result": {"drift": ["field removed: /name"]}}));
    cases.push(json!({"case": "late-error", "error": "command did not return JSON"}));
    cases.push(json!({"case": "not-public", "skipped": "synthetic response case"}));
    let report = live::summary(cases, BTreeMap::new(), 0, false);
    let sample = report
        .get("case_sample")
        .and_then(Value::as_array)
        .ok_or("missing sample")?;
    if report.get("drift_cases") != Some(&json!(1))
        || report.get("error_cases") != Some(&json!(1))
        || report.get("checked") != Some(&json!(27))
        || report.get("eligible") != Some(&json!(27))
        || report.get("omitted") != Some(&json!(7))
        || sample.len() != 20
        || report.pointer("/case_sample/0/case") != Some(&json!("late-drift"))
        || report.pointer("/case_sample/1/case") != Some(&json!("late-error"))
    {
        return Err(format!("late diagnostic hidden from bounded summary: {report}").into());
    }
    Ok(())
}

fn comparison() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let original = super::launch::fixtures().join("replies/gene/main");
    let retry = env.root().join("retried");
    response_cases::copy_capture("gene/main", &retry, |source, record| {
        if source == "mygene"
            && let Some(exchanges) = record.get_mut("exchanges").and_then(Value::as_array_mut)
        {
            let Some(mut error) = exchanges.first().cloned() else {
                return;
            };
            if let Some(error) = error.as_object_mut() {
                error.insert("status".into(), json!(429));
                error.insert(
                    "body".into(),
                    json!({"kind": "json", "value": {"error": "retry", "count": 2}}),
                );
                error.insert("retry_after_ms".into(), json!(0));
            }
            exchanges.insert(0, error);
        }
    })?;
    let before = observe(&env, &retry, "retry-before", &["get", "gene", "KRAS"])?;
    let after = observe(&env, &original, "retry-after", &["get", "gene", "KRAS"])?;
    if !before.iter().any(|item| item.status == Some(429)) {
        return Err("retry witness absent".into());
    }
    let mut changes = Vec::new();
    let mut sources = BTreeMap::new();
    live::compare(&before, &after, &mut changes, &mut sources);
    if !changes.is_empty() {
        return Err(format!("successful retry caused false drift: {changes:?}").into());
    }
    // Changed result pools alter identities without removing the endpoint contract.
    let mut dynamic = after.clone();
    for item in &mut dynamic {
        item.identity = format!("different-{}", item.identity);
    }
    live::compare(&after, &dynamic, &mut changes, &mut BTreeMap::new());
    if !changes
        .iter()
        .all(|item| item.contains("request-plan-changed"))
        || changes.is_empty()
    {
        return Err(format!("request plan mislabeled: {changes:?}").into());
    }
    changes.clear();
    let mut failed = after.clone();
    failed.first_mut().ok_or("missing real observation")?.status = Some(503);
    live::compare(&after, &failed, &mut changes, &mut BTreeMap::new());
    if !changes
        .iter()
        .any(|item| item.contains("Some(200) -> Some(503)"))
    {
        return Err("real HTTP status drift hidden".into());
    }
    witness_proof::sampling(&after)?;
    cache_after_transport(&after)?;
    bounded_command_drift(&after)?;
    report_context(&after)?;
    cache_reuse(&env, &original, &after)?;
    Ok(())
}
fn cache_reuse(
    env: &Environment,
    _original: &Path,
    after: &[Observation],
) -> Result<(), Box<dyn std::error::Error>> {
    // Reuse the actual warmed checked cache with networking refused.
    let records = env.root().join("known-recordings");
    response_cases::cache_capture("gene/main", &records, response_cases::CACHE_ACQUIRED)?;
    let cache = env.root().join("explicit-cache");
    let cache_arg = cache.to_str().ok_or("cache path")?;
    observe(
        env,
        &records,
        "warm",
        &["get", "gene", "KRAS", "--case-cache", cache_arg],
    )?;
    let report = env.root().join("cached.jsonl");
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &[
                "get".into(),
                "gene".into(),
                "KRAS".into(),
                "--offline".into(),
                "--case-cache".into(),
                cache_arg.into(),
                "--observe".into(),
                report.display().to_string(),
            ],
        )
        .output()?;
    if !output.status.success() {
        return Err("warm offline cache failed".into());
    }
    let cached: Vec<Observation> = fs::read_to_string(report)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    if cached.is_empty() || !cached.iter().all(|item| item.cached) {
        return Err("cache reuse not observed".into());
    }
    let mut changes = Vec::new();
    let mut sources = BTreeMap::new();
    live::compare(after, &cached, &mut changes, &mut sources);
    if !changes.is_empty() || !sources.is_empty() {
        return Err(format!("cache reuse caused false drift: {changes:?}").into());
    }
    Ok(())
}

fn report_context(observed: &[Observation]) -> Result<(), Box<dyn std::error::Error>> {
    let mut changes = Vec::new();
    let evidence = live::command_report(
        3,
        &json!({"status": "found"}),
        &json!({"status": "not-found"}),
        observed,
        observed,
        &mut changes,
        &mut BTreeMap::new(),
    );
    if evidence.get("ordinal") != Some(&json!(3))
        || evidence.get("before_status") != Some(&json!("found"))
        || evidence.get("after_status") != Some(&json!("not-found"))
        || evidence
            .pointer("/exchanges/items/0/role")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || evidence
            .pointer("/exchanges/items/0/before_statuses/values")
            .is_none()
    {
        return Err(format!("bounded command context missing: {evidence}").into());
    }
    let cases = vec![
        json!({"case": "changed-status", "result": {"drift": changes, "commands": [evidence]}}),
    ];
    let report = live::summary(cases, BTreeMap::new(), 0, true);
    if report.get("drift_cases") != Some(&json!(1))
        || report.pointer("/case_sample/0/result/commands/0/ordinal") != Some(&json!(3))
    {
        return Err("full report lost status context".into());
    }
    let first = observed.first().ok_or("missing observation")?;
    let many: Vec<_> = (0..40)
        .map(|index| {
            let mut item = first.clone();
            item.identity = format!("request-{index}");
            item
        })
        .collect();
    changes = Vec::new();
    let evidence = live::compare(&many, &[], &mut changes, &mut BTreeMap::new());
    if evidence.get("omitted") != Some(&json!(0))
        || evidence
            .get("items")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(40)
        || changes.len() != 40
        || changes.iter().any(|item| item.contains("omitted"))
    {
        return Err("complete exchange or drift details were discarded".into());
    }
    // Stable identities retain their exact match when an earlier result disappears.
    changes.clear();
    let surviving = many.get(1).ok_or("missing second request")?;
    let pair = many.get(..2).ok_or("missing pair")?;
    let evidence = live::compare(
        pair,
        std::slice::from_ref(surviving),
        &mut changes,
        &mut BTreeMap::new(),
    );
    if evidence.pointer("/items/0/kind") != Some(&json!("request-plan-removed"))
        || evidence.pointer("/items/1/kind") != Some(&json!("same-request"))
    {
        return Err("changed pool stole a surviving exact request".into());
    }
    Ok(())
}

fn cache_after_transport(observed: &[Observation]) -> Result<(), Box<dyn std::error::Error>> {
    let old = observed
        .iter()
        .find(|item| item.shape.header.is_some())
        .ok_or("missing actual bundle observation")?;
    let mut changed = old.clone();
    changed.shape.header = Some("changed header".into());
    let mut cached = changed.clone();
    cached.cached = true;
    cached.shape = Default::default();
    let mut changes = Vec::new();
    live::compare(
        std::slice::from_ref(old),
        &[changed, cached],
        &mut changes,
        &mut BTreeMap::new(),
    );
    if !changes
        .iter()
        .any(|item| item.contains("bundle header changed"))
    {
        return Err("later cache reuse hid a real transport contract change".into());
    }
    Ok(())
}

fn bounded_command_drift(observed: &[Observation]) -> Result<(), Box<dyn std::error::Error>> {
    let first = observed.first().ok_or("missing observation")?;
    let before: Vec<_> = (0..40)
        .map(|index| {
            let mut item = first.clone();
            item.identity = format!("request-{index}");
            item
        })
        .collect();
    let mut changes = Vec::new();
    for _ in 0..2 {
        let evidence = live::compare(&before, &[], &mut changes, &mut BTreeMap::new());
        if evidence.get("drift_omitted") != Some(&json!(0))
            || evidence
                .get("drift")
                .and_then(Value::as_array)
                .map(Vec::len)
                != Some(40)
        {
            return Err("complete per-command drift evidence missing".into());
        }
    }
    Ok(())
}
