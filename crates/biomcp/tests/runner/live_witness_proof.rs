//! Offline public runner and command regressions for retained diagnostic context.
use super::super::{launch::Environment, live, network_stub::Stub};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs};
#[path = "live_input_context.rs"]
mod input_context;
#[path = "icite_discovery.rs"]
mod icite_discovery;

pub(super) fn retained() -> Result<(), String> {
    let sample = |name: &str, changed: bool, body: Value| {
        json!({ "case": name,
 "result": {"drift": if changed { vec!["field removed"] } else { vec![] }},
 "witness": body })
    };
    let report = live::summary(
        vec![
            sample("clean", false, json!({"output":"must disappear"})),
            sample(
                "drift",
                true,
                json!({"before":{"id":"old"},"after":{"id":"new"}}),
            ),
            sample("large", true, json!({"output":"x".repeat(2 * 1024 * 1024)})),
            json!({"case":"failed","error":"missing continuation","witness":{"baseline":{"id":"old"},"live":{"error":"not JSON"}}}),
        ],
        BTreeMap::new(),
        0,
        true,
    );
    let rows = report
        .get("case_sample")
        .and_then(Value::as_array)
        .ok_or("no rows")?;
    let row = |name| {
        rows.iter()
            .find(|row| row.get("case").and_then(Value::as_str) == Some(name))
            .ok_or("missing row")
    };
    if row("clean")?.get("witness").is_some()
        || row("drift")?.pointer("/witness/after/id") != Some(&json!("new"))
        || row("failed")?.pointer("/witness/baseline/id") != Some(&json!("old"))
        || row("large")?
            .pointer("/witness/output")
            .and_then(Value::as_str)
            .map(str::len)
            != Some(2 * 1024 * 1024)
    {
        return Err("drift witnesses were discarded, retained for clean cases or unbounded".into());
    }
    let big = "x".repeat(600 * 1024);
    let report = live::summary(
        vec![
            sample("first", true, json!(big)),
            sample("second", true, json!(big)),
        ],
        BTreeMap::new(),
        0,
        true,
    );
    let rows = report
        .get("case_sample")
        .and_then(Value::as_array)
        .ok_or("no bounded rows")?;
    let size = rows
        .iter()
        .filter_map(|row| row.get("witness"))
        .try_fold(0, |size, value| {
            serde_json::to_vec(value).map(|bytes| size + bytes.len())
        })
        .map_err(|e| e.to_string())?;
    if size < 1024 * 1024
        || rows
            .iter()
            .any(|row| row.pointer("/witness/cut") == Some(&json!(true)))
    {
        return Err("one case consumed another case witness budget".into());
    }
    many_changed()?;
    long_walk_shapes()?;
    retained_context()?;
    doi_context()?;
    scholar_context().map_err(|error| error.to_string())?;
    discovery_context()
        .and_then(|()| input_context::prove())
        .and_then(|()| icite_discovery::prove())
        .map_err(|error| error.to_string())
}

fn retained_context() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = super::super::launch::fixtures().join("replies/gene/main");
    let mut captured = json!({});
    live::witness::invoke(
        &env,
        &["get".into(), "gene".into(), "KRAS".into()],
        Some(&replay),
        1,
        &mut captured,
        "baseline",
    )?;
    let baseline = captured.get("baseline").ok_or("missing baseline")?;
    if baseline
        .get("result_ids")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
        || baseline
            .pointer("/requests/0/url")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || baseline.pointer("/requests/0/shape").is_none()
        || baseline.pointer("/output/omitted").is_none()
    {
        return Err(
            "actual replay lost identities, safe URLs, shape or mixed-terms suppression".into(),
        );
    }
    let output =
        json!({"id":"native-positive","sources":[{"use":"open"}],"result":{"value":"permitted"}});
    let allowed = live::witness::context(&output, &[]);
    if allowed.get("output") != Some(&output) {
        return Err("permitted product output lost".into());
    }
    let restricted = json!({"sources":[{"use":"restricted"}],"result":{"items":[
        {"pmid":"17611497","pmcid":"PMC123","doi":"10.1000/example"},
        {"ids":["CA123","rs123"]}, {"gene":"DPYD"}, {"pma_number":"P110027"}
    ],"text":"source text must not persist"}});
    let restricted = live::witness::context(&restricted, &[]);
    if restricted
        .get("result_ids")
        .and_then(Value::as_array)
        .map(Vec::len)
        != Some(7)
        || restricted
            .to_string()
            .contains("source text must not persist")
    {
        return Err("restricted article, variant or native gene/device identities were lost or text persisted".into());
    }
    let component = json!({"sources":[{"source":"faers","use":"open"}],"result":{
        "id":"report-1","components":[{"component":"meddra","terms":{"usage":"restricted"}}],"label":"restricted native terminology"}});
    if live::witness::context(&component, &[])
        .to_string()
        .contains("restricted native terminology")
    {
        return Err("an open service bypassed its nested component terms".into());
    }
    Ok(())
}

