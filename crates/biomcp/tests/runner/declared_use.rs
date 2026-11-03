//! Feature mcp-hosted: declared use gates sources and retained replies before source I/O.
use super::{http::Server, launch::Environment, network_stub::Stub, request_keys::call};
use crate::runner::transcript::trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
mod components;
mod continuation;
mod legacy;
mod records;
const ACADEMIC: &str = "x-biomcp-use: academic\r\n";
const KEY: &str = "7c28f96a-3f48-4d6b-8d30-512c5d41e979";
const CLINPGX: &[&str] = &[
    "get",
    "pgx",
    "--gene",
    "DPYD",
    "annotations",
    "--source",
    "clinpgx",
];
const ONCOKB: &[&str] = &["get", "gene", "TP53", "oncokb"];

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("mcp-hosted/use/legacy-group-membership", || {
            legacy::check().map_err(Into::into)
        }),
        trial("mcp-hosted/use/continuation-rights", || {
            continuation::check().map_err(Into::into)
        }),
        trial("mcp-hosted/use/free-sources", || {
            records::free().map_err(Into::into)
        }),
        trial("mcp-hosted/use/record-rights", || {
            records::rights().map_err(Into::into)
        }),
        trial("mcp-hosted/use/components", || {
            components::check().map_err(Into::into)
        }),
        trial("mcp-hosted/use/source-admission", || {
            admission().map_err(Into::into)
        }),
        trial("mcp-hosted/use/academic-stored-reply", || {
            retained().map_err(Into::into)
        }),
        trial("mcp-hosted/use/header-boundary", || {
            headers().map_err(Into::into)
        }),
    ]
}
fn disabled(reply: &Value, source: &str) -> bool {
    reply
        .pointer("/result/structuredContent/sources")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["source"] == source && row["reason"] == "disabled")
        })
}
fn source_ok(reply: &Value, source: &str) -> bool {
    reply
        .pointer("/result/structuredContent/sources")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["source"] == source && row["state"] == "ok")
        })
}
fn admission() -> Result<(), String> {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let scoped = super::licensed_cache::scoped_gene_fixture(&assets, KEY)?;
    repeat_recordings(&scoped)?;
    let key = format!("x-biomcp-key-oncokb-api-key: {KEY}\r\n");
    for excluded in [false, true] {
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(json!({"disable_sources":if excluded {vec!["oncokb", "clinpgx"]} else {vec![]}})),
            Some(scoped.clone()),
            None,
            None,
            &["--hosted".into()],
        )?;
        let ddinter = call(
            &server,
            &[
                "get",
                "drug",
                "DDInter1",
                "interactions",
                "--source",
                "ddinter",
            ],
            "",
            false,
        )?;
        if !disabled(&ddinter, "ddinter") {
            return Err(format!("default use admitted DDInter: {ddinter}"));
        }
        for use_header in ["", "x-biomcp-use: commercial\r\n", ACADEMIC] {
            let args = [
                "get",
                "pgx",
                "--gene",
                "DPYD",
                "annotations",
                "--source",
                "clinpgx",
            ];
            let cpic = call(&server, &args, use_header, false)?;
            if disabled(&cpic, "clinpgx") != (excluded || use_header != ACADEMIC) {
                return Err(format!("wrong noncommercial source admission: {cpic}"));
            }
            let reply = call(&server, ONCOKB, use_header, false)?;
            if excluded || use_header != ACADEMIC {
                if !disabled(&reply, "oncokb") {
                    return Err(format!("restricted source admitted: {reply}"));
                }
            } else if reply.pointer("/result/structuredContent/error/source")
                != Some(&json!("oncokb"))
            {
                return Err("academic request lost existing required-key admission".into());
            }
            let keyed = call(&server, ONCOKB, &format!("{use_header}{key}"), false)?;
            if excluded {
                if !disabled(&keyed, "oncokb") {
                    return Err("caller key overrode server exclusion".into());
                }
            } else if use_header.is_empty() {
                if !disabled(&keyed, "oncokb") {
                    return Err("default use accepted a licensed source key".into());
                }
            } else if keyed
                .pointer("/result/structuredContent/result/items/0/sections/oncokb/state")
                != Some(&json!("data"))
            {
                return Err(format!(
                    "caller key failed to license restricted source: {keyed}"
                ));
            }
        }
        server.stop()?;
    }
    Ok(())
}
fn retained() -> Result<(), String> {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let stub = Stub::for_source("api.clinpgx.org", "application/json", |_| {
        Ok(r#"{"status":"success","data":[]}"#.into())
    })?;
    stub.retain(assets.shared_root());
    let ca = assets.root().join("ca.pem");
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca})),
        None,
        None,
        None,
        &["--hosted".into()],
    )?;
    let academic = call(&server, CLINPGX, ACADEMIC, false)?;
    if !source_ok(&academic, "clinpgx") {
        return Err(format!("academic noncommercial source failed: {academic}"));
    }
    let offline: Vec<_> = CLINPGX.iter().copied().chain(["--offline"]).collect();
    let stored = call(&server, &offline, ACADEMIC, false)?;
    if !source_ok(&stored, "clinpgx") {
        return Err(format!("academic reply was not retained: {stored}"));
    }
    for header in ["", "x-biomcp-use: commercial\r\n"] {
        let commercial = call(&server, &offline, header, false)?;
        if !disabled(&commercial, "clinpgx")
            || commercial.to_string().contains("\"symbol\":\"DPYD\"")
        {
            return Err(format!(
                "academic stored reply escaped to commercial caller: {commercial}"
            ));
        }
    }
    server.stop()?;
    if stub.finish()?.is_empty() {
        return Err("commercial source reached upstream".into());
    }
    Ok(())
}
fn headers() -> Result<(), String> {
    for hosted in [false, true] {
        let globals = if hosted {
            vec!["--hosted".into()]
        } else {
            vec![]
        };
        let mut server =
            Server::configured_globals("127.0.0.1:0", None, None, None, None, &globals)?;
        let frame = json!({"jsonrpc":"2.0","id":1,"method":"tools/call", "params":{
            "name":"biomcp", "arguments":{
                "args":["get","pgx","--gene","DPYD","--source","clinpgx","--offline"]
            }
        }});
        for extra in [
            "x-biomcp-use: invalid\r\n",
            "x-biomcp-use: \r\n",
            "x-biomcp-use: Academic\r\n",
            "x-biomcp-use: academic\r\nx-biomcp-use: commercial\r\n",
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
            if status != if hosted { 400 } else { 200 } {
                return Err("declared-use header escaped hosted boundary".into());
            }
            if hosted
                && !reply.is_some_and(|reply| {
                    reply.get("error") == Some(&json!("Invalid x-biomcp-use header; allowed values: restricted, commercial, academic"))
                })
            {
                return Err("invalid declaration omitted allowed values".into());
            }
        }
        server.stop()?;
    }
    Ok(())
}

fn repeat_recordings(root: &std::path::Path) -> Result<(), String> {
    for source in ["mygene", "oncokb"] {
        for entry in std::fs::read_dir(root.join(source)).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let mut record: Value =
                serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let exchanges = record
                .get_mut("exchanges")
                .and_then(Value::as_array_mut)
                .ok_or("fixture exchanges")?;
            let original = exchanges.clone();
            for _ in 0..12 {
                exchanges.extend(original.clone());
            }
            std::fs::write(
                path,
                serde_json::to_vec(&record).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
