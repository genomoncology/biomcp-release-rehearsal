//! Feature gene-bundle: server stale reads return before controlled publication.
use super::{
    bundle_wait::{native_body, wait_for_reader},
    configuration::invoke,
    launch::Environment,
    mcp::request,
    network_stub::Stub,
    publication::Peer,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, sync::mpsc, time::Duration};

pub(super) fn trials() -> Vec<Trial> {
    [false, true]
        .into_iter()
        .map(|missing| {
            let name = if missing {
                "missing-waits"
            } else {
                "stale-background-and-fresh-waits"
            };
            super::transcript::trial(format!("gene-bundle/{name}"), move || {
                read(missing).map_err(Into::into)
            })
        })
        .chain(super::bundle_pinned::trials())
        .collect()
}
fn read(missing: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = super::launch::fixtures().join("replies/gene/gencc");
    let body = native_body(&replay, "hgnc")?;
    let (admitted, admission) = mpsc::channel();
    let (publish, publication) = mpsc::channel();
    let mut seed = !missing;
    let stub = Stub::for_sources(
        &["storage.googleapis.com"],
        |_| "text/tab-separated-values",
        move |_| {
            if seed {
                seed = false;
                return Ok(body.clone());
            }
            admitted.send(()).map_err(|e| e.to_string())?;
            publication
                .recv_timeout(Duration::from_secs(30))
                .map_err(|e| e.to_string())?;
            Ok(body.replace("ornithine decarboxylase 1", "Synthetic refreshed gene"))
        },
    )?;
    stub.retain(env.shared_root());
    let ca = env.root().join("ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(
        &json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "disable_sources":["oncokb"]}),
    )
    .map_err(|e| e.to_string())?;
    if !missing {
        invoke(&env, vec!["get".into(), "gene".into(), "ODC1".into()], 0)?;
        expire(&env)?;
    }
    let observations = env.root().join("observations.jsonl");
    let mut server = Peer::with_globals(
        &env,
        &["--observe".into(), observations.to_string_lossy().into()],
    )?;
    server.send(&call("read", false))?;
    let result = if missing {
        admission
            .recv_timeout(Duration::from_secs(15))
            .map_err(|e| e.to_string())?;
        server.send(&request(json!("pending"), "ping", json!({})))?;
        let reply = server.next()?;
        if reply.get("id") != Some(&json!("pending")) {
            return Err(format!(
                "missing bundle returned before publication: {reply}"
            ));
        }
        publish.send(()).map_err(|e| e.to_string())?;
        checked(&server.next()?, "Synthetic refreshed gene", false)
    } else {
        stale_then_fresh(&mut server, &observations, admission, publish)
    };
    let stopped = server.finish();
    let fixture = stub.finish().map(|_| ());
    result.and(stopped).and(fixture)
}
fn call(id: &str, fresh: bool) -> Value {
    let mut args = vec!["get", "gene", "ODC1"];
    if fresh {
        args.push("--fresh");
    }
    request(
        json!(id),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":args}}),
    )
}
fn checked(reply: &Value, label: &str, stale: bool) -> Result<(), String> {
    let value = reply
        .pointer("/result/structuredContent")
        .ok_or_else(|| format!("missing content: {reply}"))?;
    let hgnc = value
        .get("sources")
        .and_then(Value::as_array)
        .and_then(|sources| {
            sources
                .iter()
                .find(|source| source.get("source") == Some(&json!("hgnc")))
        })
        .ok_or("missing HGNC outcome")?;
    // A timed-out inline refresh can also return stale rows before this held
    // fixture finishes. Its source failure must never satisfy immediate reuse.
    if hgnc.get("reason").is_some_and(|reason| !reason.is_null()) {
        return Err(format!("read waited for a failed inline refresh: {hgnc}"));
    }
    if !value.to_string().contains(label) || (hgnc.get("state") == Some(&json!("stale"))) != stale {
        return Err(format!("bundle rows or stale marker changed: {value}"));
    }
    Ok(())
}
fn stale_then_fresh(
    server: &mut Peer,
    observations: &std::path::Path,
    admission: mpsc::Receiver<()>,
    publish: mpsc::Sender<()>,
) -> Result<(), String> {
    // The fixture cannot finish until after the first stale reply is checked.
    checked(&server.next()?, "ornithine decarboxylase 1", true)?;
    admission
        .recv_timeout(Duration::from_secs(15))
        .map_err(|e| e.to_string())?;
    server.send(&call("fresh", true))?;
    std::thread::scope(|scope| {
        let (done, completion) = mpsc::channel();
        let reader = &mut *server;
        scope.spawn(move || {
            let _ = done.send(reader.next());
        });
        let waiting = wait_for_reader(observations, &completion);
        publish.send(()).map_err(|e| e.to_string())?;
        waiting?;
        checked(
            &completion
                .recv_timeout(Duration::from_secs(15))
                .map_err(|e| e.to_string())??,
            "Synthetic refreshed gene",
            false,
        )
    })?;
    server.send(&call("later", false))?;
    checked(&server.next()?, "Synthetic refreshed gene", false)
}
fn expire(env: &Environment) -> Result<(), String> {
    let folder = env.root().join("cache/biomcp");
    let path = fs::read_dir(folder)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "sqlite")
        })
        .ok_or("cache missing")?;
    rusqlite::Connection::open(path)
        .map_err(|e| e.to_string())?
        .execute("UPDATE bundle SET fresh_until=0", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