pub(super) fn credentials() -> Result<(), String> {
    const KEY: &str = "synthetic-nightly-witness-key";
    for query_key in [true, false] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let host = if query_key {
            "eutils.ncbi.nlm.nih.gov"
        } else {
            "clinicaltrialsapi.cancer.gov"
        };
        let stub = Stub::for_sources(
            &[host],
            |_| "application/json",
            move |request| {
                if query_key && !request.contains("api_key=synthetic-nightly-witness-key")
                    || !query_key
                        && !request
                            .to_ascii_lowercase()
                            .contains("x-api-key: synthetic-nightly-witness-key")
                {
                    return Err("synthetic credential was not actually sent".into());
                }
                Ok(if query_key {
                    json!({"esearchresult":{"count":"0","idlist":[]}})
                } else {
                    json!({"total":0,"data":[]})
                }
                .to_string())
            },
        )?;
        let ca = env.root().join("ca.pem");
        stub.retain(env.shared_root());
        fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
        env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"keys":{"ncbi_api_key":KEY,"nci_api_key":KEY}}))
            .map_err(|e| e.to_string())?;
        let observation = env.root().join("observed.jsonl");
        let mut args = if query_key {
            vec!["search", "article", "cancer", "--source", "pubmed"]
        } else {
            vec!["search", "trial", "--source", "nci", "--keyword", "cancer"]
        }
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        args.extend(["--observe".into(), observation.display().to_string()]);
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &args)
            .output()
            .map_err(|e| e.to_string())?;
        let bytes = fs::read(&observation).map_err(|e| e.to_string())?;
        if !output.status.success()
            || bytes.is_empty()
            || [bytes.as_slice(), &output.stdout, &output.stderr]
                .iter()
                .any(|bytes| bytes.windows(KEY.len()).any(|part| part == KEY.as_bytes()))
        {
            return Err("credential reached diagnostic evidence or local command failed".into());
        }
        let records = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        for line in records.lines() {
            let record: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
            let url = record
                .get("url")
                .and_then(Value::as_str)
                .ok_or("no sanitized URL")?;
            if url.contains("api_key")
                || record.get("headers").is_some()
                || record.get("status") != Some(&json!(200))
            {
                return Err(
                    "request witness lost status or retained credential-bearing fields".into(),
                );
            }
        }
    }
    Ok(())
}

fn discovery_context() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let replay = super::super::launch::fixtures().join("replies/article/search/live");
    let requests = super::observe(
        &env,
        &replay,
        "discovery",
        &["search", "article", "AKT1 E17K"],
    )?;
    let discovery = requests
        .iter()
        .find_map(|request| request.provenance.discovery.as_ref())
        .ok_or("checked discovery identities absent")?;
    if discovery.namespace != "pmid" || discovery.ids.len() != 192 {
        return Err("whole native discovery selection was sampled or relabeled".into());
    }
    let inputs = requests
        .iter()
        .find_map(|request| request.provenance.inputs.as_ref())
        .ok_or("explicit dependent POST inputs absent")?;
    if inputs.namespace != "pmid"
        || inputs.ids.is_empty()
        || !inputs.ids.iter().all(|id| discovery.ids.contains(id))
        || requests
            .iter()
            .filter(|request| request.bytes > 0)
            .any(|request| {
                request
                    .provenance
                    .response_digest
                    .as_ref()
                    .is_none_or(|hash| hash.len() != 64)
            })
    {
        return Err("dependent provenance or response digest lost".into());
    }
    Ok(())
}

