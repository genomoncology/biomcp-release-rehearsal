//! Explicit saves preserve source bytes and leave ownership with the caller.
use super::article_save_reply::cut_reply;
use super::launch::{Environment, fixtures};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
mod cache_boundary;

type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn check() -> Check {
    text()?;
    hidden_cli()?;
    preprint()?;
    supplement()?;
    cache_boundary::check()?;
    pdf()?;
    opaque()?;
    arguments()?;
    failures()?;
    cut_body()?;
    http_refusal()
}
fn args(kind: &str, path: &Path) -> Vec<String> {
    let mut args = vec!["article".into(), kind.into(), "28944238".into()];
    if kind == "supplement" {
        args.push("MGG3-5-553-s002.xlsx".into());
    }
    args.extend(["--save".into(), path.to_string_lossy().into()]);
    args
}
fn command(
    env: &Environment,
    capture: &str,
    args: &[String],
    exit: i32,
) -> Result<Value, Box<dyn std::error::Error>> {
    command_at(env, &fixtures().join("replies").join(capture), args, exit)
}
fn command_at(
    env: &Environment,
    replay: &Path,
    args: &[String],
    exit: i32,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut all = env.globals(Some(replay), false);
    all.extend_from_slice(args);
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &all).output()?;
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "Save commands must keep diagnostics out of their JSON response"
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn receipt(value: &Value, path: &Path, bytes: &[u8], parts: Option<u32>) -> Check {
    let saved = value
        .pointer("/result/content/data/saved")
        .ok_or("missing receipt")?;
    let sha: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        saved.get("path"),
        Some(&json!(path.canonicalize()?.to_string_lossy())),
        "Save receipts must identify the exact artifact written"
    );
    assert_eq!(
        saved.get("bytes"),
        Some(&json!(bytes.len())),
        "Save receipts must identify the exact artifact written"
    );
    assert_eq!(
        saved.get("sha256"),
        Some(&json!(sha)),
        "Save receipts must identify the exact artifact written"
    );
    assert_eq!(
        saved.get("parts"),
        parts.map(|n| json!(n)).as_ref(),
        "Save receipts must identify the exact artifact written"
    );
    assert!(
        saved.get("url").and_then(Value::as_str).is_some_and(
            |s| s.starts_with("https://pmc-oa-opendata.s3.amazonaws.com/PMC5606870.1/")
        ),
        "Save receipts must identify the exact artifact written"
    );
    assert_eq!(
        value.pointer("/result/content/data/license"),
        Some(&json!("cc_by")),
        "Save receipts must identify the exact artifact written"
    );
    super::schema::Registry::embedded()?.validate(
        value,
        &args(
            if parts.is_some() {
                "text"
            } else {
                "supplement"
            },
            path,
        ),
        false,
    )?;
    Ok(())
}
fn text() -> Check {
    let env = Environment::new()?;
    let folder = env.root().join("papers");
    fs::create_dir(&folder)?;
    let capture = "article/fulltext/live-introduction";
    let value = command(&env, capture, &args("text", &folder), 0)?;
    let path = folder.join("28944238.md");
    let body = fs::read_to_string(&path)?;
    receipt(&value, &path, body.as_bytes(), Some(25))?;
    assert!(
        body.starts_with(
            "# Targeted sequencing of 36 known or putative colorectal cancer susceptibility genes\n"
        ),
        "Full text saves must preserve complete content and existing files"
    );
    for text in [
        "PMC5606870",
        "pmid: 28944238",
        "doi: 10.1002/mgg3.317",
        "License: cc_by",
        "Source: https://",
        "Introduction",
        "Discussion",
        "**Table S4.** Alternative splicing variants.",
    ] {
        assert!(body.contains(text), "lost complete text/header: {text}");
    }
    assert!(
        body.find("Introduction") < body.find("Discussion"),
        "Full text saves must preserve complete content and existing files"
    );
    let original = body.as_bytes();
    fs::write(&path, b"caller content")?;
    command(&env, capture, &args("text", &folder), 2)?;
    assert_eq!(
        fs::read(&path)?,
        b"caller content",
        "Full text saves must preserve complete content and existing files"
    );
    let mut replace = args("text", &folder);
    replace.push("--replace".into());
    command(&env, capture, &replace, 0)?;
    assert_eq!(
        fs::read(&path)?,
        original,
        "Full text saves must preserve complete content and existing files"
    );
    let stdio = folder.join("stdio.md");
    let value = super::mcp::call_with_globals(
        &env,
        &args("text", &stdio),
        false,
        &env.globals(Some(&fixtures().join("replies").join(capture)), false),
    )?;
    receipt(&value, &stdio, original, Some(25))?;
    assert_eq!(
        fs::read(stdio)?,
        original,
        "Full text saves must preserve complete content and existing files"
    );
    assert_eq!(
        fs::read_dir(folder)?.count(),
        2,
        "Full text saves must preserve complete content and existing files"
    );
    Ok(())
}
fn hidden_cli() -> Check {
    let env = Environment::new()?;
    let destination = env.root().join(".zshenv");
    command(
        &env,
        "article/fulltext/live-introduction",
        &args("text", &destination),
        0,
    )?;
    assert!(destination.is_file(), "CLI saves allow hidden destinations");
    Ok(())
}
fn supplement() -> Check {
    let env = Environment::new()?;
    let folder = env.root().join("supplements");
    fs::create_dir(&folder)?;
    let capture = "article/supplements/live";
    let original =
        fs::read(fixtures().join("replies").join(capture).join(
            "pmc-oa/51c9ef213345bdbc7ed493ea19928bd900d5791413a6c5a65b60f2c50eac690f-0.bin",
        ))?;
    let value = command(&env, capture, &args("supplement", &folder), 0)?;
    let path = folder.join("MGG3-5-553-s002.xlsx");
    assert_eq!(
        fs::read(&path)?,
        original,
        "Supplement saves must preserve existing caller files"
    );
    receipt(&value, &path, &original, None)?;
    fs::write(&path, b"caller content")?;
    let mut replace = args("supplement", &folder);
    replace.push("--replace".into());
    let replaced = command(&env, capture, &replace, 0)?;
    assert_eq!(
        fs::read(&path)?,
        original,
        "Replacement must preserve source bytes"
    );
    receipt(&replaced, &path, &original, None)?;
    let stdio = folder.join("stdio.xlsx");
    let value = super::mcp::call_with_globals(
        &env,
        &args("supplement", &stdio),
        false,
        &env.globals(Some(&fixtures().join("replies").join(capture)), false),
    )?;
    receipt(&value, &stdio, &original, None)?;
    assert_eq!(
        fs::read(stdio)?,
        original,
        "Supplement saves must preserve existing caller files"
    );
    Ok(())
}
fn failures() -> Check {
    {
        let (kind, capture) = ("text", "article/fulltext/synthetic-cut");
        let env = Environment::new()?;
        let folder = env.root().join("empty");
        fs::create_dir(&folder)?;
        let target = folder.join("never");
        let mut initial = args(kind, &target);
        *initial.get_mut(2).ok_or("identity")? = "PMC5606870".into();
        command(&env, capture, &initial, 4)?;
        assert_eq!(
            fs::read_dir(&folder)?.count(),
            0,
            "Refused saves must leave no partial output or new files"
        );
        fs::write(&target, b"caller content")?;
        let mut replace = initial;
        replace.push("--replace".into());
        command(&env, capture, &replace, 4)?;
        assert_eq!(
            fs::read(&target)?,
            b"caller content",
            "Refused saves must leave no partial output or new files"
        );
        assert_eq!(
            fs::read_dir(folder)?.count(),
            1,
            "Refused saves must leave no partial output or new files"
        );
    }
    Ok(())
}
fn http_refusal() -> Check {
    for (kind, capture) in [
        ("text", "article/fulltext/live-introduction"),
        ("supplement", "article/supplements/live"),
    ] {
        let env = Environment::new()?;
        let path = env.root().join("server-must-not-write");
        let replay = env.root().join("metadata-only");
        super::response_cases::copy_capture(capture, &replay, |_, _| {})?;
        for source in fs::read_dir(&replay)? {
            for file in fs::read_dir(source?.path())? {
                let path = file?.path();
                if path.extension().is_some_and(|v| v == "bin") {
                    fs::remove_file(path)?;
                }
            }
        }
        let mut server = super::http::Server::start_replay(replay)?;
        let call = super::mcp::request(
            json!(7),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args(kind, &path)}}),
        );
        let (status, reply) = server.request("POST", "/mcp", &call.to_string())?;
        assert_eq!(
            status, 200,
            "HTTP requests must refuse filesystem saves and retain the source link"
        );
        assert_eq!(
            reply.pointer("/result/isError"),
            Some(&json!(true)),
            "HTTP requests must refuse filesystem saves and retain the source link"
        );
        assert_eq!(
            reply.pointer("/result/structuredContent/status"),
            Some(&json!("usage")),
            "HTTP requests must refuse filesystem saves and retain the source link"
        );
        assert!(
            reply
                .pointer("/result/structuredContent/error/url")
                .and_then(Value::as_str)
                .is_some_and(|s| s.starts_with("https://pmc-oa-opendata.s3.amazonaws.com/")),
            "HTTP requests must refuse filesystem saves and retain the source link"
        );
        assert!(
            !path.exists(),
            "HTTP requests must refuse filesystem saves and retain the source link"
        );
        server.stop()?;
    }
    Ok(())
}

