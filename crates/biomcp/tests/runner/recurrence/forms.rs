//! Each published identifier family reaches the same source-qualified cohort observation.
use crate::runner::{launch::Environment, mcp, network_stub::Stub};
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    let stub = Stub::for_sources(
        &[
            "www.cbioportal.org",
            "reg.genome.network",
            "storage.googleapis.com",
            "myvariant.info",
        ],
        |r| {
            if r.contains("hgnc_complete_set") {
                "text/tab-separated-values"
            } else {
                "application/json"
            }
        },
        reply,
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    super::config(&env, &stub)?;
    shared_forms(&env)?;
    rejections(&env)?;
    let help = super::call(
        &env,
        &["help".into(), "variant".into(), "recurrence".into()],
        0,
    )?;
    if !help
        .to_string()
        .contains(biomcp_types::VariantRecurrenceQuery::INPUT_REQUIREMENT)
    {
        return Err("recurrence help lost its exact-replacement requirement".into());
    }
    for input in [
        "rs113488022",
        "CA123643",
        "clinvar:13961",
        "VCV000013961",
        "NC_000007.13:g.140453136A>T",
        "chr7:g.140753336A>T",
        "7-140753336-A-T",
    ] {
        let mut args = super::args();
        *args.get_mut(2).ok_or("recurrence input")? = input.into();
        args.truncate(args.len() - 2);
        let cli = super::call(&env, &args, 0)?;
        observed(&cli, input)?;
        let single = mcp::call_with_globals(&env, &args, false, &[])?;
        observed(&single, input)?;
        let typed = typed(
            &env,
            "variant_recurrence",
            json!({
                "input":input,"source":"cbioportal","gene":"BRAF",
                "study":"msk_impact_2017","profile":"msk_impact_2017_mutations",
                "sample_set":"msk_impact_2017_all"
            }),
        )?;
        observed(&typed, input)?;
    }
    stub.finish()?;
    Ok(())
}
fn observed(value: &Value, input: &str) -> Result<(), String> {
    if value.pointer("/query/input") != Some(&json!(input))
        || value.pointer("/result/observed_sample_count") != Some(&json!(1))
        || value.pointer("/result/selected_sample_count") != Some(&json!(100))
        || value.pointer("/result/all_pages_read") != Some(&json!(true))
        || value.pointer("/result/matching_granularity") != Some(&json!("exact-genomic-allele"))
    {
        return Err(format!(
            "{input} did not reach the exact cohort observation: {value}"
        ));
    }
    Ok(())
}
fn reply(request: &str) -> Result<String, String> {
    if request.starts_with("GET /v1/") {
        let hit = json!({
            "_id":"chr7:g.140753336A>T", "chrom":"7",
            "hg38":{"start":140753336,"end":140753336},
            "dbsnp":{"rsid":"rs113488022"},"clinvar":{"variant_id":13961},
            "clingen":{"caid":"CA123643"}
        });
        return Ok(if request.starts_with("GET /v1/query?") {
            json!({"total":1,"hits":[hit]})
        } else {
            hit
        }
        .to_string());
    }
    if request.starts_with("GET /allele?") {
        return super::car(request);
    }
    super::cbio(request)
}

fn shared_forms(env: &Environment) -> Result<(), String> {
    let mut definitions = Vec::new();
    for (_, entity, id) in biomcp_types::command_paths()
        .into_iter()
        .filter(|(verb, entity, _)| {
            matches!(
                (*verb, *entity),
                ("variant", "recurrence" | "population" | "assertions") | ("get", "variant")
            )
        })
    {
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &["schema".into(), id.into()])
            .output()
            .map_err(|e| e.to_string())?;
        let schema: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        let name = if entity == "recurrence" {
            "RecurrenceInput"
        } else {
            "VariantLookupInput"
        };
        if schema.pointer("/properties/input/$ref") != Some(&json!(format!("#/$defs/{name}"))) {
            return Err(format!(
                "{id} no longer uses the shared variant input contract"
            ));
        }
        if entity == "recurrence"
            && !schema
                .pointer(&format!("/$defs/{name}/description"))
                .and_then(Value::as_str)
                .is_some_and(|text| {
                    text.contains(biomcp_types::VariantRecurrenceQuery::INPUT_REQUIREMENT)
                })
        {
            return Err("recurrence schema lost its exact-replacement requirement".into());
        }
        definitions.push(
            schema
                .pointer(&format!("/$defs/{name}/pattern"))
                .cloned()
                .ok_or("variant input schema")?,
        );
    }
    let first = definitions
        .first()
        .ok_or("variant lookup schemas missing")?;
    if definitions.iter().any(|definition| definition != first) {
        return Err("variant lookup commands advertise different identifier forms".into());
    }
    let validator = jsonschema::validator_for(&json!({"type":"string","pattern":first}))
        .map_err(|e| e.to_string())?;
    for input in UNSUPPORTED {
        if validator.is_valid(&json!(input)) {
            return Err(format!(
                "lookup schema advertises unrouted identifier {input}"
            ));
        }
    }
    Ok(())
}

const UNSUPPORTED: [&str; 3] = [
    "PA123",
    "COSV60000001",
    "ga4gh:VA.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
];
fn rejections(env: &Environment) -> Result<(), String> {
    for input in UNSUPPORTED {
        let mut recurrence = super::args();
        *recurrence.get_mut(2).ok_or("recurrence input")? = input.into();
        for (args, tool, arguments) in [
            (
                vec!["get".into(), "variant".into(), input.into()],
                "get_variant",
                json!({"input":input}),
            ),
            (
                vec!["variant".into(), "population".into(), input.into()],
                "variant_population",
                json!({"input":input}),
            ),
            (
                vec![
                    "variant".into(),
                    "assertions".into(),
                    input.into(),
                    "--source".into(),
                    "clinvar".into(),
                ],
                "variant_assertions",
                json!({"input":input,"source":"clinvar"}),
            ),
            (
                recurrence,
                "variant_recurrence",
                json!({"input":input,"source":"cbioportal","gene":"BRAF",
                "study":"msk_impact_2017","profile":"msk_impact_2017_mutations","sample_set":"msk_impact_2017_all","limit":3}),
            ),
        ] {
            for value in [super::call(env, &args, 2)?, typed(env, tool, arguments)?] {
                if tool == "variant_recurrence"
                    && !value.pointer("/error/valid_values").is_some_and(|values| {
                        values
                            .to_string()
                            .contains(biomcp_types::VariantRecurrenceQuery::INPUT_REQUIREMENT)
                    })
                {
                    return Err(format!(
                        "recurrence error lost its exact-replacement requirement: {value}"
                    ));
                }
                if value.pointer("/error/code") != Some(&json!("usage.arguments")) {
                    return Err(format!(
                        "unrouted identifier did not fail command admission: {value}"
                    ));
                }
            }
        }
    }
    Ok(())
}
fn typed(env: &Environment, tool: &str, arguments: Value) -> Result<Value, String> {
    let frame = mcp::request(
        json!(1),
        "tools/call",
        json!({"name":tool,"arguments":arguments}),
    );
    let replies =
        mcp::exchange_with_globals(env, &[frame], &["--tool-mode".into(), "typed".into()])?;
    replies
        .first()
        .and_then(|r| r.pointer("/result/structuredContent"))
        .cloned()
        .ok_or_else(|| format!("missing typed {tool} reply: {replies:?}"))
}