fn doi_context() -> Result<(), String> {
    let env = Environment::new().map_err(|error| error.to_string())?;
    let replay = super::super::launch::fixtures().join("replies/article/edges/live");
    let rows = super::observe(
        &env,
        &replay,
        "doi-discovery",
        &[
            "article",
            "references",
            "34096690",
            "--source",
            "semantic-scholar",
        ],
    )
    .map_err(|error| error.to_string())?;
    let discovery = rows
        .iter()
        .filter_map(|row| row.provenance.discoveries.as_ref())
        .flatten()
        .find(|ids| ids.namespace == "doi")
        .ok_or("missing graph DOI discovery")?;
    let inputs = rows
        .iter()
        .filter(|row| {
            row.provenance
                .inputs
                .as_ref()
                .is_some_and(|ids| ids.namespace == "doi")
        })
        .collect::<Vec<_>>();
    if discovery.ids.is_empty()
        || inputs.is_empty()
        || inputs.iter().any(|row| {
            row.provenance.input_context.is_none()
                || row.provenance.inputs.as_ref().is_none_or(|ids| {
                    ids.ids.is_empty() || !ids.ids.iter().all(|id| discovery.ids.contains(id))
                })
        })
    {
        return Err("DOI request lost its discovery input or context".into());
    }
    Ok(())
}

fn many_changed() -> Result<(), String> {
    let rows = (0..30)
        .map(|index| {
            json!({"case": format!("changed-{index}"),
        "result": {"drift": ["request: request-plan-changed"]},
        "witness": [{"ordinal":1,"baseline":{"requests":[],"output":"x".repeat(32 * 1024)},
            "live":{"requests":[],"output":"y".repeat(32 * 1024)}}]})
        })
        .collect();
    let report = live::summary(rows, BTreeMap::new(), 0, false);
    let rows = report
        .get("case_sample")
        .and_then(Value::as_array)
        .ok_or("missing changed cases")?;
    if rows.len() != 30
        || rows
            .iter()
            .any(|row| row.get("witness").is_none_or(Value::is_null))
        || report.get("omitted") != Some(&json!(0))
    {
        return Err("changed cases consumed each other's evidence budget".into());
    }
    let oversized = vec![json!({"complete": "x".repeat(17 * 1024 * 1024)})];
    if live::witness::within_budget(&oversized).is_ok() {
        return Err("oversized case evidence did not fail explicitly".into());
    }
    Ok(())
}

fn long_walk_shapes() -> Result<(), String> {
    // Retained nightly changed-case command counts, with whole command contexts.
    let counts = [21, 30, 51, 63, 1, 21, 1, 5, 6, 2];
    let rows = counts.iter().enumerate().map(|(index, count)| {
        let commands = (0..*count).map(|ordinal| json!({"ordinal":ordinal + 1,
            "baseline":{"requests":[],"output":"x".repeat(7000)},
            "live":{"requests":[],"output":"y".repeat(7000)}})).collect::<Vec<_>>();
        json!({"case":format!("retained-walk-{index}"), "result":{"drift":["request: request-plan-changed"]},
            "witness":commands})
    }).collect();
    let report = live::summary(rows, BTreeMap::new(), 0, true);
    let rows = report
        .get("case_sample")
        .and_then(Value::as_array)
        .ok_or("missing retained walks")?;
    for (row, count) in rows.iter().zip(counts) {
        let captures = row
            .get("witness")
            .and_then(Value::as_array)
            .ok_or("cut retained walk")?;
        if captures.len() != count || live::witness::within_budget(captures).is_err() {
            return Err("retained nightly walk lost commands or exceeded its case budget".into());
        }
    }
    if rows.len() != counts.len() {
        return Err("retained nightly walks omitted".into());
    }
    Ok(())
}

