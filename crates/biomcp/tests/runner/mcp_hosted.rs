//! Feature mcp-hosted: transport calls cannot use the host's files, keys or maintenance commands.
use super::{http::Server, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    let mut trials: Vec<_> = [false, true]
        .into_iter()
        .flat_map(|http| {
            cases().into_iter().map(move |(name, words, schema)| {
                let transport = if http { "http" } else { "stdio" };
                let trial_name = if !http && name == "health" {
                    crate::runner::pool_contract::HOSTED_MODE.0.to_owned()
                } else {
                    format!("mcp-hosted/{transport}/{name}")
                };
                transcript_trial(trial_name, move || {
                    check(http, &words, schema).map_err(|e| e.to_string().into())
                })
            })
        })
        .collect();
    for http in [false, true] {
        let transport = if http { "http" } else { "stdio" };
        trials.push(transcript_trial(
            format!("mcp-hosted/{transport}/reader"),
            move || reader(http).map_err(|e| e.to_string().into()),
        ));
    }
    trials
}

fn cases() -> Vec<(&'static str, Vec<&'static str>, &'static str)> {
    vec![
        ("health", vec!["health"], "biomcp.health/6"),
        (
            "sync",
            vec!["sync", "wikipathways"],
            "biomcp.wikipathways-sync/4",
        ),
        (
            "store-clean",
            vec!["store", "clean"],
            "biomcp.store-clean/4",
        ),
        (
            "fresh",
            vec!["get", "gene", "BRAF", "--fresh"],
            "biomcp.gene-cards/13",
        ),
        ("key-list", vec!["key", "list"], "biomcp.error/4"),
        ("key-set", vec!["key", "set", "oncokb"], "biomcp.error/4"),
        (
            "key-remove",
            vec!["key", "remove", "oncokb"],
            "biomcp.error/4",
        ),
        (
            "article-text",
            vec!["article", "text", "28944238"],
            "biomcp.article-text/7",
        ),
        (
            "article-supplement",
            vec!["article", "supplement", "28944238", "MGG3-5-553-s002.xlsx"],
            "biomcp.article-supplement/7",
        ),
        (
            "article-pdf",
            vec!["article", "pdf", "28944238"],
            "biomcp.article-pdf/6",
        ),
        (
            "trial-document",
            vec!["trial", "document", "NCT04303780", "SAP_001.pdf"],
            "biomcp.trial-document/7",
        ),
    ]
}

fn check(http: bool, words: &[&str], schema: &str) -> Check {
    let env = Environment::new()?;
    let mut args: Vec<String> = words.iter().map(|s| (*s).into()).collect();
    let path = env.root().join("saved");
    if matches!(words.first(), Some(&"article" | &"trial")) {
        args.extend(["--save".into(), path.to_string_lossy().into()]);
    }
    args.push("--offline".into());
    for hosted in [true, false] {
        let globals = if hosted {
            vec!["--hosted".into()]
        } else {
            vec![]
        };
        let value = call(http, &env, &args, &globals)?;
        if hosted {
            assert_eq!(value.get("status"), Some(&json!("usage")), "{value}");
            assert_eq!(
                value.get("error"),
                Some(&json!({
                    "code":"usage.mcp_unavailable",
                    "message":"this command is unavailable in hosted mode"
                })),
                "{value}"
            );
            assert!(!path.exists(), "hosted calls must not create a saved file");
        } else if words.first() == Some(&"key")
            || (http && matches!(words.first(), Some(&"sync" | &"store" | &"get")))
        {
            let code = if words.first() == Some(&"get") {
                "usage.arguments"
            } else {
                "usage.mcp_unavailable"
            };
            assert_eq!(value.pointer("/error/code"), Some(&json!(code)), "{value}");
            assert_ne!(
                value.pointer("/error/message"),
                Some(&json!("this command is unavailable in hosted mode")),
                "{value}"
            );
        } else {
            assert_eq!(value.get("schema"), Some(&json!(schema)), "{value}");
            assert!(value.get("error").is_none(), "{value}");
        }
    }
    // These calls run offline, so ordinary saves cannot create an artifact either.
    assert!(
        !path.exists(),
        "offline controls must not create saved files"
    );
    Ok(())
}

fn call(http: bool, env: &Environment, args: &[String], globals: &[String]) -> Check<Value> {
    let frame = super::mcp::request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":args}}),
    );
    let reply = if http {
        let server = Server::configured_globals("127.0.0.1:0", None, None, None, None, globals)?;
        let (status, reply) = server.request("POST", "/mcp", &frame.to_string())?;
        assert_eq!(status, 200, "{reply}");
        reply
    } else {
        super::mcp::exchange_with_globals(env, &[frame], globals)?
            .into_iter()
            .next()
            .ok_or("missing reply")?
    };
    let value = reply
        .pointer("/result/structuredContent")
        .ok_or("missing tool result")?;
    let text = reply
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .ok_or("missing text")?;
    assert_eq!(
        serde_json::from_str::<Value>(text)?,
        *value,
        "MCP text must match its structured result"
    );
    assert_eq!(
        reply.pointer("/result/isError"),
        Some(&json!(
            value.get("error").is_some() || value.get("status") == Some(&json!("failed"))
        )),
        "{reply}"
    );
    super::schema::Registry::embedded()?.validate(value, args, false)?;
    Ok(value.clone())
}

fn reader(http: bool) -> Check {
    let env = Environment::new()?;
    for words in [
        vec!["get", "gene", "BRAF", "--offline"],
        vec!["article", "text", "28944238", "--find", "BRAF", "--offline"],
        vec![
            "article",
            "supplement",
            "28944238",
            "MGG3-5-553-s002.xlsx",
            "--find",
            "BRAF",
            "--offline",
        ],
    ] {
        let args: Vec<String> = words.into_iter().map(str::to_owned).collect();
        let value = call(http, &env, &args, &["--hosted".into()])?;
        assert!(
            value.get("error").is_none(),
            "hosted readers must remain available: {value}"
        );
    }
    Ok(())
}
