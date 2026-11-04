//! Retained parent properties and actually declined native records carry different facts.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, civic, icite, litvar, myvariant, oncokb, pubtator};
use biomcp_types::Section;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "variant-identity/refused-cosmic-children-carry-facts",
            || cosmic().map_err(|e| e.to_string().into()),
        ),
        trial("gene-sections/refused-civic-genes-carry-facts", || {
            civic_genes().map_err(|e| e.to_string().into())
        }),
        trial("gene-sections/refused-oncokb-genes-carry-facts", || {
            oncokb_genes().map_err(|e| e.to_string().into())
        }),
        trial(
            "article-citations/refused-citation-records-carry-facts",
            || citations().map_err(|e| e.to_string().into()),
        ),
        trial(
            "article-search/refused-litvar-candidates-carry-facts",
            || candidates().map_err(|e| e.to_string().into()),
        ),
        trial(
            "article-sections/refused-documents-and-children-carry-facts",
            || annotations().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn cosmic() -> Check {
    for case in [
        "optional",
        "object-identity",
        "array-identity",
        "empty-container",
        "invalid-container",
        "absent",
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/synthetic-components", |source, recording| {
            if source == "myvariant"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && body.get("cosmic").is_some()
            {
                changed = true;
                match case {
                    "optional" => replace(body, "/cosmic/mut_freq", &(json!({"invalid":true}))),
                    "object-identity" => replace(body, "/cosmic/cosmic_id", &(json!(false))),
                    "array-identity" => replace(body, "/cosmic", &(json!([{"cosmic_id":false}]))),
                    "empty-container" => replace(body, "/cosmic", &(json!([]))),
                    "invalid-container" => {
                        replace(body, "/cosmic", &(json!("invalid optional container")))
                    }
                    _ => replace(body, "/cosmic", &(Value::Null)),
                }
            }
        })?;
        if !changed {
            return Err("MyVariant native COSMIC fragment missing".into());
        }
        let leg = fetch.leg(&myvariant::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = myvariant::cosmic::get(&leg, "chr7:g.140453136A>T", biodata::Build::GRCh37)?;
        let expected = if matches!(case, "object-identity" | "array-identity") {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || mapped.value.records.len() != usize::from(case == "optional")
        {
            return Err(format!(
                "COSMIC {case}: actual child refusal or optional container changed"
            )
            .into());
        }
        if matches!(
            case,
            "object-identity" | "array-identity" | "invalid-container"
        ) && !mapped
            .problems
            .iter()
            .any(|p| p.field.as_ref() == "cosmic.cosmic_id")
        {
            return Err("COSMIC source problem wording changed".into());
        }
    }
    Ok(())
}

fn civic_genes() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/civic", |source, recording| {
            if source == "civic"
                && let Some(row) = recording.pointer_mut("/exchanges/0/body/value/data/g0")
            {
                changed = true;
                match case {
                    "optional" => replace(row, "/variants", &(json!("invalid optional container"))),
                    "identity" => replace(row, "/id", &(json!(0))),
                    _ => *row = Value::Null,
                }
            }
        })?;
        if !changed {
            return Err("CIViC native gene missing".into());
        }
        let leg = fetch.leg(&civic::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mut rows = civic::genes(&leg, &[("ERBB2".parse()?, biodata::NcbiGeneId::new(2064)?)])?;
        let mapped = rows.pop().ok_or("CIViC alias missing")?.civic;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || matches!(mapped.value, Section::Data { .. }) != (case == "optional")
        {
            return Err(
                format!("CIViC {case}: actual gene refusal or retained container changed").into(),
            );
        }
    }
    Ok(())
}

