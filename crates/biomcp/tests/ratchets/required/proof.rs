//! Small counterexamples exercise the same inventory used by the repository check.
use super::{Check, Value, case_data::Step, tally::Inventory};
use serde_json::json;
use std::collections::BTreeSet;

fn help() -> Value {
    json!({"result":{"commands":[
        {"command":[],"arguments":[{"name":"--format","value_type":"string"}]},
        {"command":["get","trial"],"arguments":[
            {"name":"--limit","value_type":"string","aliases":["-n"]},
            {"name":"--section","value_type":"string"}
        ],"sections":[{"word":"eligibility"}]}
    ]}})
}
fn step(args: &[&str]) -> Result<Step, Box<dyn std::error::Error>> {
    Ok(serde_json::from_value(
        json!({"run":args,"expect":{"/result/id":"NCT1"}}),
    )?)
}
fn covered() -> Result<Inventory, Box<dyn std::error::Error>> {
    let mut inventory = Inventory::new(&help(), &[])?;
    inventory.step(&step(&[
        "get",
        "trial",
        "NCT1",
        "eligibility",
        "--limit",
        "1",
    ])?);
    inventory.step(&step(&["--format=md", "get", "trial", "NCT1", "-n", "2"])?);
    inventory.step(&step(&["get", "trial", "eligibility"])?);
    Ok(inventory)
}
fn pending(inventory: &Inventory) -> BTreeSet<String> {
    inventory
        .calls
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(name, _)| name.clone())
        .collect()
}
fn three_hand_counts_match_and_an_identity_never_counts_as_a_section() -> Check {
    let inventory = covered()?;
    assert_eq!(
        inventory.calls.get("command get trial"),
        Some(&3),
        "wrong call count for command get trial"
    );
    assert_eq!(
        inventory.calls.get("option get trial --limit"),
        Some(&2),
        "wrong call count for option get trial --limit"
    );
    assert_eq!(
        inventory.calls.get("section get trial eligibility"),
        Some(&1),
        "wrong call count for section get trial eligibility"
    );
    assert_eq!(
        inventory.calls.get("global --format"),
        Some(&1),
        "wrong call count for global --format"
    );
    inventory.check(&pending(&inventory))
}
fn a_new_option_without_a_case_fails() -> Check {
    let original = covered()?;
    let mut help = help();
    help.pointer_mut("/result/commands/1/arguments")
        .and_then(Value::as_array_mut)
        .ok_or("missing proof arguments")?
        .push(json!({"name":"--planted","value_type":"boolean"}));
    let mut changed = Inventory::new(&help, &[])?;
    changed.step(&step(&[
        "get",
        "trial",
        "NCT1",
        "eligibility",
        "--limit",
        "1",
        "--format=json",
    ])?);
    let failure = changed
        .check(&pending(&original))
        .err()
        .ok_or("uncovered option passed")?
        .to_string();
    assert!(failure.contains("option get trial --planted"), "{failure}");
    Ok(())
}
fn removing_the_only_section_call_fails() -> Check {
    let original = covered()?;
    let mut changed = Inventory::new(&help(), &[])?;
    changed.step(&step(&[
        "get",
        "trial",
        "NCT1",
        "--limit",
        "1",
        "--format=json",
    ])?);
    let failure = changed
        .check(&pending(&original))
        .err()
        .ok_or("uncovered section passed")?
        .to_string();
    assert!(
        failure.contains("section get trial eligibility"),
        "{failure}"
    );
    Ok(())
}
fn option_values_cannot_impersonate_options_or_sections() -> Check {
    let mut inventory = Inventory::new(&help(), &[])?;
    inventory.step(&step(&["get", "trial", "NCT1", "--limit", "eligibility"])?);
    assert_eq!(
        inventory.calls.get("section get trial eligibility"),
        Some(&0),
        "wrong call count for section get trial eligibility"
    );
    inventory.step(&step(&["--format=--limit", "get", "trial", "NCT1"])?);
    assert_eq!(
        inventory.calls.get("option get trial --limit"),
        Some(&1),
        "wrong call count for option get trial --limit"
    );
    Ok(())
}
fn cli_commands_never_impersonate_named_mcp_tools() -> Check {
    let tools = [
        json!({"name":"biomcp"}),
        json!({"name":"search"}),
        json!({"name":"fetch"}),
    ];
    let mut inventory = Inventory::new(&help(), &tools)?;
    inventory.step(&step(&["search", "trial", "melanoma"])?);
    assert_eq!(
        inventory.calls.get("tool search"),
        Some(&0),
        "wrong call count for tool search"
    );
    let named: Step = serde_json::from_value(
        json!({"mcp_call":{"name":"search","arguments":{"query":"trial melanoma"}},"expect":{"/results":[]}}),
    )?;
    inventory.step(&named);
    assert_eq!(
        inventory.calls.get("tool search"),
        Some(&1),
        "wrong call count for tool search"
    );
    assert_eq!(
        inventory.calls.get("tool biomcp"),
        Some(&0),
        "wrong call count for tool biomcp"
    );
    let legacy: Step =
        serde_json::from_value(json!({"mcp":["help"],"expect":{"/result/commands":[]}}))?;
    inventory.step(&legacy);
    assert_eq!(
        inventory.calls.get("tool biomcp"),
        Some(&1),
        "wrong call count for tool biomcp"
    );
    assert_eq!(
        inventory.calls.get("tool fetch"),
        Some(&0),
        "wrong call count for tool fetch"
    );
    Ok(())
}

