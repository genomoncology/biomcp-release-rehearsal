//! Source identity records stay distinct from retained metadata and native validation outcomes.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, allele_registry, mutalyzer, reactome, uniprot};
use libtest_mimic::Trial;
use serde_json::json;
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "variant-normalization/refused-registry-identity-carries-fact",
            || registry().map_err(|e| e.to_string().into()),
        ),
        trial(
            "variant-normalization/refused-mutalyzer-identity-carries-fact",
            || normalized().map_err(|e| e.to_string().into()),
        ),
        trial("gene-protein/refused-protein-records-carry-facts", || {
            proteins().map_err(|e| e.to_string().into())
        }),
        trial("gene-pathways/refused-pathway-records-carry-facts", || {
            pathways().map_err(|e| e.to_string().into())
        }),
    ]
}

fn registry() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/normalization/grch37", |source, recording| {
            if source == "car"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
            {
                changed = true;
                if invalid {
                    replace(body, "/@id", &(json!(false)));
                } else {
                    replace(
                        body,
                        "/genomicAlleles",
                        &(json!("invalid optional container")),
                    );
                }
            }
        })?;
        if !changed {
            return Err("Registry native allele missing".into());
        }
        let leg = fetch.leg(
            &allele_registry::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let mapped = allele_registry::normalize(&leg, "NC_000010.10:g.114758349C>T", None)?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
        {
            return Err("Registry actual allele refusal or optional metadata changed".into());
        }
    }
    Ok(())
}

fn normalized() -> Check {
    for case in ["optional", "identity", "validation"] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/normalization/mutalyzer", |source, recording| {
            if source == "mutalyzer"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
            {
                changed = true;
                match case {
                    "optional" => replace(body, "/version", &(json!({"invalid":true}))),
                    "identity" => {
                        replace(body, "/input_description", &(json!("NM_004333.4:c.1A>T")))
                    }
                    _ => replace(
                        body,
                        "/errors",
                        &(json!([{"code":"ESYNTHETIC","details":"Native validation outcome"}])),
                    ),
                }
            }
        })?;
        if !changed {
            return Err("Mutalyzer native normalization missing".into());
        }
        let leg = fetch.leg(&mutalyzer::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = mutalyzer::normalize(&leg, "NM_004333.4:c.1799T>A")?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected {
            return Err(format!(
                "Mutalyzer {case}: actual identity refusal or native validation changed"
            )
            .into());
        }
    }
    Ok(())
}

fn proteins() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/protein/synthetic-optional", |source, recording| {
            if source == "uniprot"
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value/results")
            {
                changed = true;
                let Some(mut row) = rows.get(0).cloned() else {
                    return;
                };
                replace(
                    &mut row,
                    "/proteinDescription",
                    &(json!("invalid optional container")),
                );
                if case == "identity" {
                    replace(&mut row, "/primaryAccession", &(json!("invalid")));
                }
                *rows = if case == "absent" {
                    json!([])
                } else {
                    json!([row])
                };
            }
        })?;
        if !changed {
            return Err("UniProt native protein records missing".into());
        }
        let leg = fetch.leg(&uniprot::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = uniprot::proteins(&leg, &["P42771".parse()?, "Q8N726".parse()?])?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(format!(
                "UniProt {case}: actual protein refusal or native absence changed"
            )
            .into());
        }
    }
    Ok(())
}

fn pathways() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/pathways", |source, recording| {
            if source == "reactome"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.contains("/UniProt/P04637/"))
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value")
            {
                changed = true;
                let Some(mut row) = rows.get(0).cloned() else {
                    return;
                };
                replace(&mut row, "/displayName", &(json!({"invalid":true})));
                if case == "identity" {
                    replace(&mut row, "/stId", &(json!("invalid")));
                }
                *rows = if case == "absent" {
                    json!([])
                } else {
                    json!([row])
                };
            }
        })?;
        if !changed {
            return Err("Reactome native pathway records missing".into());
        }
        let leg = fetch.leg(&reactome::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = reactome::pathways(&leg, &"P04637".parse()?)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(format!(
                "Reactome {case}: actual pathway refusal or native absence changed"
            )
            .into());
        }
    }
    Ok(())
}
