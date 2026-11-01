//! MCP transport authority is checked before provider requests or filesystem changes.
use super::{http::Server, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("article-saves/stdio-refuse-hidden-components", || {
            hidden().map_err(|e| e.to_string().into())
        }),
        transcript_trial("mcp-parity/stdio-command-refusal-reasons", || {
            command_reasons(false).map_err(|e| e.to_string().into())
        }),
        transcript_trial("mcp-http/http-command-refusal-reasons", || {
            command_reasons(true).map_err(|e| e.to_string().into())
        }),
        transcript_trial(
            "mcp-http/refuse-maintenance-and-fresh-before-effects",
            || http().map_err(|e| e.to_string().into()),
        ),
        transcript_trial(
            "article-saves/stdio-refuse-replacement-before-effects",
            || stdio().map_err(|e| e.to_string().into()),
        ),
    ]
}
fn stub() -> Result<Stub, String> {
    Stub::for_sources(
        &["www.wikipathways.org", "mygene.info", "www.ebi.ac.uk"],
        |_| "application/json",
        |_| Ok("{}".into()),
    )
}
fn refusal(value: &Value, code: &str) -> Check {
    assert_eq!(value.get("status"), Some(&json!("usage")), "{value}");
    assert_eq!(value.pointer("/error/code"), Some(&json!(code)), "{value}");
    Ok(())
}
fn http() -> Check {
    let source = stub()?;
    let env = Environment::new()?;
    let certificate = env.root().join("source.pem");
    fs::write(&certificate, &source.certificate)?;
    let server = Server::keys(
        json!({"proxy":format!("http://{}", source.address), "ca_bundle":certificate}),
        None,
    )?;
    let directory = server.environment().root().join("cache/biomcp");
    fs::create_dir_all(&directory)?;
    let old = directory.join("cache-1-4.sqlite");
    rusqlite::Connection::open(&old)?
        .execute_batch("CREATE TABLE meta(owner TEXT,version INTEGER); INSERT INTO meta VALUES ('biomcp-query',4); CREATE TABLE pool(id TEXT);")?;
    let before = fs::read(&old)?;
    let entries = fs::read_dir(&directory)?.count();
    for args in [
        vec!["sync", "wikipathways"],
        vec!["store", "clean"],
        vec!["get", "gene", "BRAF", "--fresh"],
    ] {
        let frame = super::mcp::request(
            json!(1),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        );
        let (status, reply) = server.request("POST", "/mcp", &frame.to_string())?;
        assert_eq!(
            status, 200,
            "HTTP tool calls must preserve the server admission policy"
        );
        assert_eq!(
            reply.pointer("/result/isError"),
            Some(&json!(true)),
            "{reply}"
        );
        refusal(
            reply
                .pointer("/result/structuredContent")
                .ok_or("missing tool result")?,
            if args.first().is_some_and(|arg| *arg == "get") {
                "usage.arguments"
            } else {
                "usage.mcp_unavailable"
            },
        )?;
        assert_eq!(
            fs::read(&old)?,
            before,
            "HTTP tool calls must preserve the server admission policy"
        );
        assert_eq!(
            fs::read_dir(&directory)?.count(),
            entries,
            "HTTP tool calls must preserve the server admission policy"
        );
    }
    assert!(
        source.finish()?.is_empty(),
        "refused HTTP actions contacted a provider"
    );
    Ok(())
}
fn stdio() -> Check {
    let source = stub()?;
    let env = Environment::new()?;
    let certificate = env.root().join("source.pem");
    fs::write(&certificate, &source.certificate)?;
    env.config(&json!({"proxy":format!("http://{}", source.address), "ca_bundle":certificate}))?;
    let path = env.root().join("owned.md");
    fs::write(&path, b"caller content")?;
    for words in [
        vec!["article", "text", "PPR1295425"],
        vec!["article", "supplement", "28944238", "MGG3-5-553-s002.xlsx"],
        vec!["article", "pdf", "28944238"],
        vec!["trial", "document", "NCT04303780", "SAP_001.pdf"],
    ] {
        let mut args: Vec<String> = words.into_iter().map(str::to_owned).collect();
        args.extend([
            "--save".into(),
            path.to_string_lossy().into(),
            "--replace".into(),
        ]);
        let value = super::mcp::call_with_globals(&env, &args, true, &[])?;
        refusal(&value, "usage.arguments")?;
        assert_eq!(
            fs::read(&path)?,
            b"caller content",
            "Stdio tool calls must preserve the server admission policy"
        );
    }
    assert!(
        source.finish()?.is_empty(),
        "refused stdio replacement contacted a provider"
    );
    Ok(())
}