fn dynamic_follow_never_counts_overridden_argv() -> Check {
    let mut inventory = Inventory::new(&help(), &[json!({"name":"biomcp"})])?;
    let dynamic: Step = serde_json::from_value(json!({"follow":"page",
        "run":["get","trial","NCT1","eligibility","--limit","1"],
        "mcp":["help"], "expect":{"/result/id":"NCT1"}}))?;
    inventory.step(&dynamic);
    assert!(
        inventory.calls.values().all(|count| *count == 0),
        "follow replaces static argv and clears MCP in the executor"
    );
    Ok(())
}

fn only_trailing_words_select_implicit_sections() -> Check {
    let mut inventory = Inventory::new(&help(), &[])?;
    for argv in [
        vec!["get", "trial", "NCT1", "eligibility", "NCT2"],
        vec!["get", "trial", "NCT1", "eligibility", "--section=other"],
    ] {
        inventory.step(&step(&argv)?);
    }
    assert_eq!(
        inventory.calls.get("section get trial eligibility"),
        Some(&0),
        "only the grammar's trailing suffix can select implicit sections"
    );
    inventory.step(&step(&[
        "get",
        "trial",
        "eligibility",
        "--section=eligibility",
    ])?);
    assert_eq!(
        inventory.calls.get("section get trial eligibility"),
        Some(&1),
        "an explicit section must still earn coverage"
    );
    Ok(())
}

fn explicit_equals_refusal_counts() -> Check {
    let mut inventory = Inventory::new(&help(), &[])?;
    for code in [
        "usage.arguments",
        "usage.schema",
        "usage.unknown_command",
        "usage.retired",
    ] {
        let refusal: Step = serde_json::from_value(json!({
            "run":["get","trial"], "exit":2,
            "expect":{"/error/code":{"equals":code}}
        }))?;
        inventory.step(&refusal);
    }
    assert_eq!(
        inventory.calls.get("refusal get trial"),
        Some(&4),
        "explicit equality must count the same refusal as a scalar assertion"
    );
    Ok(())
}

fn label_absence_requires_the_successful_label_source() -> Check {
    let mut help = help();
    help.pointer_mut("/result/commands")
        .and_then(Value::as_array_mut)
        .ok_or("proof commands")?
        .push(json!({
            "command":["get","drug"],"arguments":[],"sections":[{"word":"label"}]
        }));
    let mut inventory = Inventory::new(&help, &[])?;
    for (source, state) in [
        (None, "ok"),
        (Some("mychem"), "ok"),
        (Some("openfda-label"), "failed"),
        (Some("openfda-label"), "ok"),
    ] {
        let mut expect = json!({"/status":"absent", "/sources/0/state":state});
        if let Some(source) = source {
            expect
                .as_object_mut()
                .ok_or("proof expectations")?
                .insert("/sources/0/source".into(), json!(source));
        }
        let step: Step = serde_json::from_value(json!({
            "run":["get","drug","UNKNOWN","label"],"exit":1,"expect":expect
        }))?;
        inventory.step(&step);
    }
    assert_eq!(
        inventory.calls.get("failure label not-found"),
        Some(&1),
        "only successful label-source absence earns label credit"
    );
    assert_eq!(
        inventory.calls.get("failure trial not-found"),
        Some(&0),
        "label absence never earns trial credit"
    );
    assert!(
        !inventory.calls.contains_key("failure label expired-pool"),
        "label has no expired-pool surface"
    );
    assert!(
        !inventory.calls.contains_key("failure variant expired-pool"),
        "variant has no expired-pool surface"
    );
    Ok(())
}

fn trial_absence_requires_the_successful_registry_source() -> Check {
    let mut inventory = Inventory::new(&help(), &[])?;
    for (source, state, item, status) in [
        (Some("ctgov"), Some("ok"), Some("absent"), "absent"),
        (None, Some("ok"), Some("absent"), "absent"),
        (Some("nci"), Some("ok"), Some("absent"), "absent"),
        (Some("ctgov"), Some("failed"), Some("absent"), "absent"),
        (Some("ctgov"), None, Some("absent"), "absent"),
        (Some("ctgov"), Some("ok"), None, "absent"),
        (Some("ctgov"), Some("ok"), Some("failed"), "absent"),
        (Some("ctgov"), Some("ok"), Some("absent"), "partial"),
    ] {
        let mut expect = json!({"/status":status});
        let fields = expect.as_object_mut().ok_or("proof expectations")?;
        if let Some(state) = state {
            fields.insert("/sources/0/state".into(), json!(state));
        }
        if let Some(source) = source {
            fields.insert("/sources/0/source".into(), json!(source));
        }
        if let Some(item) = item {
            fields.insert("/result/items/0/status".into(), json!(item));
        }
        let step: Step = serde_json::from_value(json!({
            "run":["get","trial","NCT01811710"],"exit":1,"expect":expect
        }))?;
        inventory.step(&step);
    }
    assert_eq!(
        inventory.calls.get("failure trial not-found"),
        Some(&1),
        "only a successful checked registry absence earns trial absence credit"
    );
    Ok(())
}

pub(super) fn check() -> Check {
    label_absence_requires_the_successful_label_source()?;
    trial_absence_requires_the_successful_registry_source()?;
    only_trailing_words_select_implicit_sections()?;
    explicit_equals_refusal_counts()?;
    three_hand_counts_match_and_an_identity_never_counts_as_a_section()?;
    a_new_option_without_a_case_fails()?;
    removing_the_only_section_call_fails()?;
    option_values_cannot_impersonate_options_or_sections()?;
    cli_commands_never_impersonate_named_mcp_tools()?;
    dynamic_follow_never_counts_overridden_argv()
}
