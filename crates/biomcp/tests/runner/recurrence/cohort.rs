//! Real-sized cohorts report whole-cohort counts in one call, and page beyond the default.
use crate::runner::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};

const COMMON_ROWS: usize = 1_670;
const LARGE_ROWS: usize = 6_000;
const SELECTED_SAMPLES: u64 = 10_945;
const MEMBER: &str = "synthetic-list-member";

pub(super) fn check() -> Result<(), String> {
    common()?;
    large()?;
    uncounted()
}

fn common() -> Result<(), String> {
    let (env, stub) = serve(COMMON_ROWS)?;
    let reply = super::call(&env, &default_page(), 0)?;
    let result = reply.pointer("/result").ok_or("recurrence result")?;
    if result.get("observed_sample_count") != Some(&json!(matched(COMMON_ROWS)))
        || result.get("selected_sample_count") != Some(&json!(SELECTED_SAMPLES))
        || result.get("examined_rows") != Some(&json!(COMMON_ROWS))
        || result.get("all_pages_read") != Some(&json!(true))
        || result.get("more") != Some(&json!(false))
    {
        return Err(format!("common variant lost its cohort counts: {reply}"));
    }
    if reply.to_string().contains("percent") || result.get("frequency").is_some() {
        return Err(format!("recurrence computed a percentage: {reply}"));
    }
    if reply.to_string().contains(MEMBER) {
        return Err(format!("a list member reached the reply: {reply}"));
    }
    let reads = mutation_reads(stub)?;
    if reads != 1 {
        return Err(format!("common variant took {reads} source reads"));
    }
    Ok(())
}

fn large() -> Result<(), String> {
    let (env, stub) = serve(LARGE_ROWS)?;
    let mut args = default_page();
    args.extend(["--disable-sources".into(), "hotspots".into()]);
    let first = super::call(&env, &args, 0)?;
    if first.pointer("/result/more") != Some(&json!(true))
        || first.pointer("/result/all_pages_read") != Some(&json!(false))
    {
        return Err(format!("large cohort claimed to be complete: {first}"));
    }
    let mut next: Vec<String> = serde_json::from_value(
        first
            .pointer("/next/0/args")
            .cloned()
            .ok_or_else(|| format!("page step lost without Cancer Hotspots: {first}"))?,
    )
    .map_err(|e| e.to_string())?;
    next.extend(["--disable-sources".into(), "hotspots".into()]);
    let last = super::call(&env, &next, 0)?;
    if last.pointer("/result/more") != Some(&json!(false))
        || last.pointer("/result/all_pages_read") != Some(&json!(false))
    {
        return Err(format!("last page lost its page-local state: {last}"));
    }
    let observed = [&first, &last]
        .iter()
        .filter_map(|reply| reply.pointer("/result/observed_sample_count"))
        .filter_map(Value::as_u64)
        .sum::<u64>();
    if observed != matched(LARGE_ROWS) as u64 {
        return Err(format!("pages counted {observed} samples"));
    }
    let reads = mutation_reads(stub)?;
    if reads != 2 {
        return Err(format!("large cohort took {reads} source reads"));
    }
    Ok(())
}

fn uncounted() -> Result<(), String> {
    let (env, stub) = serve_list(COMMON_ROWS, list(None))?;
    let reply = super::call(&env, &default_page(), 0)?;
    stub.finish()?;
    if reply.pointer("/result/selected_sample_count").is_some()
        || !reply
            .to_string()
            .contains("sample_set.selected_sample_count")
    {
        return Err(format!(
            "a missing list size was not shown missing: {reply}"
        ));
    }
    Ok(())
}

fn default_page() -> Vec<String> {
    let mut args = super::args();
    args.truncate(args.len() - 2);
    args
}

fn matched(rows: usize) -> usize {
    rows.div_ceil(3)
}

fn mutation_reads(stub: Stub) -> Result<usize, String> {
    Ok(stub
        .finish()?
        .iter()
        .filter(|(_, request)| request.contains("/mutations?"))
        .count())
}

fn serve(rows: usize) -> Result<(Environment, Stub), String> {
    serve_list(rows, list(Some(SELECTED_SAMPLES)))
}

/// The selected list's own record, with its count as given and its member IDs.
fn list(count: Option<u64>) -> Value {
    let mut selected = json!({
        "sampleListId":"msk_impact_2017_all","studyId":"msk_impact_2017",
        "category":"all_cases_in_study","name":"Synthetic list","description":"Synthetic list",
        "sampleIds":[MEMBER]
    });
    if let (Some(count), Some(fields)) = (count, selected.as_object_mut()) {
        fields.insert("sampleCount".into(), json!(count));
    }
    selected
}

/// The source's study-wide reads: SUMMARY states no count, and a large study's DETAILED reply
/// is 2.5 MB and more.
fn study_lists(request: &str) -> String {
    let members = if request.contains("projection=DETAILED") {
        vec![MEMBER; 150_000]
    } else {
        Vec::new()
    };
    json!([{
        "studyId":"msk_impact_2017","sampleListId":"msk_impact_2017_all",
        "category":"all_cases_in_study","name":"Synthetic list","description":"Synthetic list",
        "sampleIds":members
    }])
    .to_string()
}

fn serve_list(rows: usize, list: Value) -> Result<(Environment, Stub), String> {
    let stub = Stub::for_sources(
        &[
            "www.cbioportal.org",
            "reg.genome.network",
            "storage.googleapis.com",
        ],
        |request| {
            if request.contains("hgnc_complete_set") {
                "text/tab-separated-values"
            } else {
                "application/json"
            }
        },
        move |request| reply(request, rows, &list),
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    super::config(&env, &stub)?;
    Ok((env, stub))
}

fn reply(request: &str, rows: usize, list: &Value) -> Result<String, String> {
    if request.starts_with("GET /api/sample-lists/") {
        return Ok(list.to_string());
    }
    if request.contains("/sample-lists?") {
        return Ok(study_lists(request));
    }
    if !request.contains("/mutations?") {
        return super::cbio(request);
    }
    let size = number(request, "pageSize=")?;
    let start = number(request, "pageNumber=")? * size;
    let page: Vec<Value> = (start..rows.min(start + size)).map(source_row).collect();
    Ok(Value::Array(page).to_string())
}

fn number(request: &str, name: &str) -> Result<usize, String> {
    request
        .split_once(name)
        .and_then(|(_, rest)| rest.split(['&', ' ']).next())
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("{name} missing from {request}"))
}

/// The field set cBioPortal returns for its SUMMARY projection, with synthetic values.
fn source_row(index: usize) -> Value {
    let alternate = if index.is_multiple_of(3) { "T" } else { "G" };
    let sample = format!("synthetic-sample-{index}");
    let mut row = super::row(&sample, alternate);
    let extra = json!({
        "uniqueSampleKey":format!("synthetic-sample-key-{index}"),
        "uniquePatientKey":format!("synthetic-patient-key-{index}"),
        "patientId":format!("synthetic-patient-{index}"),
        "center":"NA","mutationStatus":"SOMATIC","validationStatus":"Unknown",
        "tumorAltCount":40,"tumorRefCount":160,"proteinChange":"V600E",
        "mutationType":"Missense_Mutation","variantType":"SNP",
        "refseqMrnaId":"NM_004333.4","proteinPosStart":600,"proteinPosEnd":600,
        "keyword":"BRAF V600 missense"
    });
    if let (Some(row), Some(extra)) = (row.as_object_mut(), extra.as_object()) {
        row.extend(extra.clone());
    }
    row
}