fn command_reasons(http: bool) -> Check {
    let env = Environment::new()?;
    let server = http.then(|| Server::keys(json!({}), None)).transpose()?;
    for (words, command) in [
        (vec!["sync", "wikipathways"], "sync"),
        (vec!["store", "clean"], "store clean"),
        (vec!["key", "list"], "key"),
        (vec!["key", "set", "oncokb", "--replace"], "key"),
    ] {
        if !http && command != "key" {
            continue;
        }
        let args: Vec<String> = words
            .into_iter()
            .chain(["--offline"])
            .map(str::to_owned)
            .collect();
        let value = if let Some(server) = &server {
            let frame = super::mcp::request(
                json!(1),
                "tools/call",
                json!({"name":"biomcp","arguments":{"args":args}}),
            );
            let (status, reply) = server.request("POST", "/mcp", &frame.to_string())?;
            assert_eq!(
                status, 200,
                "Tool refusals must retain the command-specific reason"
            );
            assert_eq!(
                reply.pointer("/result/isError"),
                Some(&json!(true)),
                "Tool refusals must retain the command-specific reason"
            );
            let value = reply
                .pointer("/result/structuredContent")
                .ok_or("missing result")?;
            let text = reply
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
                .ok_or("missing text")?;
            assert_eq!(
                serde_json::from_str::<Value>(text)?,
                *value,
                "Tool refusals must retain the command-specific reason"
            );
            value.clone()
        } else {
            super::mcp::call_with_globals(&env, &args, true, &[])?
        };
        assert_eq!(value.get("status"), Some(&json!("usage")), "{value}");
        assert_eq!(
            value.get("error"),
            Some(&json!({
                "code":"usage.mcp_unavailable",
                "message":format!("{command} is unavailable over MCP; run it from the command line")
            })),
            "{value}"
        );
    }
    Ok(())
}

fn hidden() -> Check {
    let env = Environment::new()?;
    for relative in [
        ".zshenv",
        ".ssh/authorized_keys",
        "visible/.secret/file.txt",
    ] {
        let path = env.root().join(relative);
        fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
        hidden_destination(&env, &path)?;
    }
    #[cfg(unix)]
    {
        let hidden = env.root().join(".target");
        fs::create_dir(&hidden)?;
        let link = env.root().join("link");
        std::os::unix::fs::symlink(&hidden, &link)?;
        hidden_destination(&env, &link.join("out.txt"))?;
    }
    hidden_cwd(&env)
}
fn hidden_destination(env: &Environment, path: &std::path::Path) -> Check {
    for mut args in [
        vec!["article", "text", "PPR1295425"],
        vec!["article", "supplement", "28944238", "MGG3-5-553-s002.xlsx"],
        vec!["article", "pdf", "28944238"],
        vec!["trial", "document", "NCT04303780", "SAP_001.pdf"],
    ]
    .map(|args| args.into_iter().map(str::to_owned).collect::<Vec<_>>())
    {
        args.extend(["--save".into(), path.to_string_lossy().into_owned()]);
        let value = super::mcp::call_with_globals(env, &args, true, &[])?;
        refusal(&value, "usage.arguments")?;
        assert!(!path.exists(), "MCP created a hidden destination");
    }
    Ok(())
}
fn hidden_cwd(env: &Environment) -> Check {
    let cwd = env.root().join(".cwd");
    fs::create_dir(&cwd)?;
    let mut command = env.command(
        env!("CARGO_BIN_EXE_biomcp"),
        &["serve".into(), "--offline".into()],
    );
    let child = command
        .current_dir(&cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let request = super::mcp::request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":["article","text","PPR1295425","--save","out.txt"]}}),
    );
    let (status, stdout, stderr) = super::process::exchange(child, &[request])?;
    command.record(status, &stdout, "mcp")?;
    let replies = super::process::messages(status, &stdout, &stderr)?;
    let value = replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
        .ok_or("missing hidden-cwd response")?;
    refusal(value, "usage.arguments")?;
    assert!(
        !cwd.join("out.txt").exists(),
        "MCP created a file under hidden cwd"
    );
    Ok(())
}