fn scholar_context() -> Result<(), Box<dyn std::error::Error>> {
    for (case, args, pmids, doi) in [
        ("single", vec!["article", "recommendations", "34096690", "--limit", "5"],
         vec!["42745055", "42685317", "42846599", "42891951"],
         "10.1158/1538-7445.pancreatic26-b074"),
        ("multi", vec!["article", "recommendations", "34096690", "32955176", "--negative", "33433946", "--limit", "5"],
         vec!["42745055", "42685317", "42748921", "42830785"],
         "10.64898/2026.09.28.26364204"),
    ] {
        let env = Environment::new()?;
        let replay = super::super::launch::fixtures().join(format!("replies/article/recommendations/{case}"));
        let rows = super::observe(&env, &replay, "scholar-identities", &args)?;
        let sequences = rows.iter()
            .find(|row| row.source == biomcp_types::Source::SemanticScholar)
            .and_then(|row| row.provenance.discoveries.as_ref())
            .ok_or("provider namespace sequences absent")?;
        let find = |namespace| sequences.iter().find(|ids| ids.namespace == namespace)
            .map(|ids| ids.ids.iter().map(String::as_str).collect::<Vec<_>>());
        if find("pmid") != Some(pmids) || find("doi") != Some(vec![doi]) {
            return Err("provider PMIDs were inferred, reordered or mixed with DOI fallback".into());
        }
        let metadata: Vec<_> = rows.iter().map(|row| json!({
            "identity": row.identity, "source": row.source, "role": row.role,
            "status": row.status, "url": row.url, "provenance": row.provenance,
        })).collect();
        use std::io::Write;
        writeln!(std::io::stderr(), "SCHOLAR_DISCOVERY {}", json!({"case":case,"requests":metadata}))?;
    }
    Ok(())
}

// A sampler cannot prove absence beyond its field or array window.
pub(super) fn sampling(observations: &[biomcp_fetch::Observation]) -> Result<(), Box<dyn std::error::Error>> {
    use biomcp_fetch::Shape;
    let historical: Shape = serde_json::from_value(json!({"fields": {}, "positive": {}, "header": null}))?;
    if !historical.incomplete || Shape::inspect(br#"{}"#, "application/json").incomplete {
        return Err("unknown historical shape was treated as a complete new inspection".into());
    }
    let sample = observations.first().ok_or("missing native observation")?;
    let report = live::compare(std::slice::from_ref(sample), &[], &mut Vec::new(), &mut BTreeMap::new());
    if report.pointer("/items/0/shape_comparison") != Some(&json!("unexamined")) {
        return Err("unpaired request was described as examined".into());
    }
    let before = Shape::inspect(br#"{"z":1}"#, "application/json");
    let mut crowded = serde_json::Map::new();
    for index in 0..70 {
        crowded.insert(format!("a{index:03}"), json!(index));
    }
    crowded.insert("z".into(), json!(1));
    let crowded = serde_json::to_vec(&crowded)?;
    let arrays = br#"{"rows":[{}, {}, {}, {"optional":1}]}"#;
    let complete = br#"{"other":1}"#;
    for (body, incomplete) in [
        (crowded.as_slice(), true),
        (arrays.as_slice(), true),
        (complete.as_slice(), false),
    ] {
        let mut old = sample.clone();
        old.shape = before.clone();
        let mut new = old.clone();
        new.shape = Shape::inspect(body, "application/json");
        if new.shape.incomplete != incomplete {
            return Err("sampling limit was not reported".into());
        }
        let mut changes = Vec::new();
        let report = live::compare(&[old], &[new], &mut changes, &mut BTreeMap::new());
        if incomplete {
            if !changes.is_empty()
                || report.pointer("/items/0/shape_comparison") != Some(&json!("incomplete"))
            {
                return Err("incomplete sampling invented a field removal or hid its limit".into());
            }
        } else if !changes.iter().any(|item| item.ends_with("field removed: /z")) {
            return Err("complete shape lost genuine removal detection".into());
        }
    }
    Ok(())
}
