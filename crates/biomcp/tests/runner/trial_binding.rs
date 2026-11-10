//! Registry continuation capabilities bind selectors and consumed native positions.
use super::{configuration::invoke, http::Server, launch::Environment, mcp, network_stub::Stub};
use serde_json::{Value, json};
use std::sync::mpsc;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    [false, true]
        .into_iter()
        .flat_map(|meta| [false, true].map(move |broad| (meta, broad)))
        .map(|(meta, broad)| {
            super::transcript::trial(
                format!(
                    "trial-search/bound-continuation-{}-{}",
                    if broad { "broader" } else { "native" },
                    if meta { "mcp" } else { "cli" }
                ),
                move || check(meta, broad).map_err(Into::into),
            )
        })
        .chain([false, true].map(|broad| {
            super::transcript::trial(
                if broad {
                    "trial-search/hosted-broader-continuation-crosses-independent-stores"
                } else {
                    "trial-search/hosted-continuation-crosses-independent-stores"
                },
                move || cross_store(broad).map_err(Into::into),
            )
        }))
        .chain([false, true].map(|portable| {
            super::transcript::trial(
                if portable {
                    "trial-search/portable-continuation-carries-expansion-source"
                } else {
                    "trial-search/stored-continuation-carries-expansion-source"
                },
                move || carried_source(portable).map_err(Into::into),
            )
        }))
        .collect()
}

fn registry(broad: bool) -> Result<(Stub, mpsc::Receiver<String>), String> {
    let (observed, calls) = mpsc::channel();
    let stub = Stub::for_sources(
        &["clinicaltrials.gov", "mychem.info"],
        |_| "application/json",
        move |request| {
            observed
                .send(request.lines().next().unwrap_or_default().to_owned())
                .map_err(|_| "request observer closed")?;
            if request.contains("/v1/query?") {
                return Ok(
                    json!({"total":1,"hits":[{"_id":"KTUFNOKKBVMGRW-UHFFFAOYSA-N",
                "drugcentral":{"synonyms":["Typed drug","Brand alias"]}}]})
                    .to_string(),
                );
            }
            if request.contains("/version") {
                return Ok(json!({"dataTimestamp":"2026-11-10T00:00:00Z"}).to_string());
            }
            if broad && !request.contains("query.spons=") {
                return Ok(json!({"totalCount":0,"studies":[]}).to_string());
            }
            let position = if request.contains("pageToken=native-four") {
                4
            } else if request.contains("pageToken=native-two") {
                2
            } else {
                0
            };
            let row = |id: &str| {
                json!({"protocolSection":{"identificationModule":{"nctId":id,
            "briefTitle":"Synthetic bound cursor"},"statusModule":{"overallStatus":"RECRUITING"},
            "conditionsModule":{"conditions":["Other condition"]}}})
            };
            let rows = match position {
                0 => vec![row("NCT00000001"), row("invalid")],
                2 => vec![row("NCT00000003"), row("NCT00000004")],
                _ => vec![row("invalid"), row("invalid")],
            };
            Ok(
                json!({"totalCount":8,"studies":rows,"nextPageToken":match position {
            0=>"native-two",2=>"native-four",_=>"native-six"}})
                .to_string(),
            )
        },
    )?;
    Ok((stub, calls))
}

fn config(stub: &Stub, assets: &Environment) -> Result<Value, String> {
    let ca = assets.root().join("registry-ca.pem");
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    Ok(
        json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "disable_sources":["mychem","chembl","openfda-label"]}),
    )
}

fn setup(broad: bool) -> Result<(Environment, Stub, mpsc::Receiver<String>), String> {
    let (stub, calls) = registry(broad)?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    stub.retain(env.shared_root());
    env.config(&config(&stub, &env)?)
        .map_err(|e| e.to_string())?;
    Ok((env, stub, calls))
}

