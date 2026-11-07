//! Feature mcp-hosted: each hosted call owns its source credentials.
use super::{http::Server, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check<T = ()> = Result<T, String>;
const CALLER: &str = "synthetic-request-caller";
const SERVER: &str = "synthetic-request-server";
pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-hosted/keys/discovery", || {
            discovery().map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/keys/licensed-request-snapshot", || {
            licensed().map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/keys/public-wire-fallback", || {
            public().map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/keys/stdio-license", || {
            stdio().map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/keys/startup-recovery", || {
            recovery(false).map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/keys/recovery-freezes-static-settings", || {
            recovery(true).map_err(Into::into)
        }),
    ]
}
fn recovery(freeze: bool) -> Check {
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"proxy":"ftp://127.0.0.1:1"})),
        None,
        None,
        None,
        &["--hosted".into()],
    )?;
    server
        .environment()
        .config(&json!({"keys":{"oncokb_api_key":SERVER}}))
        .map_err(|e| e.to_string())?;
    let args = [
        "get",
        "variant",
        "chr7:g.140753336A>T",
        "oncokb",
        "--build",
        "GRCh38",
        "--tumor-type",
        "Synthetic tumor",
        "--offline",
    ];
    for supplied in [true, false] {
        let extra = if supplied {
            format!("x-biomcp-key-oncokb-api-key: {CALLER}\r\n")
        } else {
            String::new()
        };
        let reply = call(
            &server,
            &args,
            &format!("{extra}x-biomcp-use: academic\r\n"),
            false,
        )?;
        let content = reply
            .pointer("/result/structuredContent")
            .ok_or("missing recovery content")?;
        if supplied {
            if content.get("error").is_some()
                || !content["sources"].as_array().is_some_and(|sources| {
                    sources
                        .iter()
                        .any(|source| source["source"] == "oncokb" && source["reason"] == "offline")
                })
            {
                return Err("startup recovery dropped the caller credential".into());
            }
            if freeze {
                server
                    .environment()
                    .config(&json!({
                        "proxy":"ftp://127.0.0.1:1", "keys":{"oncokb_api_key":SERVER}
                    }))
                    .map_err(|error| error.to_string())?;
            }
        } else if content.pointer("/error/source") != Some(&json!("oncokb")) {
            return Err("startup recovery retained a previous caller credential".into());
        }
    }
    server.stop()
}
fn discovery() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let listed = super::key_registration::invoke(&env, &["key", "list"], "", 0)?;
    let entries = listed
        .pointer("/result/entries")
        .and_then(Value::as_array)
        .ok_or("missing entries")?;
    for entry in entries {
        let config = entry["config"].as_str().ok_or("missing config name")?;
        let header = format!(
            "x-biomcp-key-{}",
            config
                .strip_prefix("keys.")
                .ok_or("config prefix")?
                .replace('_', "-")
        );
        let tier = if matches!(entry["source"].as_str(), Some("oncokb" | "alphagenome")) {
            "no-fallback"
        } else {
            "fallback"
        };
        if entry.get("header") != Some(&json!(header)) || entry.get("tier") != Some(&json!(tier)) {
            return Err("key discovery omitted its header or declared tier".into());
        }
    }
    Ok(())
}
pub(super) fn call(
    server: &Server,
    args: &[&str],
    extra: &str,
    extra_argument: bool,
) -> Check<Value> {
    let mut arguments = json!({"args":args});
    if extra_argument {
        arguments
            .as_object_mut()
            .ok_or("arguments object")?
            .insert("ncbi_api_key".into(), json!(CALLER));
    }
    let frame = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"biomcp","arguments":arguments}});
    let headers = format!(
        "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-11-25\r\n{extra}"
    );
    let (status, reply, _) = server.raw(
        "POST",
        "/mcp",
        &server.address.to_string(),
        &headers,
        &frame.to_string(),
    )?;
    let reply = reply.ok_or("missing reply")?;
    if status != 200
        || [CALLER, SERVER]
            .iter()
            .any(|key| reply.to_string().contains(key))
    {
        return Err("request rejected or credential escaped in reply".into());
    }
    if let Some(context) = super::transcript::current() {
        let args: Vec<String> = args.iter().map(|arg| (*arg).into()).collect();
        let key = context.allocate(&args);
        context.record(
            &key,
            json!({"args":args,"surface":"http","http_status":status,
            "supplied_header":!extra.is_empty(),"extra_argument":extra_argument,
            "reply":reply}),
        )?;
    }
    Ok(reply)
}
fn licensed() -> Check {
    let replay = super::launch::fixtures().join("replies/gene/oncokb-synthetic/data");
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let scoped = super::licensed_cache::scoped_gene_fixture(&assets, CALLER)?;
    for hosted in [true, false] {
        let globals = if hosted {
            vec!["--hosted".into()]
        } else {
            vec![]
        };
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(json!({"keys":{"oncokb_api_key":SERVER}})),
            Some(if hosted {
                scoped.clone()
            } else {
                replay.clone()
            }),
            None,
            None,
            &globals,
        )?;
        let args = ["get", "gene", "TP53", "oncokb"];
        let headers: &[bool] = if hosted {
            &[false, true, false]
        } else {
            &[true]
        };
        for &header in headers {
            let extra = if header {
                format!("X-BioMCP-Key-Oncokb-Api-Key: {CALLER}\r\n")
            } else {
                String::new()
            };
            let reply = call(
                &server,
                &args,
                &format!("{extra}x-biomcp-use: academic\r\n"),
                false,
            )?;
            if hosted && !header {
                if reply.pointer("/result/structuredContent/error/message")
                    != Some(&json!("add the source key in your account settings"))
                    || reply.pointer("/result/structuredContent/error/source")
                        != Some(&json!("oncokb"))
                {
                    return Err(
                        "licensed source used a server key or omitted account guidance".into(),
                    );
                }
            } else if reply
                .pointer("/result/structuredContent/result/items/0/sections/oncokb/state")
                != Some(&json!("data"))
            {
                return Err("request credential failed to admit recorded licensed reply".into());
            }
        }
        if hosted {
            concurrent(&args)?;
        }
        let reply = call(&server, &args, "", true)?;
        if reply.pointer("/error/code") != Some(&json!(-32602)) {
            return Err("tool argument accepted a source key".into());
        }
        header_boundaries(&server, hosted)?;
        server.stop()?;
    }
    Ok(())
}
fn header_boundaries(server: &Server, hosted: bool) -> Check {
    let frame = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"biomcp","arguments":{"args":["help"]}}});
    for extra in [
        format!(
            "x-biomcp-key-oncokb-api-key: {CALLER}\r\nX-BIOMCP-KEY-ONCOKB-API-KEY: {SERVER}\r\n"
        ),
        "x-biomcp-key-oncokb-api-key: \r\n".into(),
    ] {
        let headers = format!(
            "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-11-25\r\n{extra}"
        );
        let (status, reply, _) = server.raw(
            "POST",
            "/mcp",
            &server.address.to_string(),
            &headers,
            &frame.to_string(),
        )?;
        if status != if hosted { 400 } else { 200 }
            || reply.as_ref().is_some_and(|value| {
                [CALLER, SERVER]
                    .iter()
                    .any(|key| value.to_string().contains(key))
            })
        {
            return Err("ambiguous credential headers were accepted or reflected".into());
        }
    }
    Ok(())
}
fn concurrent(args: &[&str]) -> Check {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let scoped = super::licensed_cache::scoped_gene_fixture(&assets, CALLER)?;
    // Both callers run the public gene join before the hosted key refusal.
    // Each caller needs its own identical exchange in the shared replay cursor.
    for entry in std::fs::read_dir(scoped.join("mygene")).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let mut record: Value =
            serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let exchanges = record
            .get_mut("exchanges")
            .and_then(Value::as_array_mut)
            .ok_or("MyGene replay exchanges")?;
        if exchanges.len() != 1 {
            return Err("concurrent MyGene fixture requires one exchange per caller".into());
        }
        exchanges.extend(exchanges.clone());
        std::fs::write(
            &path,
            serde_json::to_vec(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"keys":{"oncokb_api_key":SERVER}})),
        Some(scoped),
        None,
        None,
        &["--hosted".into()],
    )?;
    let extra = format!("x-biomcp-key-oncokb-api-key: {CALLER}\r\nx-biomcp-use: academic\r\n");
    let (keyed, unkeyed) = std::thread::scope(|scope| {
        let keyed = scope.spawn(|| call(&server, args, &extra, false));
        let unkeyed = scope.spawn(|| call(&server, args, "x-biomcp-use: academic\r\n", false));
        Ok::<_, String>((
            keyed.join().map_err(|_| "keyed caller stopped")??,
            unkeyed.join().map_err(|_| "unkeyed caller stopped")??,
        ))
    })?;
    if let Some(context) = super::transcript::current() {
        let key = context.allocate(&["concurrent-licensed-calls".into()]);
        context.record(
            &key,
            json!({"surface":"http-concurrent","keyed_reply":keyed,"unkeyed_reply":unkeyed}),
        )?;
    }
    if keyed.pointer("/result/isError") != Some(&json!(false))
        || !keyed
            .pointer("/result/structuredContent/sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources
                    .iter()
                    .any(|source| source["source"] == "mygene" && source["state"] == "ok")
            })
    {
        return Err("concurrent keyed caller exhausted the public gene fixture".into());
    }
    if keyed.pointer("/result/structuredContent/result/items/0/sections/oncokb/state")
        != Some(&json!("data"))
        || unkeyed.pointer("/result/structuredContent/error/source") != Some(&json!("oncokb"))
    {
        return Err("concurrent callers shared a credential snapshot".into());
    }
    server.stop()?;
    Ok(())
}
fn public() -> Check {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let recorded: Value = serde_json::from_str(&super::launch::fixture_text(
        "replies/author/synthetic/semantic-scholar/90d8b5261ee71845c924edbc2407040f440b73da2ab1fd8e51159839ba6ca612.json",
    )?).map_err(|e| e.to_string())?;
    recorded
        .pointer("/exchanges/0/body/value")
        .ok_or("missing recorded source reply")?;
    // A found author keeps this case on Semantic Scholar alone; an empty page now also asks PubMed.
    let body = json!({"offset":0,"total":1,"data":[{"authorId":"901","name":"Louis Williams"}]})
        .to_string();
    let stub = super::network_stub::Stub::for_source(
        "api.semanticscholar.org",
        "application/json",
        move |_| Ok(body.clone()),
    )?;
    stub.retain(assets.shared_root());
    let ca = assets.root().join("ca.pem");
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    for hosted in [true, false] {
        let globals = if hosted {
            vec!["--hosted".into()]
        } else {
            vec![]
        };
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(
                json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"keys":{"semantic_scholar_api_key":SERVER}}),
            ),
            None,
            None,
            None,
            &globals,
        )?;
        for (index, supplied) in [true, false].into_iter().enumerate() {
            let query = format!("synthetic request {hosted} {index}");
            let extra = if supplied {
                format!("x-biomcp-key-semantic-scholar-api-key: {CALLER}\r\n")
            } else {
                String::new()
            };
            call(
                &server,
                &["search", "author", &query],
                &format!("{extra}x-biomcp-use: academic\r\n"),
                false,
            )?;
        }
        server.stop()?;
    }
    let requests = stub.finish()?;
    if requests.len() != 4 {
        return Err("public source wire count".into());
    }
    for ((_, request), expected) in requests.iter().zip([CALLER, SERVER, SERVER, SERVER]) {
        let value = request.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("x-api-key"))
                .map(|(_, value)| value.trim())
        });
        if value != Some(expected) {
            return Err("public source selected wrong request credential".into());
        }
    }
    Ok(())
}

fn stdio() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"keys":{"oncokb_api_key":SERVER,"alphagenome_api_key":SERVER}}))
        .map_err(|e| e.to_string())?;
    let mut globals = env.globals(
        Some(&super::launch::fixtures().join("replies/gene/oncokb-synthetic/data")),
        false,
    );
    globals.push("--hosted".into());
    for (source, args) in [
        ("oncokb", vec!["get", "gene", "TP53", "oncokb"]),
        (
            "alphagenome",
            vec!["get", "variant", "rs113488022", "alphagenome"],
        ),
    ] {
        let frame = super::mcp::request(
            json!(1),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        );
        let replies = super::mcp::exchange_with_globals(&env, &[frame], &globals)?;
        let reply = replies.first().ok_or("missing stdio reply")?;
        if reply.pointer("/result/structuredContent/error/source") != Some(&json!(source))
            || reply.pointer("/result/structuredContent/error/message")
                != Some(&json!("add the source key in your account settings"))
            || reply.to_string().contains(SERVER)
        {
            return Err("hosted stdio used a server credential or omitted account guidance".into());
        }
    }
    Ok(())
}
