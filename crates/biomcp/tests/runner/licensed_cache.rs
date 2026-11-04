//! Feature mcp-hosted: licensed replies belong to the caller's effective key.
use super::{http::Server, launch::Environment, network_stub::Stub, request_keys::call};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
const FIRST: &str = "synthetic-cache-caller-a";
const SECOND: &str = "synthetic-cache-caller-b";
const STATE: &str = "/result/structuredContent/result/candidates/0/sections/oncokb/state";

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-hosted/cache/licensed-callers", || {
            licensed().map_err(Into::into)
        }),
        transcript_trial("mcp-hosted/cache/licensed-recordings", || {
            recordings().map_err(Into::into)
        }),
    ]
}

pub(super) fn scoped_gene_fixture(
    env: &Environment,
    key: &str,
) -> Result<std::path::PathBuf, String> {
    let directory = env.root().join("scoped-recordings");
    super::response_cases::copy_capture("gene/oncokb-synthetic/data", &directory, |_, _| {})
        .map_err(|e| e.to_string())?;
    let mut secrets = biomcp_fetch::Secrets::default();
    secrets.insert(
        biomcp_fetch::SecretName::OncokbApiKey,
        biomcp_fetch::Secret::new(key),
    );
    let fingerprint = secrets
        .fingerprint(biomcp_fetch::SecretName::OncokbApiKey)
        .ok_or("synthetic fingerprint")?;
    for entry in std::fs::read_dir(directory.join("oncokb")).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("recording identity")?;
        let bytes = stem
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| {
                let text = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
                u8::from_str_radix(text, 16).map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let identity = bytes.try_into().map_err(|_| "recording identity length")?;
        let scoped: String = fingerprint
            .namespace(identity)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        std::fs::rename(&path, path.with_file_name(format!("{scoped}.json")))
            .map_err(|e| e.to_string())?;
    }
    Ok(directory)
}

fn request(server: &Server, key: Option<&str>, offline: bool) -> Result<Value, String> {
    let args = super::oncokb::args();
    let mut args: Vec<_> = args.iter().map(String::as_str).collect();
    if offline {
        args.push("--offline");
    }
    let header = key
        .map(|key| format!("x-biomcp-key-oncokb-api-key: {key}\r\n"))
        .unwrap_or_default();
    let reply = call(
        server,
        &args,
        &format!("{header}x-biomcp-use: academic\r\n"),
        false,
    )?;
    if [FIRST, SECOND]
        .iter()
        .any(|key| reply.to_string().contains(key))
    {
        return Err("licensed credential escaped in reply".into());
    }
    Ok(reply)
}

fn licensed() -> Result<(), String> {
    for hosted in [true, false] {
        let assets = Environment::new().map_err(|e| e.to_string())?;
        let stub = Stub::for_source("www.oncokb.org", "application/json", |request| {
            let mut reply: Value =
                serde_json::from_str(&super::oncokb::reply(request)?).map_err(|e| e.to_string())?;
            let summary = json!(if request.contains(SECOND) {
                "Synthetic caller B result"
            } else {
                "Synthetic caller A result"
            });
            reply
                .as_object_mut()
                .ok_or("synthetic source reply object")?
                .insert("variantSummary".into(), summary);
            Ok(reply.to_string())
        })?;
        stub.retain(assets.shared_root());
        let ca = assets.root().join("ca.pem");
        std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
        let globals = if hosted {
            vec!["--hosted".into()]
        } else {
            vec![]
        };
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(
                json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
                "keys":{"oncokb_api_key":FIRST}}),
            ),
            None,
            None,
            None,
            &globals,
        )?;
        let first = request(&server, Some(FIRST), false)?;
        if first.pointer(STATE) != Some(&json!("data")) {
            return Err("caller A did not fetch the licensed reply".into());
        }
        if request(&server, Some(FIRST), true)?.pointer(STATE) != Some(&json!("data")) {
            return Err("caller A did not retain its own licensed cached reply".into());
        }
        let other = request(&server, Some(SECOND), true)?;
        if hosted && other.pointer(STATE) == Some(&json!("data")) {
            return Err("caller B read caller A's licensed cache while offline".into());
        }
        let second = request(&server, Some(SECOND), false)?;
        if second.pointer(STATE) != Some(&json!("data"))
            || !second.to_string().contains(if hosted {
                "Synthetic caller B result"
            } else {
                "Synthetic caller A result"
            })
        {
            return Err("licensed reply did not follow the effective caller key".into());
        }
        let keyless = request(&server, None, false)?;
        if hosted
            && keyless.pointer("/result/structuredContent/error/source") != Some(&json!("oncokb"))
        {
            return Err("keyless caller received licensed cached data".into());
        }
        server.stop()?;
        let requests = stub.finish()?;
        if requests.len() != if hosted { 2 } else { 1 } {
            return Err("licensed cache fetched the wrong number of replies".into());
        }
    }
    Ok(())
}

fn recordings() -> Result<(), String> {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let stub = Stub::for_source("www.oncokb.org", "application/json", super::oncokb::reply)?;
    stub.retain(assets.shared_root());
    let ca = assets.root().join("ca.pem");
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    let directory = assets.root().join("recordings");
    let legacy = assets.root().join("legacy-recordings");
    for hosted in [false, true] {
        let capture = if hosted { &directory } else { &legacy };
        let mut globals = vec!["--record".into(), capture.to_string_lossy().into()];
        if hosted {
            globals.push("--hosted".into());
        }
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(
                json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"keys":{"oncokb_api_key":FIRST}}),
            ),
            None,
            None,
            None,
            &globals,
        )?;
        if request(&server, Some(FIRST), false)?.pointer(STATE) != Some(&json!("data")) {
            return Err("synthetic licensed capture was not published".into());
        }
        server.stop()?;
    }
    for entry in std::fs::read_dir(legacy.join("oncokb")).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        std::fs::copy(
            entry.path(),
            directory.join("oncokb").join(entry.file_name()),
        )
        .map_err(|e| e.to_string())?;
    }
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        None,
        Some(directory.clone()),
        None,
        None,
        &["--hosted".into()],
    )?;
    for key in [Some(SECOND), Some(FIRST), None, Some(SECOND)] {
        let reply = request(&server, key, false)?;
        if (reply.pointer(STATE) == Some(&json!("data"))) != (key == Some(FIRST)) {
            return Err("recording or replay cache crossed licensed callers".into());
        }
    }
    server.stop()?;
    let mut ordinary = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"keys":{"oncokb_api_key":FIRST}})),
        Some(directory),
        None,
        None,
        &[],
    )?;
    if request(&ordinary, Some(SECOND), false)?.pointer(STATE) != Some(&json!("data")) {
        return Err("ordinary recording behavior changed".into());
    }
    ordinary.stop()?;
    if stub.finish()?.len() != 2 {
        return Err("record/replay opened an unexpected connection".into());
    }
    Ok(())
}