fn cross_store(broad: bool) -> Result<(), String> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let (stub, calls) = registry(broad)?;
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let config = config(&stub, &assets)?;
    let start = |key| hosted_server(&config, key);
    if !broad && start(Some("invalid")).is_ok() {
        return Err("a malformed configured cursor key started a hosted task".into());
    }
    let key = URL_SAFE_NO_PAD.encode([1; 32]);
    let mut first = start(Some(&key))?;
    let mut other = start(Some(&key))?;
    stub.retain(first.environment().shared_root());
    stub.retain(other.environment().shared_root());
    if first.environment().root() == other.environment().root() {
        return Err("the two hosted cores must have independent stores".into());
    }
    let initial = hosted_call(
        &first,
        &[
            "search",
            "trial",
            "--sponsor",
            "Synthetic Registry",
            "--condition",
            "pancreatic",
            "--limit",
            "2",
            "--exact-term",
        ],
        "",
    )?;
    let next = super::action(&initial, "page")?;
    let token = next
        .windows(2)
        .find(|pair| pair.first().is_some_and(|option| option == "--next-page"))
        .and_then(|pair| pair.get(1))
        .filter(|token| token.starts_with("biomcp:ctgov-page:v2:"))
        .ok_or("hosted query did not issue a portable cursor")?;
    let words: Vec<_> = next.iter().map(String::as_str).collect();
    for _ in calls.try_iter() {}
    let native = followed_cursor(&first, &words, &calls)?;
    if followed_cursor(&other, &words, &calls)? != native {
        return Err("independent task changed the native cursor or selectors".into());
    }
    first.stop()?;
    let mut replacement = start(Some(&key))?;
    if followed_cursor(&replacement, &words, &calls)? != native {
        return Err("replacement task changed the native cursor or selectors".into());
    }
    hosted_refusals(&other, &next, token, &calls)?;
    let wrong_key = URL_SAFE_NO_PAD.encode([2; 32]);
    let mut wrong = start(Some(&wrong_key))?;
    hosted_refusal(&wrong, &words, "", &calls)?;
    wrong.stop()?;
    replacement.stop()?;
    other.stop()?;
    stub.finish()?;
    Ok(())
}

fn hosted_server(config: &Value, key: Option<&str>) -> Result<Server, String> {
    Server::configured_globals(
        "127.0.0.1:0",
        Some(config.clone()),
        None,
        key.map(|value| ("BIOMCP_TRIAL_CONTINUATION_KEY", value)),
        None,
        &["--hosted".into()],
    )
}

fn carried_source(portable: bool) -> Result<(), String> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let (stub, calls) = registry(false)?;
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let mut config = config(&stub, &assets)?;
    *config
        .get_mut("disable_sources")
        .ok_or("missing source configuration")? = json!(["chembl", "openfda-label"]);
    let key = portable.then(|| URL_SAFE_NO_PAD.encode([1; 32]));
    let mut first = hosted_server(&config, key.as_deref())?;
    stub.retain(first.environment().shared_root());
    let initial = hosted_call(
        &first,
        &[
            "search",
            "trial",
            "--intervention",
            "Typed drug",
            "--limit",
            "2",
        ],
        "",
    )?;
    let requests: Vec<_> = calls.try_iter().collect();
    if !requests
        .iter()
        .any(|request| request.contains("/v1/query?"))
        || !requests
            .iter()
            .any(|request| request.contains("Brand+alias") || request.contains("Brand%20alias"))
    {
        return Err(format!(
            "first page did not use a source-resolved alias: {requests:?}"
        ));
    }
    let mut second = portable
        .then(|| hosted_server(&config, key.as_deref()))
        .transpose()?;
    let server = second.as_ref().unwrap_or(&first);
    let page_two = super::action(&initial, "page")?;
    let page_two: Vec<_> = page_two.iter().map(String::as_str).collect();
    let continued = hosted_call(server, &page_two, "")?;
    let requests: Vec<_> = calls.try_iter().collect();
    if requests
        .iter()
        .any(|request| request.contains("/v1/query?"))
        || !requests.iter().any(|request| {
            request.contains("pageToken=native-two")
                && (request.contains("Brand+alias") || request.contains("Brand%20alias"))
        })
        || continued.pointer("/page/offset") != Some(&json!(2))
        || continued
            .get("sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources
                    .iter()
                    .any(|source| source.get("source") == Some(&json!("mychem")))
            })
    {
        return Err(format!(
            "continuation lost the expanded query or repeated resolution: {requests:?} {continued}"
        ));
    }
    let page_three = super::action(&continued, "page")?;
    let page_three: Vec<_> = page_three.iter().map(String::as_str).collect();
    hosted_refusal(
        server,
        &page_three,
        "x-biomcp-exclude-sources: mychem\r\n",
        &calls,
    )?;
    if let Some(server) = &mut second {
        server.stop()?;
    } else {
        check_distinct_literal(&first, page_two)?;
    }
    first.stop()?;
    stub.finish()?;
    Ok(())
}

