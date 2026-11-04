//! Feature variant-alphagenome: the CLI preserves native groups through a local TLS gRPC peer.
use super::{alphagenome::invoke, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
#[path = "alphagenome_capture.rs"]
mod capture;
use biomcp_test_support::grpc as support;
const SECTION: &str = "/result/candidates/0/sections/alphagenome";
const KEY: &str = "synthetic-alphagenome-cli-placeholder";
pub(super) fn trials() -> Vec<Trial> {
    let mut trials = installed_trials();
    trials.push(transcript_trial(
        "variant-alphagenome/alphagenome-source-identity-refusals",
        || refusals().map_err(Into::into),
    ));
    trials.push(transcript_trial(
        "variant-alphagenome/alphagenome-field-admission-local-tls",
        || field_admission().map_err(Into::into),
    ));
    trials.extend(capture::trials());
    trials
}
pub(super) fn installed_trials() -> Vec<Trial> {
    vec![transcript_trial(
        crate::runner::required_contract::ALPHAGENOME.0,
        || run().map_err(Into::into),
    )]
}
fn run() -> Result<(), String> {
    for (status, id) in [
        (200, "CA123643"),
        (200, "rs113488022"),
        (401, "CA123643"),
        (429, "CA123643"),
        (500, "CA123643"),
        (999, "CA123643"),
    ] {
        let messages = if status == 999 {
            vec![vec![10, 255]]
        } else {
            messages()
        };
        let fixture = fixture(messages, if status == 999 { 200 } else { status })?;
        let env = environment(&fixture)?;
        let args = ["get", "variant", id, "alphagenome"].map(str::to_owned);
        let reply = invoke(&env, &args, 0)?;
        if status == 200 {
            successful(&env, id, &args, &reply)?;
        } else {
            let reason = match status {
                401 => "rejected",
                429 => "rate-limited",
                500 => "unavailable",
                _ => "bad-reply",
            };
            if reply.pointer(&format!("{SECTION}/state")) != Some(&json!("unavailable"))
                || reply.pointer(&format!("{SECTION}/reason")) != Some(&json!(reason))
                || reply.pointer(&format!("{SECTION}/data")).is_some()
            {
                return Err(format!(
                    "source failure {status} exposed data or lost its reason"
                ));
            }
        }
        routed(&fixture, if status == 200 { 5 } else { 1 })?;
        fixture.finish().map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn successful(
    env: &Environment,
    id: &str,
    command: &[String],
    reply: &Value,
) -> Result<(), String> {
    groups(reply)?;
    let mcp = super::mcp::call_with_globals(env, command, false, &[])?;
    for response in [reply, &mcp] {
        if id.starts_with("rs") {
            if response.pointer("/query/build") != Some(&json!("GRCh38"))
                || response.pointer("/query/build_defaulted") != Some(&json!(true))
            {
                return Err("AlphaGenome identity lookup lost its chosen assembly default".into());
            }
        } else if response.pointer("/query/build").is_some()
            || response.pointer("/query/build_defaulted").is_some()
        {
            return Err("CA identity lookup gained an unused assembly default".into());
        }
    }
    if mcp.get("result") != reply.get("result") || mcp.get("next") != reply.get("next") {
        return Err("native CLI and MCP groups or continuations differ".into());
    }
    let args = page_args(reply)?;
    let later = invoke(env, &args, 0)?;
    let later_mcp = super::mcp::call_with_globals(env, &args, false, &[])?;
    if later.get("result") != later_mcp.get("result") {
        return Err("native continuation differs through MCP".into());
    }
    if later.pointer(&format!("{SECTION}/data/scorers/0/rows/0/value"))
        != Some(&json!({"kind":"negative-infinity"}))
    {
        return Err("native negative infinity was dropped or converted".into());
    }
    let markdown = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &[
                "get",
                "variant",
                id,
                "alphagenome",
                "--section-limit",
                "4",
                "--format",
                "md",
            ]
            .map(str::to_owned),
        )
        .output()
        .map_err(|e| e.to_string())?;
    let markdown = String::from_utf8(markdown.stdout).map_err(|e| e.to_string())?;
    if !["NaN", "+Inf", "-Inf"]
        .into_iter()
        .all(|label| markdown.contains(label))
    {
        return Err("Markdown lost native nonfinite scalar labels".into());
    }
    if !markdown.contains("--section-offset") {
        return Err("Markdown omitted executable scorer paging".into());
    }
    if later.pointer(&format!("{SECTION}/data/scorers/0/rows/0/tensor_index"))
        != Some(&json!([0, 1, 1]))
        || later.pointer(&format!("{SECTION}/data/scorers/1/page/returned")) != Some(&json!(0))
    {
        return Err("scorers did not page independently in native order".into());
    }
    Ok(())
}
fn page_args(reply: &Value) -> Result<Vec<String>, String> {
    reply
        .get("next")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|step| step.get("kind") == Some(&json!("page")))
        .and_then(|step| step.get("args"))
        .and_then(Value::as_array)
        .ok_or("missing executable scorer continuation")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "invalid continuation argument".into())
        })
        .collect()
}
fn routed(fixture: &support::Fixture, expected: usize) -> Result<(), String> {
    let mut resolved = false;
    let mut requests = 0;
    while let Ok(event) = fixture.events.try_recv() {
        match event {
            support::Seen::Http(request) => {
                if request.starts_with("GET /v1/query?") && request.contains("assembly=hg38") {
                    continue;
                }
                if !request.starts_with("GET /allele/CA123643?")
                    && !request.starts_with("GET /allele?hgvs=NC_000007.14%3Ag.140753336A%3ET")
                {
                    return Err("unexpected resolver request".into());
                }
                resolved = true;
            }
            support::Seen::Path(path)
                if path
                    != "/google.gdm.gdmscience.alphagenome.v1main.DnaModelService/ScoreVariant" =>
            {
                return Err("section dispatched the wrong scoring method".into());
            }
            support::Seen::Request(headers, body) => {
                if !resolved
                    || headers.get("x-goog-api-key").and_then(|v| v.to_str().ok()) != Some(KEY)
                    || body.get(5..) != Some(expected_request().as_slice())
                {
                    return Err(
                        "scoring preceded resolution or changed keyed request mapping".into(),
                    );
                }
                requests += 1;
            }
            _ => {}
        }
    }
    if requests != expected {
        return Err("scoring request retried or was omitted".into());
    }
    Ok(())
}
fn refusals() -> Result<(), String> {
    for (build, coordinates, mitochondrial, reason) in [
        ("GRCh37", true, false, "unsupported"),
        ("GRCh38", false, false, "no-join-id"),
        ("GRCh38", true, true, "unsupported"),
    ] {
        let expression = if mitochondrial {
            "NC_012920.1:m.100A>T"
        } else if build == "GRCh37" {
            "NC_000007.13:g.140453136A>T"
        } else {
            "NC_000007.14:g.140753336A>T"
        };
        let position = if mitochondrial {
            99
        } else if coordinates {
            140453135
        } else {
            140753335
        };
        let reference = if coordinates { "A" } else { "AA" };
        let allele = json!({
            "@id":"http://reg.genome.network/allele/CA123643",
            "genomicAlleles":[{
                "referenceGenome":build, "hgvs":[expression],
                "coordinates":[{"start":position,"end":position + 1,"referenceAllele":reference,"allele":"T"}]
            }]
        });
        let fixture = source_fixture(messages(), 200, allele)?;
        let env = environment(&fixture)?;
        let reply = invoke(
            &env,
            &["get", "variant", "CA123643", "alphagenome"].map(str::to_owned),
            0,
        )?;
        if reply.pointer(&format!("{SECTION}/reason")) != Some(&json!(reason))
            || reply.pointer(&format!("{SECTION}/data")).is_some()
        {
            return Err("unsupported source identity was scored".into());
        }
        let mut identity_requests = 0;
        while let Ok(event) = fixture.events.try_recv() {
            match event {
                support::Seen::Request(_, _) => {
                    return Err("refused identity reached scoring RPC".into());
                }
                support::Seen::Http(_) => identity_requests += 1,
                _ => {}
            }
        }
        if identity_requests != 1 {
            return Err("refusal omitted source identity resolution".into());
        }
        fixture.finish().map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn groups(reply: &Value) -> Result<(), String> {
    let rows = reply
        .pointer(&format!("{SECTION}/data/scorers"))
        .and_then(Value::as_array)
        .ok_or("missing scorer groups")?;
    if rows.len() != 3 {
        return Err("native scorer groups were collapsed".into());
    }
    for (ordinal, (group, shape)) in rows
        .iter()
        .zip([[2, 2, 2], [2, 1, 1], [1, 1, 2]])
        .enumerate()
    {
        if group.get("shape") != Some(&json!(shape))
            || group.get("request_ordinal") != Some(&json!(ordinal))
            || group.get("data_type") != Some(&json!("DATA_TYPE_FLOAT32"))
            || group.get("units").is_some()
            || group.get("model_version").is_some()
        {
            return Err("group provenance or native scalar representation changed".into());
        }
    }
    if rows.first().and_then(|g| g.pointer("/rows/0/value")) != Some(&json!(0.25))
        || rows.first().and_then(|g| g.pointer("/rows/1/value")) != Some(&json!({"kind":"nan"}))
        || rows.first().and_then(|g| g.pointer("/rows/2/value"))
            != Some(&json!({"kind":"positive-infinity"}))
        || rows.first().and_then(|g| g.pointer("/rows/2/tensor_index")) != Some(&json!([0, 1, 0]))
        || rows
            .get(2)
            .and_then(|g| g.pointer("/rows/0/gene"))
            .is_some()
        || rows.first().and_then(|g| g.get("next")).is_none()
        || rows.get(1).and_then(|g| g.get("next")).is_some()
    {
        return Err(
            "values, coordinates, nongene metadata or independent next pages changed".into(),
        );
    }
    let native = rows
        .first()
        .and_then(|group| group.pointer("/rows/0"))
        .ok_or("first scalar row")?;
    if native.get("track")
        != Some(&json!({
            "name":"synthetic track 0","strand":"STRAND_UNSTRANDED",
            "ontology_term":{"namespace":"ONTOLOGY_TYPE_UBERON","id":2048},
            "biosample":{"type":"BIOSAMPLE_TYPE_TISSUE","name":"synthetic tissue","stage":"synthetic stage"},
            "assay":"synthetic assay","gtex_tissue":"synthetic GTEx tissue","histone_mark_code":"synthetic histone",
            "transcription_factor_code":"synthetic transcription factor","data_source":"synthetic data source",
            "endedness":"ENDEDNESS_PAIRED","genetically_modified":true,"nonzero_mean":1.25
        }))
        || native.get("gene")
            != Some(&json!({"gene_id":"ENSG00000100000","name":"synthetic gene",
        "strand":"STRAND_POSITIVE","type":"synthetic gene type","junction_start":10,"junction_end":20}))
    {
        return Err("native track or gene metadata fields changed".into());
    }
    Ok(())
}
fn field_admission() -> Result<(), String> {
    for (mode, field, omitted) in [
        ("variant", "variant", ""),
        ("track", "tracks.0.name", ""),
        ("gene", "genes.0.gene_id", ""),
        ("endedness", "tracks.0.endedness", "track/endedness"),
        (
            "ontology",
            "tracks.0.ontology_term.ontology_type",
            "track/ontology_term",
        ),
        (
            "biosample",
            "tracks.0.biosample.type",
            "track/biosample/type",
        ),
        ("gene-strand", "genes.0.strand", "gene/strand"),
    ] {
        let fixture = fixture(changed_messages(mode), 200)?;
        let env = environment(&fixture)?;
        let args = [
            "get",
            "variant",
            "CA123643",
            "alphagenome",
            "--section-limit",
            "1",
        ]
        .map(str::to_owned);
        let reply = invoke(&env, &args, 0)?;
        let scorer = format!("{SECTION}/data/scorers/0");
        let expected = format!("scorers.0.{field}");
        let problems = reply
            .get("problems")
            .and_then(Value::as_array)
            .ok_or("missing field problem")?;
        if problems.len() != 1
            || problems.first().and_then(|problem| problem.get("field")) != Some(&json!(expected))
        {
            return Err(format!(
                "{mode} lost its single scoped field problem: {problems:?}"
            ));
        }
        if omitted.is_empty() {
            if reply.pointer(&format!("{scorer}/page/returned")) != Some(&json!(0))
                || reply.pointer(&format!("{scorer}/rows")).is_some()
            {
                return Err(format!("{mode} admitted a scalar with invalid identity"));
            }
        } else if reply.pointer(&format!("{scorer}/rows/0/tensor_index")) != Some(&json!([0, 0, 0]))
            || reply
                .pointer(&format!("{scorer}/rows/0/{omitted}"))
                .is_some()
        {
            return Err(format!(
                "{mode} lost its scalar or retained the invalid field"
            ));
        }
        let next = invoke(&env, &page_args(&reply)?, 0)?;
        let expected_returned = usize::from(!matches!(mode, "gene" | "variant"));
        if next.pointer(&format!("{scorer}/page/offset")) != Some(&json!(1))
            || next.pointer(&format!("{scorer}/page/returned")) != Some(&json!(expected_returned))
            || next.pointer(&format!("{SECTION}/data/scorers/1/rows/0/gene/gene_id"))
                != Some(&json!("ENSG00000100000"))
        {
            return Err(format!(
                "{mode} shifted native positions or refused a valid sibling"
            ));
        }
        fixture.finish().map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn fixture(messages: Vec<Vec<u8>>, status: u16) -> Result<support::Fixture, String> {
    let allele = json!({"@id":"http://reg.genome.network/allele/CA123643","genomicAlleles":[{
        "referenceGenome":"GRCh38","hgvs":["NC_000007.14:g.140753336A>T"],
        "coordinates":[{"start":140753335,"end":140753336,"referenceAllele":"A","allele":"T"}]}]});
    source_fixture(messages, status, allele)
}
fn source_fixture(
    messages: Vec<Vec<u8>>,
    status: u16,
    allele: Value,
) -> Result<support::Fixture, String> {
    support::Fixture::with_http(
        support::Hold::None,
        0,
        true,
        false,
        messages,
        Some(support::HttpReply {
            hosts: vec![
                "gdmscience.googleapis.com".into(),
                "reg.genome.network".into(),
                "myvariant.info".into(),
            ],
            replies: vec![
                ("GET /allele".into(), allele.to_string().into_bytes()),
                (
                    "GET /v1/query?".into(),
                    json!({"total":1,"hits":[{"_id":"chr7:g.140753336A>T",
                        "chrom":"7","hg38":{"start":140753336,"end":140753336},
                        "clingen":{"caid":"CA123643"},"dbsnp":{"rsid":"rs113488022"}}]})
                    .to_string()
                    .into_bytes(),
                ),
            ],
        }),
        status,
    )
    .map_err(|e| e.to_string())
}
fn environment(fixture: &support::Fixture) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    std::fs::write(&ca, &fixture.certificate).map_err(|e| e.to_string())?;
    env.config(
        &json!({"proxy":format!("http://{}",fixture.address), "ca_bundle":ca,
        "keys":{"alphagenome_api_key":KEY}}),
    )
    .map_err(|e| e.to_string())?;
    Ok(env)
}
fn messages() -> Vec<Vec<u8>> {
    changed_messages("")
}
fn changed_messages(mode: &str) -> Vec<Vec<u8>> {
    [[2, 2, 2], [2, 1, 1], [1, 1, 2]]
        .into_iter()
        .enumerate()
        .flat_map(|(ordinal, shape)| {
            let tensor = [field(1, &shape), number(2, 2), number(4, 1)].concat();
            let mut metadata = field(
                1,
                &variant(if mode == "variant" && ordinal == 0 {
                    140753337
                } else {
                    140753336
                }),
            );
            for track in 0..shape[2] {
                metadata.extend(field(
                    2,
                    &track_metadata(track, if ordinal == 0 && track == 0 { mode } else { "" }),
                ));
            }
            if ordinal < 2 {
                for row in 0..shape[1] {
                    metadata.extend(field(
                        3,
                        &gene_metadata(row, if ordinal == 0 && row == 0 { mode } else { "" }),
                    ));
                }
            }
            let descriptor = field(
                1,
                &field(1, &[field(1, &tensor), field(2, &metadata)].concat()),
            );
            let data = (0..usize::from(shape[0]) * usize::from(shape[1]) * usize::from(shape[2]))
                .flat_map(|index| {
                    match (ordinal, index) {
                        (0, 1) => f32::NAN,
                        (0, 2) => f32::INFINITY,
                        (0, 3) => f32::NEG_INFINITY,
                        _ => index as f32 + 0.25,
                    }
                    .to_le_bytes()
                })
                .collect::<Vec<_>>();
            [descriptor, field(2, &field(1, &data))]
        })
        .collect()
}
fn track_metadata(track: u8, mode: &str) -> Vec<u8> {
    let mut mean = vec![(14 << 3) | 5];
    mean.extend(1.25f32.to_le_bytes());
    [
        field(
            1,
            if mode == "track" {
                String::new()
            } else {
                format!("synthetic track {track}")
            }
            .as_bytes(),
        ),
        number(2, 3),
        field(
            3,
            &[
                number(1, if mode == "ontology" { 999 } else { 2 }),
                number(2, 2048),
            ]
            .concat(),
        ),
        field(
            4,
            &[
                number(1, if mode == "biosample" { 999 } else { 4 }),
                field(2, b"synthetic tissue"),
                field(3, b"synthetic stage"),
            ]
            .concat(),
        ),
        field(5, b"synthetic assay"),
        field(8, b"synthetic GTEx tissue"),
        field(9, b"synthetic histone"),
        field(10, b"synthetic transcription factor"),
        field(11, b"synthetic data source"),
        number(12, if mode == "endedness" { 999 } else { 2 }),
        number(13, 1),
        mean,
    ]
    .concat()
}
fn gene_metadata(row: u8, mode: &str) -> Vec<u8> {
    [
        field(
            1,
            if mode == "gene" {
                "invalid-gene".to_owned()
            } else {
                format!("ENSG{:011}", 100_000 + u64::from(row))
            }
            .as_bytes(),
        ),
        number(2, if mode == "gene-strand" { 999 } else { 1 }),
        field(3, b"synthetic gene"),
        field(4, b"synthetic gene type"),
        number(5, 10),
        number(6, 20),
    ]
    .concat()
}
fn expected_request() -> Vec<u8> {
    [
        field(
            1,
            &[
                field(1, b"chr7"),
                number(2, 140491191),
                number(3, 141015479),
                number(4, 3),
            ]
            .concat(),
        ),
        field(2, &variant(140753336)),
        number(3, 9606),
        field(4, &field(2, &number(1, 4))),
        field(4, &field(3, &number(2, 7))),
        field(
            4,
            &field(1, &[number(1, 501), number(2, 1), number(3, 3)].concat()),
        ),
    ]
    .concat()
}
fn variant(position: u64) -> Vec<u8> {
    [
        field(1, b"chr7"),
        number(2, position),
        field(3, b"A"),
        field(4, b"T"),
    ]
    .concat()
}
fn number(tag: u8, mut value: u64) -> Vec<u8> {
    let mut bytes = vec![tag << 3];
    while value > 127 {
        bytes.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}
fn field(tag: u8, data: &[u8]) -> Vec<u8> {
    let mut bytes = number(tag, data.len() as u64);
    if let Some(first) = bytes.first_mut() {
        *first |= 2;
    }
    bytes.extend(data);
    bytes
}
