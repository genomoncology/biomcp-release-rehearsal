//! Native page owners retain traversal when an examined record is refused.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, cbioportal, cellosaurus, monarch, seer};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "cell-line-identity/refused-candidates-carry-page-facts",
            || cells().map_err(|e| e.to_string().into()),
        ),
        trial("phenotype-identity/refused-terms-carry-page-facts", || {
            terms().map_err(|e| e.to_string().into())
        }),
        trial("disease-seer/refusals-follow-selected-source-rows", || {
            statistics().map_err(|e| e.to_string().into())
        }),
        trial(
            "variant-recurrence/refused-mutations-carry-window-facts",
            || mutations().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn cells() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("cell-line/examples", |source, recording| {
            if source == "cellosaurus"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.contains("/search/"))
                && let Some(row) =
                    recording.pointer_mut("/exchanges/0/body/value/Cellosaurus/cell-line-list/0")
            {
                changed = true;
                replace(row, "/name-list", &(json!("invalid optional container")));
                if invalid {
                    replace(row, "/accession-list/0/value", &(json!("invalid")));
                }
            }
        })?;
        if !changed {
            return Err("Cellosaurus native search row missing".into());
        }
        let leg = fetch.leg(&cellosaurus::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({"query":"MOLM13","limit":5}))?;
        let page = cellosaurus::search(&leg, &query)?;
        if page.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.received != 1
            || page.items.len() != usize::from(!invalid)
            || page.problems.is_empty()
        {
            return Err(
                "Cellosaurus lost actual identity refusal or retained field problem".into(),
            );
        }
    }
    Ok(())
}

fn terms() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("phenotype/synthetic", |source, recording| {
            if source == "monarch"
                && let Some(row) = recording.pointer_mut("/exchanges/0/body/value/items/0")
            {
                changed = true;
                replace(row, "/name", &(json!({"invalid":true})));
                if invalid {
                    replace(row, "/id", &(json!("invalid")));
                }
            }
        })?;
        if !changed {
            return Err("Monarch native search row missing".into());
        }
        let leg = fetch.leg(&monarch::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({"text":"synthetic","limit":5}))?;
        let page = monarch::phenotype::search(&leg, &query)?;
        if page.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.received != 1
            || page.items.len() != usize::from(!invalid)
            || page.problems.is_empty()
        {
            return Err("Monarch lost actual HP refusal or retained field problem".into());
        }
    }
    Ok(())
}

fn statistics() -> Check {
    catalog()?;
    series()
}

fn catalog() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("disease/seer-synthetic", |source, recording| {
            if source == "seer"
                && let Some(sites) = recording.pointer_mut("/exchanges/0/body/value/CancerSites")
            {
                changed = true;
                replace(sites, "/0/active", &(json!("invalid optional field")));
                if invalid {
                    replace(sites, "/0/value", &(json!("invalid")));
                }
            }
        })?;
        if !changed {
            return Err("SEER native catalog rows missing".into());
        }
        let leg = fetch.leg(&seer::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = seer::catalog(&leg, 1, 0)?;
        let retained = seer::catalog(&leg, 1, 1)?;
        if page.window.refusals()
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.window.rows.len() != usize::from(!invalid)
            || page.window.next_offset() != Some(1)
            || retained.window.refusals() != Refusal::None
        {
            return Err("SEER catalog refusal escaped its selected native window".into());
        }
    }
    Ok(())
}

fn series() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("disease/seer-synthetic", |source, recording| {
            if source == "seer"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                let Ok(mut document) = serde_json::from_str::<Value>(native) else {
                    return;
                };
                if let Some(row) = document.pointer_mut("/data/5_3_1_1_55/data_series/0") {
                    changed = true;
                    if let Some(slot) = row.pointer_mut("/1") {
                        *slot = json!("invalid optional rate");
                    };
                    if invalid && let Some(slot) = row.pointer_mut("/0") {
                        *slot = json!("invalid year");
                    }
                    *body = json!(document.to_string());
                }
            }
        })?;
        if !changed {
            return Err("SEER native statistic row missing".into());
        }
        let leg = fetch.leg(&seer::SPEC, fetch.deadline(Duration::from_secs(20)));
        let selection = seer::Selection {
            site: "55".parse()?,
            sex: "3".parse()?,
            race: "1".parse()?,
            age_range: "1".parse()?,
            dataset: "4".parse()?,
            statistic: seer::Statistic::Survival {
                interval: "5".parse()?,
            },
        };
        let page = seer::statistics(&leg, &selection, 1, 0)?;
        let retained = seer::statistics(&leg, &selection, 1, 1)?;
        if page.window.refusals()
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.window.rows.len() != usize::from(!invalid)
            || page.window.next_offset() != Some(1)
            || retained.window.refusals() != Refusal::None
        {
            return Err("SEER statistic refusal escaped its selected native window".into());
        }
    }
    Ok(())
}

fn mutations() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("variant/recurrence", |source, recording| {
            if source == "cbioportal"
                && recording
                    .get("url")
                    .and_then(|value| value.as_str())
                    .is_some_and(|u| u.contains("/mutations?"))
                && let Some(rows) = recording.pointer_mut("/exchanges/0/body/value")
            {
                changed = true;
                let Some(mut row) = rows.get(0).cloned() else {
                    return;
                };
                replace(&mut row, "/chr", &(json!({"invalid":true})));
                if invalid {
                    replace(&mut row, "/sampleId", &(json!(" ")));
                }
                *rows = json!([row]);
            }
        })?;
        if !changed {
            return Err("cBioPortal native mutation row missing".into());
        }
        let leg = fetch.leg(&cbioportal::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({
            "input":"rs113488022","source":"cbioportal","gene":"BRAF",
            "study":"msk_impact_2017","profile":"msk_impact_2017_mutations",
            "sample_set":"msk_impact_2017_all","limit":3
        }))?;
        let page = cbioportal::mutations(&leg, &query, biodata::NcbiGeneId::new(673)?)?;
        if page.window.refusals()
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.window.rows.len() != usize::from(!invalid)
            || page.window.received() != 1
        {
            return Err("cBioPortal lost actual mutation refusal or native received count".into());
        }
    }
    Ok(())
}
