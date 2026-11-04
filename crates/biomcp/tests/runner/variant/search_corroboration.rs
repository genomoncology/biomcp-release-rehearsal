//! Feature variant-search: only independently confirmed fallback candidates can match.
use super::super::{launch, network_stub::Stub};
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    run(false, true)?;
    run(false, false)?;
    run(true, false)
}
fn run(disable_clinvar: bool, pair: bool) -> Result<(), String> {
    let stub = Stub::for_sources(
        &["myvariant.info", "eutils.ncbi.nlm.nih.gov"],
        |request| {
            if request.contains("efetch.fcgi") {
                "text/plain"
            } else {
                "application/json"
            }
        },
        move |request| body(request, pair),
    )?;
    let env = super::source_environment(&stub)?;
    let replay = env.root().join("recordings");
    let mut args = [
        "search",
        "variant",
        "--gene",
        "BRAF",
        "--protein",
        "V600E",
        "--disable-sources",
        if disable_clinvar {
            "noncommercial,clinvar"
        } else {
            "noncommercial"
        },
        "--record",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    args.push(replay.to_string_lossy().into());
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    check_reply(&value, disable_clinvar, pair)?;
    // Replay the exact recorded exchanges in another isolated installed process.
    let replay_env = launch::Environment::new()
        .map_err(|e| e.to_string())?
        .with_replay_owner(env.shared_root());
    let mut replay_args = args.into_iter().take(8).collect::<Vec<_>>();
    replay_args.extend(["--replay".into(), replay.to_string_lossy().into()]);
    let replay_output = replay_env
        .command(env!("CARGO_BIN_EXE_biomcp"), &replay_args)
        .output()
        .map_err(|e| e.to_string())?;
    let replay_value: Value =
        serde_json::from_slice(&replay_output.stdout).map_err(|e| e.to_string())?;
    if replay_value.pointer("/result") != value.pointer("/result") {
        return Err(format!("replay changed confirmation: {replay_value}"));
    }
    if pair {
        super::search_policy::hosted_http(&replay, "V600E", |value| {
            check_reply(value, false, true)
        })?;
    }
    super::super::network_stub::retain_replay_owner(env.shared_root().as_ref())
        .map_err(|e| e.to_string())?;
    let requests = stub.finish()?;
    check_requests(requests, disable_clinvar, pair)
}
fn check_reply(value: &Value, disable_clinvar: bool, pair: bool) -> Result<(), String> {
    let expected = if disable_clinvar || pair { 1 } else { 4 };
    let total = if pair { 2 } else { 9 };
    let items = value
        .pointer("/result/items")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("no candidates: {value}"))?;
    if items.len() != expected
        || value.pointer("/result/excluded")
            != Some(&json!({
                "count": total - expected, "reason": "reference-unconfirmed"
            }))
        || value.pointer("/page/returned") != Some(&json!(expected))
    {
        return Err(format!("fallback confirmation counts differ: {value}"));
    }
    let ids = items
        .iter()
        .filter_map(|row| row.pointer("/record/id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    for id in [
        "chr7:g.140453136C>T",
        "chr1:g.101C>T",
        "chr1:g.102A>T",
        "chr1:g.103A>T",
        "chr1:g.104A>T",
    ] {
        if ids.contains(&id) {
            return Err(format!("unconfirmed record matched: {id}: {value}"));
        }
    }
    if (!disable_clinvar && !pair && !ids.contains(&"chr2:g.100A>T"))
        || items
            .iter()
            .any(|row| row.pointer("/transcripts/0/matched") != Some(&json!(true)))
    {
        return Err(format!("confirmed reference did not match: {value}"));
    }
    if pair && ids != ["chr7:g.140453136A>T"] || disable_clinvar && ids != ["chr1:g.105A>T"] {
        return Err(format!(
            "same-record allowed evidence selected the wrong identity: {value}"
        ));
    }
    Ok(())
}
fn check_requests(
    requests: super::Requests,
    disable_clinvar: bool,
    pair: bool,
) -> Result<(), String> {
    let reference = requests
        .iter()
        .filter(|(_, request)| request.contains("efetch.fcgi"))
        .collect::<Vec<_>>();
    if reference.len()
        != if disable_clinvar {
            0
        } else if pair {
            1
        } else {
            5
        }
    {
        return Err(format!("unexpected reference lookup count: {requests:?}"));
    }
    for (_, request) in reference {
        if !request.contains("db=nuccore")
            || !request.contains("strand=1")
            || !request.contains("seq_start=")
            || !request.contains("seq_stop=")
        {
            return Err(format!(
                "unbounded or unoriented reference lookup: {request}"
            ));
        }
    }
    Ok(())
}
fn row(id: &str, extra: Value) -> Value {
    let mut row = json!({"_id":id,"snpeff":{"ann":[{"feature_id":"NM_SYNTHETIC.1",
        "feature_type":"transcript","genename":"BRAF","hgvs_p":"p.Val600Glu"}]}});
    if let (Some(row), Some(extra)) = (row.as_object_mut(), extra.as_object()) {
        row.extend(extra.clone());
    }
    row
}
fn body(request: &str, pair: bool) -> Result<String, String> {
    if request.contains("efetch.fcgi") {
        let (accession, position, base) = if request.contains("id=NC_000007.13") {
            ("NC_000007.13", 140453136, "A")
        } else if request.contains("id=NC_000002.12") {
            ("NC_000002.12", 100, "A")
        } else if request.contains("seq_start=101&") {
            ("NC_000001.10", 101, "A")
        } else if request.contains("seq_start=103&") {
            ("NC_000001.10", 999, "A")
        } else if request.contains("seq_start=104&") {
            return Ok("<ERROR>unavailable</ERROR>".into());
        } else {
            return Err(format!("unexpected reference request: {request}"));
        };
        return Ok(format!(
            ">{accession}:{position}-{position} synthetic\n{base}\n"
        ));
    }
    let mut rows = vec![
        row(
            "chr7:g.140453136A>T",
            json!({"clinvar":{"variant_id":13961}}),
        ),
        row(
            "chr7:g.140453136C>T",
            json!({"chrom":"7","hg19":{"start":140453136,"end":140453136}}),
        ),
        row(
            "chr2:g.100A>T",
            json!({"chrom":"2","hg38":{"start":100,"end":100}}),
        ),
        row(
            "chr1:g.101C>T",
            json!({"chrom":"1","hg19":{"start":101,"end":101}}),
        ),
        row(
            "chr1:g.102A>T",
            json!({"chrom":"1","hg19":{"start":102,"end":102},"hg38":{"start":102,"end":102}}),
        ),
        row(
            "chr1:g.103A>T",
            json!({"chrom":"1","hg19":{"start":103,"end":103}}),
        ),
        row(
            "chr1:g.104A>T",
            json!({"chrom":"1","hg19":{"start":104,"end":104}}),
        ),
        row("chr1:g.105A>T", json!({"dbsnp":{"rsid":"rs123"}})),
        row("chr1:g.106A>T", json!({"clinvar":{"variant_id":123}})),
    ];
    rows.truncate(if pair { 2 } else { 9 });
    Ok(json!({"total":rows.len(),"hits":rows}).to_string())
}