fn pdf() -> Check {
    let env = Environment::new()?;
    let replay = fixtures().join("replies/article/saves/pdf");
    let mut original = None;
    for entry in fs::read_dir(replay.join("pmc-oa"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|v| v == "bin") {
            original = Some(fs::read(path)?);
        }
    }
    let original = original.ok_or("recorded source PDF absent")?;
    assert!(
        original.starts_with(b"%PDF-"),
        "PDF saves must publish only the checked complete document"
    );
    let folder = env.root().join("pdf");
    fs::create_dir(&folder)?;
    let mut cli = args("supplement", &folder);
    *cli.get_mut(3).ok_or("file argument")? = "MGG3-5-553-s001.pdf".into();
    let value = command_at(&env, &replay, &cli, 0)?;
    let path = folder.join("MGG3-5-553-s001.pdf");
    receipt(&value, &path, &original, None)?;
    assert_eq!(
        fs::read(path)?,
        original,
        "PDF saves must publish only the checked complete document"
    );
    let stdio = folder.join("stdio.pdf");
    let mut cli = args("supplement", &stdio);
    *cli.get_mut(3).ok_or("file argument")? = "MGG3-5-553-s001.pdf".into();
    let value =
        super::mcp::call_with_globals(&env, &cli, false, &env.globals(Some(&replay), false))?;
    receipt(&value, &stdio, &original, None)?;
    assert_eq!(
        fs::read(stdio)?,
        original,
        "PDF saves must publish only the checked complete document"
    );
    Ok(())
}
fn opaque() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("opaque");
    // Preserve request identity and exact declared filename; edit only source MIME/body.
    super::response_cases::copy_capture("article/supplements/live", &replay, |_, record| {
        if let Some(Value::String(xml)) = record.pointer_mut("/exchanges/0/body/value") {
            *xml = xml.replace(
                "xlink:href=\"MGG3-5-553-s002.xlsx\"",
                "mimetype=\"application\" mime-subtype=\"vnd.openxmlformats-officedocument.wordprocessingml.document\" xlink:href=\"MGG3-5-553-s002.xlsx\"",
            );
        }
    })?;
    let bytes = b"PK\x03\x04opaque Word document, no extraction";
    fs::write(
        replay
            .join("pmc-oa/51c9ef213345bdbc7ed493ea19928bd900d5791413a6c5a65b60f2c50eac690f-0.bin"),
        bytes,
    )?;
    let reader = vec![
        "article".into(),
        "supplement".into(),
        "28944238".into(),
        "MGG3-5-553-s002.xlsx".into(),
    ];
    let unavailable = command_at(&env, &replay, &reader, 0)?;
    assert_eq!(
        unavailable.pointer("/result/content/data/reason"),
        Some(&json!("unsupported")),
        "Opaque supplement saves must preserve native bytes and media type"
    );
    let path = env.root().join("opaque.docx");
    let value = command_at(&env, &replay, &args("supplement", &path), 0)?;
    receipt(&value, &path, bytes, None)?;
    assert_eq!(
        fs::read(path)?,
        bytes,
        "Opaque supplement saves must preserve native bytes and media type"
    );
    Ok(())
}
fn arguments() -> Check {
    let env = Environment::new()?;
    for kind in ["text", "supplement"] {
        for extra in [
            vec!["--find", "literal"],
            vec!["--variant", "E17K"],
            vec!["--offset", "1"],
        ] {
            let path = env.root().join("never");
            let mut cli = args(kind, &path);
            cli.extend(extra.iter().map(|s| (*s).to_owned()));
            command(&env, "article/supplements/live", &cli, 2)?;
            assert!(
                !path.exists(),
                "Save paths must not be accepted as ordinary output formats"
            );
        }
    }
    Ok(())
}

