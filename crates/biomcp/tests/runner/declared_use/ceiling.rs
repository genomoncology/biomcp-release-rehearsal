//! Feature mcp-hosted: hosted ceilings refuse higher declarations and optional keys preserve source rights.
use super::{ACADEMIC, KEY, call, disabled, source_ok};
use crate::runner::{http::Server, launch::fixtures};
use serde_json::json;

pub(super) fn check() -> Result<(), String> {
    for (ceiling, allowed, refused) in [
        ("restricted", "restricted", vec!["commercial", "academic"]),
        ("commercial", "restricted, commercial", vec!["academic"]),
        ("academic", "restricted, commercial, academic", vec![]),
    ] {
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            Some(json!({"hosted_max_use":ceiling})),
            None,
            None,
            None,
            &["--hosted".into()],
        )?;
        let frame = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"biomcp","arguments":{"args":["get","gene","TP53","oncokb","--offline"]}}});
        for mode in ["restricted", "commercial", "academic"] {
            let headers = format!(
                "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n\
                 MCP-Protocol-Version: 2025-11-25\r\nx-biomcp-use: {mode}\r\n\
                 x-biomcp-key-oncokb-api-key: {KEY}\r\n"
            );
            let (status, reply, _) = server.raw(
                "POST",
                "/mcp",
                &server.address.to_string(),
                &headers,
                &frame.to_string(),
            )?;
            if refused.contains(&mode) {
                if status != 400
                    || reply
                        != Some(
                            json!({"error":format!("Invalid x-biomcp-use header; allowed values: {allowed}")}),
                        )
                {
                    return Err(
                        "hosted ceiling failed to refuse a higher declaration with allowed modes"
                            .into(),
                    );
                }
            } else if status != 200 {
                return Err(format!(
                    "hosted ceiling {ceiling} refused allowed {mode}: {status} {reply:?}"
                ));
            }
        }
        // An absent declaration remains restricted even when the ceiling is higher.
        let reply = call(
            &server,
            &["search", "author", "Louis Williams", "--offline"],
            "",
            false,
        )?;
        if !disabled(&reply, "semantic-scholar") {
            return Err("missing declaration stopped selecting restricted".into());
        }
        server.stop()?;
    }
    Ok(())
}

pub(super) fn optional_keys() -> Result<(), String> {
    for (fixture, args, source, header) in [
        (
            "author/synthetic",
            vec!["search", "author", "Louis Williams"],
            "semantic-scholar",
            "semantic-scholar-api-key",
        ),
        (
            "drug/label-discovery",
            vec![
                "search",
                "drug",
                "NDA000001",
                "--source",
                "openfda-label",
                "--limit",
                "1",
            ],
            "openfda-label",
            "openfda-api-key",
        ),
    ] {
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            None,
            Some(fixtures().join("replies").join(fixture)),
            None,
            None,
            &["--hosted".into()],
        )?;
        let headers = format!("x-biomcp-use: commercial\r\nx-biomcp-key-{header}: {KEY}\r\n");
        let reply = call(&server, &args, &headers, false)?;
        if (source == "semantic-scholar" && !disabled(&reply, source))
            || (source == "openfda-label" && !source_ok(&reply, source))
        {
            return Err(format!(
                "optional rate key changed source admission: {reply}"
            ));
        }
        if reply.to_string().contains(KEY) {
            return Err("caller credential escaped in reply".into());
        }
        // Default academic ceiling still admits configured noncommercial sources.
        if source == "semantic-scholar"
            && !source_ok(&call(&server, &args, ACADEMIC, false)?, source)
        {
            return Err("default academic ceiling lost source access".into());
        }
        server.stop()?;
    }
    Ok(())
}
