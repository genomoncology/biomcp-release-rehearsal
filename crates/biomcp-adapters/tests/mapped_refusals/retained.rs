//! Sources without a mapped record-decline branch retain their explicit None decision.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, nci, openfda_label};
use biomcp_fetch::RequestError;
use biomcp_types::Reason;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "drug-label/retained-label-fields-do-not-become-refusals",
            || labels().map_err(|e| e.to_string().into()),
        ),
        trial(
            "organization-nci/retained-organization-fields-do-not-become-refusals",
            || organizations().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn labels() -> Check {
    for case in ["present", "empty", "container", "member"] {
        let mut changed = false;
        let (_root, fetch) = replay("drug/label-discovery", |source, recording| {
            if source == "openfda-label"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let Ok(mut document) = serde_json::from_str::<Value>(native) else {
                    return;
                };
                if let Some(row) = document.pointer_mut("/results/0") {
                    changed = true;
                    replace(
                        row,
                        "/boxed_warning",
                        &(match case {
                            "present" => json!(["Native text"]),
                            "empty" => json!([]),
                            "container" => json!({"invalid":true}),
                            _ => json!([false, "Native text"]),
                        }),
                    );
                    *body = json!(document.to_string());
                }
            }
        })?;
        if !changed {
            return Err("openFDA native label revision missing".into());
        }
        let leg = fetch.leg(
            &openfda_label::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let page = openfda_label::get(&leg, &"98b197ea-bca4-427e-8737-97f70a3262bc".parse()?)?;
        if page.window.refusals() != Refusal::None
            || page.window.rows.len() != 1
            || page.window.received() != 1
        {
            return Err(format!("openFDA {case}: retained field became a record refusal").into());
        }
    }
    Ok(())
}

fn organizations() -> Check {
    for invalid_record in [false, true] {
        let mut changed = false;
        let mut native_count = 0;
        let (_root, fetch) = replay("organization/adapter-values/nci", |source, recording| {
            if source == "nci"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|url| url.contains("org_city=Boston"))
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value/data")
            {
                changed = true;
                native_count = rows.as_array().map_or(0, Vec::len);
                if invalid_record {
                    if let Some(slot) = rows.pointer_mut("/0") {
                        *slot = json!(false);
                    };
                } else {
                    replace(rows, "/0/contact_name", &(json!({"invalid":true})));
                    replace(rows, "/0/org_coordinates", &(json!({"invalid":true})));
                }
            }
        })?;
        if !changed {
            return Err("NCI native organization records missing".into());
        }
        let leg = fetch.leg(&nci::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({"city":"Boston","state":"MA","source":"nci"}))?;
        let result = nci::organization::search(&leg, &query);
        if invalid_record {
            if !matches!(result,Err(RequestError::Source(error)) if error.reason==Reason::BadReply)
            {
                return Err(
                    "NCI malformed whole record did not preserve protocol rejection".into(),
                );
            }
        } else {
            let mapped = result?;
            if mapped.refusals != Refusal::None
                || mapped.value.len() != native_count
                || mapped.problems.is_empty()
            {
                return Err("NCI retained optional fields became record refusals".into());
            }
        }
    }
    Ok(())
}
