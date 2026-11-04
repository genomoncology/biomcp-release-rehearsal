use super::launch::Environment;
tool_only! { use super::launch::ENV_PATHS; }
tool_only! { use crate::runner::transcript::trial as transcript_trial; }
tool_only! { use libtest_mimic::Trial; }
use std::{fs, path::Path};
tool_only! { use std::env; }

tool_only! { #[path = "fixture_evidence_proof.rs"] mod fixture_evidence_proof; }
tool_only! { #[path = "native_evidence_proof.rs"] mod native_evidence_proof; }

tool_only! { type Check = Result<(), String>; }

tool_only! {
pub(crate) fn trials() -> Vec<Trial> {
    let mut trials = vec![
        transcript_trial("article-semantic-sections/checked-edge-boundaries", || {
            super::semantic_tables::check().map_err(|e| e.to_string().into())
        }),
        transcript_trial("schema-discovery/rejects-increasing-citation-counts", || {
            let assertion = super::assertions::Assertion::Nonincreasing;
            if assertion.check("", &serde_json::json!([8, 9])).is_ok() {
                return Err("increasing counts accepted".into());
            }
            assertion
                .check("", &serde_json::json!([9, 9, 0]))
                .map_err(Into::into)
        }),
        transcript_trial("schema-discovery/null-and-empty-boundaries", || {
            boundaries().map_err(Into::into)
        }),
        transcript_trial("schema-discovery/rejects-bad-cases", || {
            negatives().map_err(Into::into)
        }),
        transcript_trial("schema-discovery/named-mcp-case", || {
            named_mcp_case().map_err(Into::into)
        }),
        transcript_trial("schema-discovery/selects-and-lists-cases", || {
            selection().map_err(Into::into)
        }),
        transcript_trial("schema-discovery/isolates-child-environments", || {
            isolation().map_err(Into::into)
        }),
        #[cfg(unix)]
        transcript_trial("schema-discovery/output-failure", || {
            output_failure().map_err(Into::into)
        }),
    ];
    trials.extend(fixture_evidence_proof::trials());
    trials.push(transcript_trial(
        "schema-discovery/persistent-native-evidence",
        || native_evidence_proof::check().map_err(|error| error.to_string().into()),
    ));
    trials
}
}

tool_only! {
fn put(root: &Path, relative: &str, step: &str) -> Check {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        path,
        format!(
            "feature: schema-discovery\nstory: Runner contract probe\nsteps:\n  - run: [schema, biomcp.source/1]\n{step}\n"
        ),
    )
    .map_err(|error| error.to_string())
}
}

tool_only! {
fn invoke(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut arguments = vec![
        "--cases".to_owned(),
        root.display().to_string(),
        "--color".into(),
        "never".into(),
    ];
    arguments.extend(args.iter().map(|value| {
        if args.contains(&"--exact") && value.ends_with(".yaml") { format!("schema-discovery/{value}") } else { (*value).to_owned() }
    }));
    Environment::new()
        .map_err(|error| error.to_string())?
        .command(
            env::current_exe().map_err(|error| error.to_string())?,
            &arguments,
        )
        .output()
        .map_err(|error| error.to_string())
}
}

tool_only! {
fn negatives() -> Check {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::write(directory.path().join("missing-floor.json"), "[\"missing\"]")
        .map_err(|e| e.to_string())?;
    let cases = [
        (
            "wrong-value",
            "    expect: {'/type': integer}",
            "/type: expected \"integer\", actual \"string\"",
        ),
        (
            "missing-null",
            "    expect: {'/absent': null}",
            "/absent: expected null, actual <missing>",
        ),
        (
            "missing-ceiling",
            "    expect: {'/absent': {max_bytes: 10}}",
            "actual <missing>",
        ),
        (
            "wrong-exit",
            "    exit: 2",
            "exit: expected 2, actual Some(0)",
        ),
        (
            "whole-bytes",
            "    max_bytes: 0",
            "stdout max_bytes: expected <= 0, actual",
        ),
        (
            "whole-tokens",
            "    max_tokens: 0",
            "stdout max_tokens: expected <= 0, actual",
        ),
        (
            "pointer-bytes",
            "    expect: {'/type': {max_bytes: 0}}",
            "/type max_bytes: expected <= 0, actual 8",
        ),
        (
            "pointer-tokens",
            "    expect: {'/type': {max_tokens: 1}}",
            "/type max_tokens: expected <= 1, actual 2",
        ),
    ];
    put(
        directory.path(),
        "markdown-pair.yaml",
        concat!(
            "  - run: [help, author, --format, json]\n  - run: [help, author, --format, md]\n",
            "    max_bytes: 8192\n    expect: {'': {includes_all_of: ",
            "['biomcp author papers', '--format', '--disable-sources']}}",
        ),
    )?;
    let accepted = invoke(directory.path(), &["markdown-pair.yaml", "--exact"])?;
    if !accepted.status.success() {
        return Err(format!("matching Markdown pair failed: {}", String::from_utf8_lossy(&accepted.stdout)));
    }
    for (name, step, diagnostic) in cases.into_iter().chain(super::negative_cases::cases()) {
        let filename = format!("{name}.yaml");
        put(directory.path(), &filename, step)?;
        let output = invoke(directory.path(), &[&filename, "--exact"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if output.status.success()
            || !stdout
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains(&format!("test schema-discovery/{filename} ... FAILED"))
            || !stdout.contains(diagnostic)
            || !stdout.contains("1 failed")
        {
            return Err(format!(
                "{name}: expected a selected failing case with {diagnostic}; actual {stdout}; stderr {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}
}

tool_only! {
fn selection() -> Check {
    empty_selection()?;
    #[cfg(unix)]
    literal_backslash_names()?;
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    put(
        directory.path(),
        "left/same.yaml",
        "    expect: {'/type': string}",
    )?;
    put(
        directory.path(),
        "right/same.yaml",
        "    expect: {'/type': integer}",
    )?;
    let all = invoke(directory.path(), &["--list"])?;
    let listing = String::from_utf8_lossy(&all.stdout);
    if !all.status.success()
        || !listing.contains("schema-discovery/selects-and-lists-cases: test")
        || !listing.contains("article-search/concurrent-process-pool-publication: test")
        || !listing.contains("variant-alphagenome/alphagenome-native-groups-local-tls: test")
    {
        return Err(format!(
            "custom case directory omitted feature trials: {listing}"
        ));
    }
    let listed = invoke(directory.path(), &["same.yaml", "--list"])?;
    let listing = String::from_utf8_lossy(&listed.stdout);
    if !listed.status.success()
        || !listing.contains("left/same.yaml: test")
        || !listing.contains("right/same.yaml: test")
    {
        return Err(format!(
            "distinct path names missing from --list: {listing}"
        ));
    }
    let output = invoke(directory.path(), &["left/same.yaml", "--exact"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || !stdout.contains("left/same.yaml ... ok")
        || !stdout.contains("1 passed")
        || !stdout.contains("filtered out")
    {
        return Err(format!(
            "--cases with Cargo filter did not select the passing case: {stdout}"
        ));
    }
    let filtered = invoke(directory.path(), &["left/", "--list"])?;
    let stdout = String::from_utf8_lossy(&filtered.stdout);
    if !filtered.status.success()
        || !stdout.contains("left/same.yaml")
        || stdout.contains("right/same.yaml")
    {
        return Err(format!("--cases with filtered --list: {stdout}"));
    }
    Ok(())
}
}

tool_only! {
fn empty_selection() -> Check {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    // Discovery rejects empty directories before applying any name filter.
    if invoke(directory.path(), &[])?.status.success() {
        return Err("empty discovery unexpectedly succeeded".into());
    }
    put(
        directory.path(),
        "selected.yaml",
        "    expect: {'/type': string}",
    )?;
    for args in [
        vec!["no-such-runner-selection-7e41"],
        vec!["no-such-runner-selection-7e41", "--exact"],
        vec!["no-such-runner-selection-7e41", "--list"],
        vec!["selected.yaml", "--exact", "--skip", "selected.yaml"],
        vec!["selected.yaml", "--exact", "--ignored"],
    ] {
        let filter = args.first().ok_or("missing name filter")?;
        let output = invoke(directory.path(), &args)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() || !stderr.contains(filter) {
            return Err(format!(
                "empty selection {args:?} must fail and name its filter; actual {:?}: {stderr}",
                output.status.code()
            ));
        }
    }
    for (args, expected) in [
        (vec!["selected.yaml", "--exact"], "1 passed"),
        (vec!["selected.yaml", "--exact", "--bench"], "1 ignored"),
    ] {
        let output = invoke(directory.path(), &args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() || !stdout.contains(expected) {
            return Err(format!(
                "selection {args:?}: expected {expected}; actual {stdout}"
            ));
        }
    }
    Ok(())
}
}

#[cfg(unix)]
tool_only! {
fn literal_backslash_names() -> Check {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    for name in ["left/same.yaml", "left\\same.yaml"] {
        put(directory.path(), name, "    expect: {'/type': string}")?;
    }
    let output = invoke(directory.path(), &["left\\same.yaml", "--exact"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || !stdout.contains("left\\same.yaml ... ok")
        || !stdout.contains("1 passed")
        || !stdout.contains("filtered out")
    {
        return Err(format!("literal filename must remain distinct: {stdout}"));
    }
    Ok(())
}
}

tool_only! {
fn isolation() -> Check {
    let before: Vec<_> = env::vars_os().collect();
    let first = Environment::new().map_err(|error| error.to_string())?;
    let second = Environment::new().map_err(|error| error.to_string())?;
    let profiles = tempfile::tempdir().map_err(|error| error.to_string())?;
    let profile = env::var_os("LLVM_PROFILE_FILE")
        .unwrap_or_else(|| profiles.path().join("probe-%m-%p.profraw").into_os_string());
    let read = |environment: &Environment| -> Result<serde_json::Value, String> {
        let output = environment
            .command(
                env::current_exe().map_err(|error| error.to_string())?,
                &["--runner-environment-probe".into()],
            )
            .env("LLVM_PROFILE_FILE", &profile)
            .env("NCBI_API_KEY", "hostile-test-credential")
            .env("HTTPS_PROXY", "http://hostile.invalid")
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err("environment child failed".into());
        }
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
    };
    let first_step = read(&first)?;
    let next_step = read(&first)?;
    let other_case = read(&second)?;
    for (key, _) in ENV_PATHS {
        let first = first_step
            .get(key)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("child lacks {key}"))?;
        if next_step.get(key).and_then(serde_json::Value::as_str) != Some(first)
            || other_case.get(key).and_then(serde_json::Value::as_str) == Some(first)
        {
            return Err(format!(
                "{key}: case steps must share environment; cases must differ"
            ));
        }
    }
    let nested_profile = Path::new(&profile)
        .parent()
        .ok_or("profile directory missing")?
        .join("harness-%m-%p.profraw");
    for child in [&first_step, &next_step, &other_case] {
        let nested = child.get("nested").ok_or("nested probe missing")?;
        for (values, expected) in [(child, Path::new(&profile)), (nested, nested_profile.as_path())] {
            if values
                .get("LLVM_PROFILE_FILE")
                .and_then(serde_json::Value::as_str)
                != expected.to_str()
            {
                return Err("nested profiling must retain its directory and harness classification".into());
            }
        }
        for key in ["NCBI_API_KEY", "HTTPS_PROXY"] {
            if child.get(key).is_none_or(serde_json::Value::is_null)
                || nested.get(key) != Some(&serde_json::Value::Null)
            {
                return Err(format!("nested launch failed to clear {key}"));
            }
        }
    }
    if before != env::vars_os().collect::<Vec<_>>() {
        return Err("runner changed process environment".into());
    }
    Ok(())
}
}

#[cfg(unix)]
tool_only! {
fn output_failure() -> Check {
    // An unconnected datagram rejects writes with a non-BrokenPipe error.
    // Unlike a read-only file, Rust stdout does not silently treat it as closed.
    let socket = std::os::unix::net::UnixDatagram::unbound().map_err(|e| e.to_string())?;
    let writer: std::os::fd::OwnedFd = socket.into();
    let output = Environment::new()
        .map_err(|error| error.to_string())?
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &["schema".into(), "biomcp.source/1".into()],
        )
        .stdout(writer)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.code() != Some(5) || output.stderr != b"biomcp: output failed\n" {
        return Err(format!(
            "output failure: expected exit 5 and fixed diagnostic, actual {:?} {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
}

// This probe belongs only to the test executable. It observes the same child
// launch boundary as case steps without adding any command to the product.
tool_only! {
pub(crate) fn environment_probe() -> std::process::ExitCode {
    let mut values: serde_json::Map<_, _> = ENV_PATHS
        .iter()
        .map(|(key, _)| *key)
        .chain(["LLVM_PROFILE_FILE", "NCBI_API_KEY", "HTTPS_PROXY"])
        .map(|key| (key.to_owned(), serde_json::json!(env::var(key).ok())))
        .collect();
    if env::args().nth(2).as_deref() != Some("--leaf") {
        let nested = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let output = Environment::new()?
                .command(
                    env::current_exe()?,
                    &["--runner-environment-probe".into(), "--leaf".into()],
                )
                .output()?;
            if !output.status.success() {
                return Err("nested probe failed".into());
            }
            Ok(serde_json::from_slice(&output.stdout)?)
        })();
        let Ok(nested) = nested else {
            return std::process::ExitCode::FAILURE;
        };
        values.insert("nested".into(), nested);
    }
    match serde_json::to_writer(std::io::stdout().lock(), &values) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
}

// Schemas have no null-valued fields or empty arrays, so these traversal states
// need one direct boundary table. All other assertion proof uses schema commands.
tool_only! {
fn boundaries() -> Check {
    for expect in [
        "{'/items/*': []}",
        "{'/items/*': {equals: []}}",
        "{'/items/*': {count: 0}}",
    ] {
        let step: super::Step = serde_saphyr::from_str(&format!("run: []\nexpect: {expect}"))
            .map_err(|error| error.to_string())?;
        let result = super::assertions::prepare(&step);
        let expected = "/items/*: wildcard cannot assert an empty array; assert on the parent array instead, such as {'/items': {equals: [], count: 0}}";
        if result.as_ref().err().map(String::as_str) != Some(expected) {
            return Err(format!("{expect}: expected {expected}, actual {result:?}"));
        }
    }
    let rows = [
        ("{}", "{'': {omits: [missing]}}", None),
        (
            "{\"x\":null}",
            "{'': {omits: [x]}}",
            Some("unexpected field x"),
        ),
        (
            "{\"x\":1}",
            "{'': {omits: [x]}}",
            Some("unexpected field x"),
        ),
        ("[]", "{'': {omits: [x]}}", Some("requires an object")),
        ("{}", "{'/missing': {omits: [x]}}", Some("<missing>")),
        ("null", "{'': null}", None),
        ("null", "{'/missing': null}", Some("<missing>")),
        ("null", "{'/*': []}", Some("wildcard requires an array")),
        ("[]", "{'': {equals: [], count: 0}}", None),
        ("[]", "{'': {includes: null}}", Some("expected member null")),
        (
            "[]",
            "{'/*': {equals: [], count: 0}}",
            Some("empty wildcard selection"),
        ),
        ("[null]", "{'/*': [null]}", None),
        ("[[]]", "{'/*/*': []}", Some("empty wildcard selection")),
        ("[{\"x\":null},{}]", "{'/*/x': [null]}", Some("<missing>")),
        (
            "[{\"x\":null},{\"x\":[]}]",
            "{'/*/x/*': []}",
            Some("wildcard requires an array"),
        ),
    ];
    for (stdout, expect, error) in rows {
        let step: super::Step = serde_saphyr::from_str(&format!("run: []\nexpect: {expect}"))
            .map_err(|error| error.to_string())?;
        // Keep selector failures covered independently of preparation rejection.
        let result = if matches!(
            error,
            Some("empty wildcard selection" | "wildcard requires an array")
        ) {
            let value = serde_json::from_str(stdout).map_err(|error| error.to_string())?;
            let pointer = step.expect.keys().next().ok_or("missing pointer")?;
            super::assertions::select(&value, pointer).map(|_| ())
        } else {
            let assertions = super::assertions::prepare(&step)?;
            super::assertions::check(stdout.as_bytes(), &step, &assertions)
        };
        match (result, error) {
            (Ok(()), None) => (),
            (Err(actual), Some(expected)) if actual.contains(expected) => (),
            (actual, expected) => {
                return Err(format!(
                    "{stdout}, {expect}: expected {expected:?}, actual {actual:?}"
                ));
            }
        }
    }
    Ok(())
}
}

// Native proofs own these directories outside Environment's temporary root.
pub(super) fn retained_cli(
    env: &Environment,
    args: &[String],
    exit: i32,
    directory: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir(directory).map_err(|e| e.to_string())?;
    fs::write(
        directory.join("args.json"),
        serde_json::to_vec(args).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .output()
        .map_err(|e| e.to_string())?;
    fs::write(directory.join("cli.stdout"), &output.stdout).map_err(|e| e.to_string())?;
    fs::write(directory.join("cli.stderr"), &output.stderr).map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) || !output.stderr.is_empty() {
        return Err(format!(
            "native CLI exit {:?}, expected {exit}",
            output.status.code()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
pub(super) fn retained_both(
    env: &Environment,
    args: &[String],
    exit: i32,
    directory: &Path,
) -> Result<serde_json::Value, String> {
    let value = retained_cli(env, args, exit, directory)?;
    let start = args
        .iter()
        .position(|s| matches!(s.as_str(), "get" | "search"))
        .ok_or("missing native command")?;
    let (globals, command) = args.split_at(start);
    let mcp = super::mcp::call_retained(env, command, exit >= 2, globals, Some(directory))?;
    if value != mcp {
        return Err("native CLI and MCP differ".into());
    }
    super::schema::Registry::embedded()?.validate(&value, command, exit < 2)?;
    Ok(value)
}

tool_only! {
fn named_mcp_case() -> Check {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::write(directory.path().join("named.yaml"), r#"feature: schema-discovery
story: Named tool checks its structured answer
steps:
  - mcp_call:
      name: biomcp
      arguments:
        args: [help, get, gene]
    expect:
      /result/commands/0/command: [get, gene]
"#).map_err(|error| error.to_string())?;
    let output = invoke(directory.path(), &["named.yaml", "--exact"])?;
    if !output.status.success() {
        return Err(format!("named MCP case failed: {} {}",
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
}
}