fn check_distinct_literal(first: &Server, expanded_page: Vec<&str>) -> Result<(), String> {
    let literal = hosted_call(
        first,
        &[
            "search",
            "trial",
            "--exact-term",
            "--intervention",
            "(\"Brand alias\" OR \"Typed drug\")",
            "--limit",
            "2",
        ],
        "x-biomcp-exclude-sources: mychem\r\n",
    )?;
    let literal_page = super::action(&literal, "page")?;
    if literal_page.iter().map(String::as_str).eq(expanded_page) {
        return Err(
            "identical native cursor and expression aliased distinct stored dependencies".into(),
        );
    }
    let words: Vec<_> = literal_page.iter().map(String::as_str).collect();
    let literal_second = hosted_call(first, &words, "x-biomcp-exclude-sources: mychem\r\n")?;
    if literal_second.pointer("/page/offset") != Some(&json!(2)) {
        return Err(format!(
            "literal continuation acquired the other cursor's dependency: {literal_second}"
        ));
    }
    Ok(())
}

fn followed_cursor(
    server: &Server,
    words: &[&str],
    calls: &mpsc::Receiver<String>,
) -> Result<String, String> {
    let reply = hosted_call(server, words, "")?;
    let requests: Vec<_> = calls
        .try_iter()
        .filter(|request| request.contains("pageToken=native-two"))
        .collect();
    if requests.len() != 1
        || reply.pointer("/page/offset") != Some(&json!(2))
        || reply.pointer("/page/returned") != Some(&json!(2))
    {
        return Err(format!(
            "expected one native continuation at consumed offset two, observed {}: {reply}",
            requests.len()
        ));
    }
    requests
        .into_iter()
        .next()
        .ok_or_else(|| "missing native continuation".into())
}

fn hosted_refusals(
    server: &Server,
    next: &[String],
    token: &str,
    calls: &mpsc::Receiver<String>,
) -> Result<(), String> {
    let mut tampered = token.to_owned();
    let index = "biomcp:ctgov-page:v2:".len();
    let first = tampered
        .get(index..index + 1)
        .ok_or("missing encoded cursor")?;
    let replacement = if first == "A" { "B" } else { "A" };
    tampered.replace_range(index..index + 1, replacement);
    for altered in [
        replace(next, "--offset", "40")?,
        replace(next, "--sponsor", "Other")?,
        next.iter()
            .cloned()
            .chain(["--exact-condition".into()])
            .collect(),
        replace(next, "--next-page", &tampered)?,
        replace(next, "--next-page", "native-two")?,
        replace(
            next,
            "--next-page",
            &format!("biomcp:ctgov-page:v2:{}", "A".repeat(12_000)),
        )?,
    ] {
        let words: Vec<_> = altered.iter().map(String::as_str).collect();
        hosted_refusal(server, &words, "", calls)?;
    }
    let words: Vec<_> = next.iter().map(String::as_str).collect();
    hosted_refusal(server, &words, "x-biomcp-exclude-sources: ctgov\r\n", calls)
}

fn hosted_refusal(
    server: &Server,
    words: &[&str],
    headers: &str,
    calls: &mpsc::Receiver<String>,
) -> Result<(), String> {
    let reply = hosted_call(server, words, headers)?;
    if calls.try_iter().next().is_some()
        || reply.pointer("/error/option") != Some(&json!("--next-page"))
        || !reply
            .pointer("/error/message")
            .and_then(Value::as_str)
            .is_some_and(|message| message.starts_with("this trial continuation cannot verify"))
    {
        return Err(format!(
            "hosted cursor refusal lost local admission: {reply}"
        ));
    }
    Ok(())
}

fn hosted_call(server: &Server, args: &[&str], extra: &str) -> Result<Value, String> {
    let frame = mcp::request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":args}}),
    );
    let (status, reply) = server.request_with_headers(
        "POST",
        "/mcp",
        &frame.to_string(),
        &format!("x-biomcp-use: restricted\r\n{extra}"),
    )?;
    if status != 200 {
        return Err(format!("hosted HTTP status {status}: {reply}"));
    }
    reply
        .pointer("/result/structuredContent")
        .cloned()
        .ok_or_else(|| format!("missing hosted content: {reply}"))
}