fn oncokb_genes() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/oncokb-synthetic/data", |source, recording| {
            if source == "oncokb"
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value")
                && rows.is_array()
            {
                changed = true;
                match case {
                    "optional" => replace(rows, "/0/geneType", &(json!({"invalid":true}))),
                    "identity" => replace(rows, "/0/entrezGeneId", &(json!(0))),
                    _ => *rows = json!([]),
                }
            }
        })?;
        if !changed {
            return Err("OncoKB native gene missing".into());
        }
        let leg = fetch.leg(&oncokb::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = oncokb::gene(&leg, &"TP53".parse()?, biodata::NcbiGeneId::new(7157)?)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.is_some() != (case == "optional") {
            return Err(
                format!("OncoKB {case}: actual gene refusal or retained field changed").into(),
            );
        }
    }
    Ok(())
}

fn citations() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay(
            "article/citations/synthetic-malformed",
            |source, recording| {
                if source == "icite"
                    && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value/data")
                {
                    changed = true;
                    match case {
                        "identity" => replace(rows, "/0/pmid", &(json!(0))),
                        "absent" => *rows = json!([]),
                        _ => {}
                    }
                }
            },
        )?;
        if !changed {
            return Err("iCite native records missing".into());
        }
        let leg = fetch.leg(&icite::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = icite::publications(&leg, &[biodata::Pmid::new(33123537)?])?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(
                format!("iCite {case}: actual record refusal or native absence changed").into(),
            );
        }
        if case == "optional"
            && mapped
                .value
                .first()
                .is_none_or(|row| row.problems.is_empty())
        {
            return Err("iCite optional-field control missed native problems".into());
        }
    }
    Ok(())
}

fn candidates() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay(
            "article/search/synthetic-mixed-dropped-words",
            |source, recording| {
                if source == "litvar"
                    && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value")
                    && rows.is_array()
                {
                    changed = true;
                    let Some(mut row) = rows.get(0).cloned() else {
                        return;
                    };
                    replace(&mut row, "/name", &(json!({"invalid":true})));
                    if case == "identity" {
                        replace(&mut row, "/_id", &(json!(0)));
                    }
                    *rows = if case == "absent" {
                        json!([])
                    } else {
                        json!([row])
                    };
                }
            },
        )?;
        if !changed {
            return Err("LitVar native candidates missing".into());
        }
        let leg = fetch.leg(&litvar::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = litvar::select(&leg, None, "rs121434592")?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || matches!(mapped.value, litvar::Selection::Exact(_)) != (case == "optional")
        {
            return Err(format!(
                "LitVar {case}: actual candidate refusal or native absence changed"
            )
            .into());
        }
    }
    Ok(())
}

fn annotations() -> Check {
    for case in [
        "present",
        "empty-container",
        "invalid-container",
        "identity",
        "child",
        "absent",
    ] {
        let mut changed = false;
        let (_root, fetch) = replay(
            "article/annotations/synthetic-key-domains",
            |source, recording| {
                if source == "pubtator"
                    && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value/PubTator3")
                {
                    changed = true;
                    match case {
                        "empty-container" => replace(rows, "/0/passages", &(json!([]))),
                        "invalid-container" => {
                            replace(rows, "/0/passages", &(json!("invalid optional container")))
                        }
                        "identity" => replace(rows, "/0/id", &(json!("invalid"))),
                        "child" => replace(
                            rows,
                            "/0/passages/0/annotations",
                            &(json!([{"infons":{"type":"Variant"}}])),
                        ),
                        "absent" => *rows = json!([]),
                        _ => {}
                    }
                }
            },
        )?;
        if !changed {
            return Err("PubTator native document missing".into());
        }
        let leg = fetch.leg(&pubtator::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mut rows = pubtator::annotations(&leg, &[biodata::Pmid::new(33123537)?])?;
        let mapped = rows
            .pop()
            .ok_or("PubTator requested document result missing")?
            .annotations;
        let expected = if matches!(case, "identity" | "child") {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || matches!(mapped.value, Section::Data { .. }) != (case == "present")
        {
            return Err(
                format!("PubTator {case}: actual document or child refusal changed").into(),
            );
        }
    }
    Ok(())
}
