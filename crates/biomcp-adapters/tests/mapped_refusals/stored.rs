//! Stored native exports retain actual row decisions at synchronization and query boundaries.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, clingen, clinvar, ddinter, erepo, wikipathways};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("gene-sections/refused-dosage-records-carry-facts", || {
            dosage().map_err(|e| e.to_string().into())
        }),
        trial("drug-ddinter/refused-export-rows-carry-sync-facts", || {
            pairs().map_err(|e| e.to_string().into())
        }),
        trial(
            "pathway-wikipathways/refused-catalog-members-carry-facts",
            || catalog().map_err(|e| e.to_string().into()),
        ),
        trial(
            "variant-search/refused-clinvar-identities-carry-facts",
            || candidates().map_err(|e| e.to_string().into()),
        ),
        trial(
            "variant-assertions/refused-erepo-records-carry-facts",
            || assertions().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn dosage() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/clingen", |source, recording| {
            if source == "clingen"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.contains("gene-dosage"))
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let replacement = if invalid {
                    native.replacen("\"TP53\",\"HGNC:11998\"", "\"TP53\",\"invalid\"", 1)
                } else {
                    native.replace("Sufficient Evidence for Haploinsufficiency", " ")
                };
                changed = replacement != native;
                *body = json!(replacement);
            }
        })?;
        if !changed {
            return Err("ClinGen dosage fixture edit missed native row".into());
        }
        clingen::sync(&fetch)?;
        let leg = fetch.leg(&clingen::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = clingen::dosage(&leg, "HGNC:11998".parse()?, &"TP53".parse()?)?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || mapped.value.is_some() == invalid
        {
            return Err("ClinGen actual dosage refusal or optional assessment changed".into());
        }
    }
    Ok(())
}

fn pairs() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("drug/ddinter-synthetic", |source, recording| {
            if source == "ddinter"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.ends_with("code_A.csv"))
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let replacement = if invalid {
                    format!(
                        "{native}invalid,Synthetic declined drug,DDInter99,Synthetic partner,Moderate\r\n"
                    )
                } else {
                    native.replace("Moderate", " ")
                };
                changed = replacement != native;
                *body = json!(replacement);
            }
        })?;
        if !changed {
            return Err("DDInter native CSV fixture edit did not apply".into());
        }
        let mapped = ddinter::sync(&fetch)?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
        {
            return Err(
                "DDInter sync lost actual decoder refusal or retained optional level".into(),
            );
        }
    }
    Ok(())
}

fn catalog() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("pathway/examples-sync", |source, recording| {
            if source == "wikipathways"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.contains("findPathwaysByText"))
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let Ok(mut catalog) = serde_json::from_str::<Value>(native) else {
                    return;
                };
                if let Some(row) = catalog.pointer_mut("/pathwayInfo/0") {
                    changed = true;
                    if invalid {
                        replace(row, "/id", &(json!(false)));
                    } else {
                        replace(row, "/name", &(json!({"invalid":true})));
                    }
                    *body = json!(catalog.to_string());
                }
            }
        })?;
        if !changed {
            return Err("WikiPathways native catalog member missing".into());
        }
        wikipathways::sync(&fetch)?;
        let leg = fetch.leg(&wikipathways::SPEC, fetch.deadline(Duration::from_secs(20)));
        let (mapped, _) = wikipathways::search(&leg, "")?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || mapped.value.is_empty()
        {
            return Err(
                "WikiPathways lost actual catalog member refusal or retained optional name".into(),
            );
        }
    }
    Ok(())
}

fn candidates() -> Check {
    for case in ["present", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/search/coding-pten", |source, recording| {
            if source == "clinvar"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let Ok(mut document) = serde_json::from_str::<Value>(native) else {
                    return;
                };
                if let Some(rows) = document.pointer_mut("/esearchresult/idlist") {
                    changed = true;
                    *rows = match case {
                        "identity" => json!([false]),
                        "absent" => json!([]),
                        _ => json!(["1321696"]),
                    };
                    if case == "absent" {
                        replace(&mut document, "/esearchresult/count", &(json!("0")));
                    }
                    *body = json!(document.to_string());
                }
            }
        })?;
        if !changed {
            return Err("ClinVar native idlist missing".into());
        }
        let leg = fetch.leg(&clinvar::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({"gene":"PTEN","coding":"c.333G>T","limit":50}))?;
        let page = clinvar::search(&leg, &query).map_err(|error| {
            format!("ClinVar {case}: native boundary rejected fixture: {error}")
        })?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if page.rows.refusals != expected || page.rows.value.len() != usize::from(case == "present")
        {
            return Err(
                format!("ClinVar {case}: actual search ID refusal or absence changed").into(),
            );
        }
    }
    Ok(())
}

fn assertions() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/assertions/erepo-current", |source, recording| {
            if source == "erepo"
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value/data")
            {
                changed = true;
                match case {
                    "optional" => replace(rows, "/0/condition", &(json!({"invalid":true}))),
                    "identity" => replace(rows, "/0/PCERDocID", &(json!(" "))),
                    _ => *rows = json!([]),
                }
            }
        })?;
        if !changed {
            return Err("eRepo native assertions missing".into());
        }
        let leg = fetch.leg(&erepo::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = erepo::assertions(&leg, "CA000434", None)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(
                format!("eRepo {case}: actual assertion refusal or absence changed").into(),
            );
        }
    }
    Ok(())
}
