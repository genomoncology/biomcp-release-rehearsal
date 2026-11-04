//! Confined native request identities with synthetic response-only boundary variants.
use biomcp_adapters::pmc_oa;
use biomcp_fetch::{Fetch, FetchConfig, Mode, RequestError};
use biomcp_types::Reason;
use serde_json::Value;
use std::{fs, path::Path, time::Duration};
type Error = Box<dyn std::error::Error>;
#[expect(
    clippy::disallowed_methods,
    reason = "Public adapter proof edits response exchanges in an isolated replay directory"
)]
fn fixture(change: impl FnOnce(&mut Value), xml: &str) -> Result<tempfile::TempDir, Error> {
    let directory = tempfile::tempdir()?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replies/article/fulltext/live-introduction/pmc-oa");
    let target = directory.path().join("pmc-oa");
    fs::create_dir(&target)?;
    let mut change = Some(change);
    for file in fs::read_dir(source)? {
        let file = file?;
        let mut record: Value = serde_json::from_slice(&fs::read(file.path())?)?;
        let url = record
            .get("url")
            .and_then(Value::as_str)
            .ok_or("missing native URL")?
            .to_owned();
        let body = record
            .pointer_mut("/exchanges/0/body/value")
            .ok_or("missing exchange")?;
        if url.ends_with(".json") {
            change.take().ok_or("duplicate metadata recording")?(body);
        } else if url.contains(".xml?") {
            *body = Value::String(xml.to_owned());
        }
        fs::write(target.join(file.file_name()), serde_json::to_vec(&record)?)?;
    }
    Ok(directory)
}
fn read(
    change: impl FnOnce(&mut Value),
    xml: &str,
) -> Result<Result<Option<pmc_oa::Document>, RequestError>, Error> {
    let directory = fixture(change, xml)?;
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().into(),
        },
        ..FetchConfig::default()
    })?;
    let leg = fetch.leg(&pmc_oa::SPEC, fetch.deadline(Duration::from_secs(20)));
    Ok(
        pmc_oa::read(&leg, "PMC5606870".parse()?, Some("28944238".parse()?))
            .map(|mapped| mapped.value),
    )
}
fn article(front: &str, body: &str) -> String {
    format!(
        "<article><front><article-meta>{front}</article-meta></front><body>{body}</body></article>"
    )
}
fn bad(result: Result<Option<pmc_oa::Document>, RequestError>) -> Result<(), Error> {
    match result {
        Err(RequestError::Source(error)) if error.reason == Reason::BadReply => Ok(()),
        _ => Err("invalid source response did not produce bad_reply".into()),
    }
}
#[test]
fn table_spans_are_bounded_before_expansion() -> Result<(), Error> {
    for span in ["0", "257", "99999999999999999999999999999999", "no"] {
        let body = format!(
            "<table-wrap id=\"T1\"><table><tr><td colspan=\"{span}\">invented</td></tr></table></table-wrap>"
        );
        bad(read(|_| {}, &article("", &body))?)?;
    }
    bad(read(
        |_| {},
        &article("", "<table-wrap id=\"T1\"/><table-wrap id=\"T1\"/>"),
    )?)?;
    Ok(())
}
