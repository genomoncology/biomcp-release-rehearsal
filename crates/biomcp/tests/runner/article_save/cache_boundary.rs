use super::super::article_save_reply::cut_reply;
use super::{Check, Environment, args};
use serde_json::{Value, json};
use std::fs;

// Explicit saves must neither supply nor consume the row reader's reply cache.
pub(super) fn check() -> Check {
    let stub = super::super::network_stub::Stub::with_length(
        "pmc-oa-opendata.s3.amazonaws.com",
        |request| {
            if request.contains("list-type=") {
                "application/xml"
            } else {
                "binary/octet-stream"
            }
        },
        csv_reply,
        |_, bytes| bytes,
    )?;
    for reader_first in [false, true] {
        let env = Environment::new()?;
        let ca = env.root().join("source.pem");
        stub.retain(env.shared_root());
        fs::write(&ca, &stub.certificate)?;
        env.config(&json!({"ca_bundle":ca, "proxy":format!("http://{}",stub.address)}))?;
        let path = env.root().join("original.csv");
        let mut save = args("supplement", &path);
        *save.get_mut(2).ok_or("identity")? = "PMC5606870".into();
        let reader = save.get(..4).ok_or("reader arguments")?.to_vec();
        let initial = if reader_first { &reader } else { &save };
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), initial)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        if !reader_first {
            assert_eq!(
                fs::read(&path)?,
                b"gene,value\nAKT1,1\n",
                "Save operations must respect cache access and caller ownership"
            );
            fs::remove_file(&path)?;
        }
        let mut offline = vec!["--offline".into()];
        offline.extend(if reader_first { save } else { reader });
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &offline)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(4),
            "raw save crossed the reply cache boundary (reader_first={reader_first}): {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_offline_failure(&output.stdout)?;
        assert!(
            !path.exists(),
            "Save operations must respect cache access and caller ownership"
        );
    }
    assert!(
        stub.finish()?
            .iter()
            .any(|(_, request)| request.contains(".xlsx?")),
        "Save operations must respect cache access and caller ownership"
    );
    Ok(())
}
fn assert_offline_failure(stdout: &[u8]) -> Check {
    let value: Value = serde_json::from_slice(stdout)?;
    assert!(
        value.get("result").is_none(),
        "failed save exposed its body"
    );
    assert_eq!(
        value.pointer("/status"),
        Some(&json!("failed")),
        "Failed saves must retain the overall failure status"
    );
    assert_eq!(
        value.pointer("/sources/0/source"),
        Some(&json!("pmc-oa")),
        "Failed saves must name the source that refused offline access"
    );
    assert_eq!(
        value.pointer("/sources/0/state"),
        Some(&json!("failed")),
        "Failed saves must retain the failed source state"
    );
    assert_eq!(
        value.pointer("/sources/0/reason"),
        Some(&json!("offline")),
        "Save operations must respect cache access and caller ownership"
    );
    Ok(())
}

fn csv_reply(request: &str) -> Result<String, String> {
    if request.contains(".xlsx?") {
        return Ok("gene,value\nAKT1,1\n".into());
    }
    Ok(cut_reply(request)?.replace(
        "xlink:href=\"MGG3-5-553-s002.xlsx\"",
        "mimetype=\"text\" mime-subtype=\"csv\" xlink:href=\"MGG3-5-553-s002.xlsx\"",
    ))
}
