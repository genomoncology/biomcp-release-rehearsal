//! Feature mcp-hosted: free sources and per-record rights work in the default use.
use super::{ACADEMIC, call, disabled};
use crate::runner::{
    http::Server,
    launch::{self, Environment},
};
use serde_json::{Value, json};

pub(super) fn free() -> Result<(), String> {
    let cases: &[(&str, &[&str], &str)] = &[
        (
            "pgx/cpic-synthetic",
            &["get", "pgx", "--gene", "DPYD", "--source", "cpic"],
            "cpic",
        ),
        (
            "trial/native-exact",
            &[
                "search",
                "trial",
                "--condition",
                "Snyder-Robinson syndrome",
                "--exact-condition",
                "--limit",
                "1",
            ],
            "ctgov",
        ),
        (
            "article/fulltext/synthetic",
            &["get", "article", "28944238"],
            "pubmed",
        ),
        (
            "variant/assertions/domains",
            &["get", "variant", "clinvar:13961", "clinvar"],
            "clinvar",
        ),
    ];
    for (fixture, args, source) in cases {
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            None,
            Some(launch::fixtures().join("replies").join(fixture)),
            None,
            None,
            &["--hosted".into()],
        )?;
        let reply = call(&server, args, "", false)?;
        let rows = reply
            .pointer("/result/structuredContent/sources")
            .and_then(Value::as_array)
            .ok_or("missing sources")?;
        if !rows
            .iter()
            .any(|row| row["source"] == *source && row["state"] == "ok")
        {
            return Err(format!("free source failed under restricted use: {reply}"));
        }
        server.stop()?;
    }
    Ok(())
}

pub(super) fn rights() -> Result<(), String> {
    for license in ["CC BY", "CC BY NC"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let replay = env.root().join("replies");
        biomcp_test_support::capture::copy_capture(
            "article/fulltext/synthetic",
            &replay,
            |source, record| {
                if source == "pmc-oa" {
                    for exchange in record["exchanges"].as_array_mut().into_iter().flatten() {
                        if let Some(body) = exchange.pointer_mut("/body/value")
                            && let Some(text) = body.as_str()
                        {
                            *body = json!(text.replace(
                                "\"license_code\": \"CC BY\"",
                                &format!("\"license_code\": \"{license}\""),
                            ));
                        }
                    }
                }
            },
        )
        .map_err(|e| e.to_string())?;
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            None,
            Some(replay),
            None,
            None,
            &["--hosted".into()],
        )?;
        let args = ["article", "text", "PMC5606870", "--part", "1"];
        let academic = call(&server, &args, ACADEMIC, false)?;
        if academic.pointer("/result/structuredContent/status") != Some(&json!("complete")) {
            return Err(format!("academic full text failed: {academic}"));
        }
        let witness = academic
            .pointer("/result/structuredContent/result/content/data/markdown")
            .and_then(Value::as_str)
            .and_then(|text| text.lines().map(str::trim).find(|line| line.len() > 40))
            .map(|line| json!(line).to_string().trim_matches('"').to_owned())
            .ok_or_else(|| format!("academic reply has no body text: {academic}"))?;
        for header in [
            "",
            "x-biomcp-use: restricted\r\n",
            "x-biomcp-use: commercial\r\n",
        ] {
            let reply = call(&server, &args, header, false)?;
            if license == "CC BY NC" {
                // The withheld reply keeps a skipped section and a reading link, never the text.
                let content = reply.pointer("/result/structuredContent/result/content");
                if content.and_then(|c| c.get("state")) != Some(&json!("skipped"))
                    || content.and_then(|c| c.get("data")).is_some()
                    || reply.to_string().contains(&witness)
                {
                    return Err(format!("noncommercial full text leaked content: {reply}"));
                }
            }
            if disabled(&reply, "pmc-oa") != (license == "CC BY NC") {
                return Err(format!("record rights did not gate cached text: {reply}"));
            }
            if license == "CC BY"
                && reply.pointer("/result/structuredContent/status") != Some(&json!("complete"))
            {
                return Err(format!("CC BY text was withheld: {reply}"));
            }
        }
        server.stop()?;
    }
    Ok(())
}
