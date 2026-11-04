//! Positive controls change only the native response body and its output assertions.
use super::Owner;
use crate::runner::Step;
use crate::runner::transcript;
use serde_json::{Value, json};
pub(super) fn prepare(
    owner: &Owner,
    first: &Step,
    control: &str,
    wire: &str,
) -> Result<(Step, String), String> {
    let mut body: Value = serde_json::from_str(wire).map_err(|e| e.to_string())?;
    if owner.string {
        body = serde_json::from_str(body.as_str().ok_or("native JSON-string wrapper")?)
            .map_err(|e| e.to_string())?;
    }
    let mut step = first.clone();
    if control != "refused" {
        step.expect.clear();
        let count = usize::from(control == "optional");
        if count == 0 && owner.count.ends_with("items") {
            let (parent, field) = owner.count.rsplit_once('/').ok_or("native items pointer")?;
            step.expect.insert(parent.into(), json!({"omits":[field]}));
        } else {
            step.expect.insert(
                owner.count.into(),
                if owner.count.ends_with("items") {
                    json!({"count":count})
                } else {
                    json!(count)
                },
            );
        }
        if !owner.total.is_empty() {
            step.expect.insert(
                owner.total.into(),
                json!(if control == "target-refused" {
                    1
                } else {
                    count
                }),
            );
        }
        if control == "optional" || control == "target-refused" {
            *body
                .pointer_mut(owner.identity)
                .ok_or("accepted native identity slot")? =
                serde_json::from_str(owner.accepted_identity).map_err(|e| e.to_string())?;
            let field = if control == "target-refused" {
                *body
                    .pointer_mut("/mechanisms/0/target_chembl_id")
                    .ok_or("native target identity")? = json!({"invalid":true});
                "target_chembl_id"
            } else {
                owner.optional
            };
            step.expect
                .insert("/problems/*/field".into(), json!({"includes": field}));
        } else {
            *body.pointer_mut(owner.rows).ok_or("native record array")? = json!([]);
            if !owner.total_field.is_empty() {
                *body.pointer_mut(owner.total_field).ok_or("native total")? = json!(0);
            }
        }
    }
    // Empty metadata study sections retain their successfully loaded study identity.
    if control == "empty" && owner.feature != "study-get" {
        if step
            .run
            .as_ref()
            .is_some_and(|args| args.first().is_some_and(|arg| arg == "search"))
        {
            step.expect.insert("/status".into(), json!("complete"));
        } else {
            step.exit = 1;
            if owner.feature == "disease-seer" {
                step.expect.insert("/status".into(), json!("absent"));
            }
        }
    }
    let wire = if owner.string {
        json!(body.to_string()).to_string()
    } else {
        body.to_string()
    };
    Ok((step, wire))
}

pub(super) fn kinds(owner: &Owner) -> &'static [&'static str] {
    if owner.name == "chembl-mechanisms" {
        &["empty", "optional", "refused", "target-refused"]
    } else {
        &["empty", "optional", "refused"]
    }
}

pub(super) fn requests(
    owner: &Owner,
    control: &str,
    target: &str,
    requests: &[(String, String)],
) -> Result<Option<String>, String> {
    if let Some(transcript) = transcript::current() {
        transcript.record(
            &format!("cache/{}/{control}/requests", owner.name),
            serde_json::to_value(requests).map_err(|error| error.to_string())?,
        )?;
    }
    let received = requests
        .iter()
        .filter(|(_, request)| request.lines().next() == Some(target))
        .count();
    let expected = if control.ends_with("refused") { 2 } else { 1 };
    Ok((received != expected).then(|| {
        format!(
            "{} {control}: expected {expected} native requests, received {received}",
            owner.name
        )
    }))
}