fn cut_body() -> Check {
    let stub = super::network_stub::Stub::with_length(
        "pmc-oa-opendata.s3.amazonaws.com",
        |request| {
            if request.contains("list-type=") {
                "application/xml"
            } else {
                "binary/octet-stream"
            }
        },
        cut_reply,
        |request, bytes| {
            if request.contains(".xlsx?") {
                bytes + 100
            } else {
                bytes
            }
        },
    )?;
    let env = Environment::new()?;
    let ca = env.root().join("source.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate)?;
    env.config(&json!({"ca_bundle":ca, "proxy":format!("http://{}",stub.address)}))?;
    let folder = env.root().join("cut");
    fs::create_dir(&folder)?;
    let path = folder.join("never.xlsx");
    let mut failures = Vec::new();
    for replace in [false, true] {
        if replace {
            fs::write(&path, b"caller content")?;
        }
        let mut cli = args("supplement", &path);
        *cli.get_mut(2).ok_or("identity")? = "PMC5606870".into();
        if replace {
            cli.push("--replace".into());
        }
        let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &cli).output()?;
        assert_eq!(
            output.status.code(),
            Some(4),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        failures.push(String::from_utf8_lossy(&output.stdout).into_owned());
        if replace {
            assert_eq!(
                fs::read(&path)?,
                b"caller content",
                "Cut content must leave no partial save behind"
            );
            assert_eq!(
                fs::read_dir(&folder)?.count(),
                1,
                "Cut content must leave no partial save behind"
            );
        } else {
            assert_eq!(
                fs::read_dir(&folder)?.count(),
                0,
                "Cut content must leave no partial save behind"
            );
        }
    }
    let requests = stub.finish()?;
    assert!(
        requests
            .iter()
            .any(|(_, request)| request.contains(".xlsx?")),
        "requests={requests:?}; replies={failures:?}"
    );
    Ok(())
}

fn preprint() -> Check {
    let env = Environment::new()?;
    let folder = env.root().join("preprints");
    fs::create_dir(&folder)?;
    let mut cli = args("text", &folder);
    *cli.get_mut(2).ok_or("identity")? = "PPR1295425".into();
    let value = command(&env, "article/preprint/live", &cli, 0)?;
    let text = fs::read_to_string(folder.join("PPR1295425.md"))?;
    assert!(
        text.contains("ID: PPR1295425"),
        "Preprint saves must keep native identity and licensing metadata"
    );
    assert!(
        text.contains("License: cc_by"),
        "Preprint saves must keep native identity and licensing metadata"
    );
    assert!(
        text.contains("Introduction"),
        "Preprint saves must keep native identity and licensing metadata"
    );
    assert_eq!(
        value.pointer("/result/content/data/saved/url"),
        Some(&json!(
            "https://www.ebi.ac.uk/europepmc/webservices/rest/PPR1295425/fullTextXML"
        )),
        "Preprint saves must keep native identity and licensing metadata"
    );
    assert!(
        value
            .pointer("/result/content/data/saved/parts")
            .and_then(Value::as_u64)
            .is_some_and(|n| n > 1),
        "Preprint saves must keep native identity and licensing metadata"
    );
    Ok(())
}
