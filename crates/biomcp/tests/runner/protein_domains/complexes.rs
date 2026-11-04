use super::{Environment, Stub, Value, call, json, setup};
pub(super) fn run() -> Result<(), String> {
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", reply)?;
    let env = setup(&stub)?;
    let first = call(&env, &["protein", "complexes", "P38398"], 0)?;
    super::super::protein_budgets::check(&first, "synthetic-complex/headers", "cli")?;
    if first.pointer("/result/native_total") != Some(&json!(3))
        || first.pointer("/page/total") != Some(&json!(2))
        || first.pointer("/result/items/0/predicted") != Some(&json!(true))
    {
        return Err(format!(
            "description match admitted or predicted flag changed: {first}"
        ));
    }
    let pool = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("complex pool")?;
    let kept = call(
        &env,
        &[
            "protein",
            "complexes",
            "P38398",
            "--pool",
            pool,
            "--offset",
            "0",
        ],
        0,
    )?;
    if kept.pointer("/result/items/0/predicted") != Some(&json!(true))
        || kept.pointer("/page/total") != Some(&json!(2))
    {
        return Err("retained complex page changed its source rows".into());
    }
    let step = super::super::action(&first, "section")?;
    let words: Vec<_> = step.iter().map(String::as_str).collect();
    let members = call(&env, &words, 0)?;
    super::super::protein_budgets::check(&members, "synthetic-complex/members-first", "cli")?;
    if members.pointer("/page/total") != Some(&json!(93))
        || members.pointer("/page/returned") != Some(&json!(5))
        || members.pointer("/result/items/1/protein").is_some()
        || members.pointer("/result/items/1/organism") != Some(&json!("chemical synthesis; -2"))
    {
        return Err(format!(
            "native non-protein facts or bounded page lost: {members}"
        ));
    }
    walk(&env, members)?;
    rejected(&env)?;
    if stub.finish()?.len() != 6 {
        return Err("retained member paging made extra provider requests".into());
    }
    Ok(())
}
fn walk(env: &Environment, members: Value) -> Result<(), String> {
    let pool = members
        .pointer("/page/pool")
        .cloned()
        .ok_or("member pool")?;
    let mut value = members;
    let mut ids = Vec::new();
    loop {
        ids.extend(
            value
                .pointer("/result/items")
                .and_then(Value::as_array)
                .ok_or("members")?
                .iter()
                .map(|r| r.get("id").and_then(Value::as_str).unwrap_or("").to_owned()),
        );
        if value
            .pointer("/page/offset")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + value
                .pointer("/page/returned")
                .and_then(Value::as_u64)
                .unwrap_or(0)
            >= 93
        {
            break;
        }
        let action = super::super::action(&value, "page")?;
        let words: Vec<_> = action.iter().map(String::as_str).collect();
        value = call(env, &words, 0)?;
        super::super::protein_budgets::check(
            &value,
            &format!(
                "synthetic-complex/member-offset-{}",
                value
                    .pointer("/page/offset")
                    .and_then(Value::as_u64)
                    .ok_or("offset")?
            ),
            "cli",
        )?;
        if value.pointer("/page/pool") != Some(&pool) {
            return Err("member continuation renewed the retained snapshot".into());
        }
    }
    if ids.len() != 93 || ids.iter().collect::<std::collections::BTreeSet<_>>().len() != 93 {
        return Err("member walk lost or duplicated native rows".into());
    }
    Ok(())
}
fn rejected(env: &Environment) -> Result<(), String> {
    let wrong = call(
        env,
        &[
            "protein",
            "complex-members",
            "P04637",
            "--complex",
            "CPX-715",
        ],
        4,
    )?;
    super::super::protein_budgets::check(&wrong, "synthetic-complex/wrong-anchor", "cli")?;
    if wrong.pointer("/problems/0/record") != Some(&json!("CPX-715"))
        || wrong.pointer("/problems/0/field") != Some(&json!("participants.anchor.P04637"))
        || wrong.get("result").is_some()
    {
        return Err(format!("wrong anchor exposed member rows: {wrong}"));
    }
    let mismatch = call(
        env,
        &[
            "protein",
            "complex-members",
            "P38398",
            "--complex",
            "CPX-999",
        ],
        4,
    )?;
    super::super::protein_budgets::check(&mismatch, "synthetic-complex/wrong-complex", "cli")?;
    if mismatch.pointer("/sources/0/reason") != Some(&json!("bad-reply")) {
        return Err("mismatched complex identity admitted".into());
    }
    call(
        env,
        &[
            "protein",
            "complex-members",
            "P38398",
            "--complex",
            "CPX-999",
        ],
        4,
    )?;
    for complex in ["CPX-716", "CPX-717"] {
        let rejected = call(
            env,
            &["protein", "complex-members", "P38398", "--complex", complex],
            4,
        )?;
        super::super::protein_budgets::check(
            &rejected,
            &format!("synthetic-complex/participant-whitespace-{complex}"),
            "cli",
        )?;
        if rejected.get("result").is_some() {
            return Err("native participant whitespace established a protein join".into());
        }
    }
    Ok(())
}

fn member(id: String, kind: &str) -> Value {
    json!(
        {"identifier":id,"interactorType":kind,"organismName":if kind=="protein"{"Homo sapiens; 9606"}else{"chemical synthesis; -2"},
        "name":"Native participant","stochiometry":"minValue: 1, maxValue: 2","xrefs":[{"database":"native","identifier":"identifier"}]}
    )
}
pub(super) fn reply(request: &str) -> Result<String, String> {
    if request.contains("/complex/CPX-716") || request.contains("/complex/CPX-717") {
        let id = if request.contains("CPX-716") {
            "CPX-716"
        } else {
            "CPX-717"
        };
        let row = if id == "CPX-716" {
            member(" P38398 ".into(), "protein")
        } else {
            member("P38398".into(), " protein ")
        };
        return Ok(json!(
            {"complexAc":id,"name":"Hostile participant identity","species":"Homo sapiens; 9606","predictedComplex":false,"participants":[row]}
        )
        .to_string());
    }
    if request.contains("/complex/") {
        return Ok(json!(
            {"complexAc":"CPX-715","name":"Exact complex","species":"Homo sapiens; 9606","predictedComplex":false,
            "participants":[member("P38398".into(),"protein"),member("Q99728".into(),"protein")]}
        )
        .to_string());
    }
    let mut members = vec![
        member("P38398".into(), "protein"),
        member("CHEBI:29105".into(), "small molecule"),
    ];
    for n in 1..92 {
        members.push(member(format!("P{n:05}"), "protein"));
    }
    Ok(json!(
        {"size":3,"totalNumberOfResults":3,"elements":[ {"complexAC":"CPX-100","complexName":"Native predicted complex",
        "organismName":"Homo sapiens; 9606","predictedComplex":true,"interactors":members},{"complexAC":"CPX-715",
        "complexName":"Exact complex","organismName":"Homo sapiens; 9606","predictedComplex":false,"interactors":[member("P38398".into(),
        "protein"),member("Q99728".into(),"protein")]},{"complexAC":"CPX-26353","complexName":"Mentions BRCA1","description":"P38398",
        "organismName":"Homo sapiens; 9606","predictedComplex":false,"interactors":[member("P15056".into(),"protein")]} ]}
    )
    .to_string())
}