fn call(env: &Environment, args: &[String], meta: bool, error: bool) -> Result<Value, String> {
    if meta {
        mcp::call_with_globals(env, args, error, &[])
    } else {
        invoke(env, args.to_vec(), if error { 2 } else { 0 })
    }
}
fn replace(args: &[String], key: &str, value: &str) -> Result<Vec<String>, String> {
    let mut changed = args.to_vec();
    let index = changed
        .iter()
        .position(|arg| arg == key)
        .ok_or("missing continuation selector")?
        + 1;
    *changed.get_mut(index).ok_or("missing selector value")? = value.into();
    Ok(changed)
}

fn check(meta: bool, broad: bool) -> Result<(), String> {
    let (env, stub, calls) = setup(broad)?;
    let args: Vec<_> = [
        "search",
        "trial",
        "--sponsor",
        "Synthetic Registry",
        "--condition",
        "pancreatic",
        "--phase",
        "1",
        "--limit",
        "2",
        "--exact-term",
    ]
    .map(str::to_owned)
    .into();
    let first = call(&env, &args, meta, false)?;
    let next = super::action(&first, "page")?;
    if first.pointer("/page/returned") != Some(&json!(1)) {
        return Err(format!("first page lacks bound native position: {first}"));
    }
    for changed in [
        replace(&next, "--offset", "40")?,
        replace(&next, "--phase", "2")?,
        replace(&next, "--sponsor", "Other")?,
        next.iter()
            .cloned()
            .chain(["--exact-condition".into()])
            .collect(),
        replace(&next, "--next-page", "native-two")?,
        replace(&next, "--next-page", "biomcp:ctgov-names:v1:native-two")?,
        replace(
            &next,
            "--next-page",
            &format!("biomcp:ctgov-page:v1:{}", "0".repeat(64)),
        )?,
    ] {
        refused(&env, &changed, meta, &calls, true)?;
    }
    let mut excluded = next.clone();
    excluded.extend(["--disable-sources".into(), "ctgov".into()]);
    refused(&env, &excluded, meta, &calls, false)?;
    let restart = refused(&env, &replace(&next, "--offset", "40")?, meta, &calls, true)?;
    call(&env, &restart, meta, false)?;
    let second = call(&env, &next, meta, false)?;
    if second.pointer("/page/offset") != Some(&json!(2))
        || second.pointer("/page/returned") != Some(&json!(2))
    {
        return Err(format!(
            "valid bound cursor lost consumed position: {second}"
        ));
    }
    let exact = super::action(&second, "disambiguate")?;
    if exact
        .iter()
        .any(|word| matches!(word.as_str(), "--offset" | "--next-page"))
    {
        return Err(format!(
            "exact-condition restart retained position: {exact:?}"
        ));
    }
    call(&env, &exact, meta, false)?;
    let third = call(&env, &super::action(&second, "page")?, meta, false)?;
    let fourth = super::action(&third, "page")?;
    if third.pointer("/page/offset") != Some(&json!(4))
        || third.pointer("/page/returned") != Some(&json!(0))
        || !fourth.windows(2).any(|pair| pair == ["--offset", "6"])
    {
        return Err(format!(
            "all-refused page did not consume native rows: {third}"
        ));
    }
    stub.finish()?;
    Ok(())
}

fn refused(
    env: &Environment,
    args: &[String],
    meta: bool,
    calls: &mpsc::Receiver<String>,
    restart_allowed: bool,
) -> Result<Vec<String>, String> {
    for _ in calls.try_iter() {}
    let reply = call(env, args, meta, true)?;
    if calls.try_iter().next().is_some()
        || reply.pointer("/error/option") != Some(&json!("--next-page"))
        || !reply
            .pointer("/error/message")
            .and_then(Value::as_str)
            .is_some_and(|message| message.starts_with("this trial continuation cannot verify"))
    {
        return Err(format!(
            "unbound cursor reached registry or lost guidance: {reply}"
        ));
    }
    if !restart_allowed {
        if reply.get("next").is_some() {
            return Err(format!(
                "disabled source kept an executable next step: {reply}"
            ));
        }
        return Ok(Vec::new());
    }
    let fresh = super::action(&reply, "configure")?;
    if !fresh.iter().any(|word| word == "--fresh")
        || fresh
            .iter()
            .any(|word| matches!(word.as_str(), "--offset" | "--next-page"))
    {
        return Err(format!("invalid fresh recovery: {fresh:?}"));
    }
    Ok(fresh)
}
