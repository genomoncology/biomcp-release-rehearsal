//! Native wrappers expose actual source record decisions independently of retained properties.
use super::{
    families::{Check, replay},
    trial,
};
use crate::value::replace;
use biomcp_adapters::{Refusal, gwas, orcid, semantic_scholar, variantvalidator};
use libtest_mimic::Trial;
use serde_json::json;
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("author-search/refused-authors-carry-page-facts", || {
            authors().map_err(|e| e.to_string().into())
        }),
        trial(
            "variant-normalization/refused-validator-records-carry-facts",
            || variants().map_err(|e| e.to_string().into()),
        ),
        trial(
            "author-orcid/refused-orcid-work-records-carry-facts",
            || works().map_err(|e| e.to_string().into()),
        ),
        trial(
            "variant-population/refused-gwas-records-carry-page-facts",
            || associations().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn authors() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row = json!({"authorId":"12345","name":{"invalid":true}});
        if case == "identity" {
            replace(&mut row, "/authorId", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let page: semantic_scholar::author::Page =
            serde_json::from_value(json!({"offset":0,"total":2,"next":1,"data":rows}))?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if page.refusals() != expected
            || page.len() != usize::from(case == "optional")
            || page.next() != Some(1)
        {
            return Err(format!(
                "Semantic Scholar {case}: actual author refusal or continuation changed"
            )
            .into());
        }
        let value = serde_json::to_value(&page)?;
        if value.get("refusals").is_some() {
            return Err("Semantic Scholar exposed its private refusal fact".into());
        }
    }
    Ok(())
}

fn variants() -> Check {
    for case in ["optional", "identity", "validation"] {
        let mut changed = false;
        let (_root, fetch) = replay(
            "variant/normalization/variantvalidator",
            |source, recording| {
                if source == "variantvalidator"
                    && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                {
                    changed = true;
                    match case {
                        "optional" => {
                            replace(body, "/z/validation_warnings", &(json!({"invalid":true})))
                        }
                        "identity" => {
                            replace(body, "/z/submitted_variant", &(json!("NM_004333.4:c.1A>T")))
                        }
                        _ => replace(body, "/flag", &(json!("warning"))),
                    }
                }
            },
        )?;
        if !changed {
            return Err("VariantValidator native records missing".into());
        }
        let leg = fetch.leg(
            &variantvalidator::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let mapped =
            variantvalidator::resolve(&leg, "NM_004333.4:c.1799T>A", biodata::Build::GRCh38)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || mapped.rejected != (case == "validation")
            || mapped.candidates.len()
                != if case == "optional" {
                    2
                } else if case == "identity" {
                    1
                } else {
                    0
                }
        {
            return Err(format!(
                "VariantValidator {case}: actual transcript refusal or validation outcome changed"
            )
            .into());
        }
    }
    Ok(())
}

fn works() -> Check {
    for case in ["optional", "identity", "invalid-container", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("orcid/translated-title", |source, recording| {
            if source == "orcid"
                && let Some(groups) = recording.pointer_mut("/exchanges/0/body/value/group")
            {
                changed = true;
                match case {
                    "optional" => replace(
                        groups,
                        "/0/work-summary/0/journal-title",
                        &(json!({"invalid":true})),
                    ),
                    "identity" => replace(groups, "/0/work-summary/0/put-code", &(json!(0))),
                    "invalid-container" => replace(
                        groups,
                        "/0/work-summary",
                        &(json!("invalid optional container")),
                    ),
                    _ => *groups = json!([]),
                }
            }
        })?;
        if !changed {
            return Err("ORCID native groups missing".into());
        }
        let leg = fetch.leg(&orcid::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = orcid::works(&leg, &"0000-0002-4299-3018".parse()?)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || mapped.value.len() != usize::from(matches!(case, "optional" | "identity"))
            || mapped.value.first().is_some_and(|group| {
                group.assertions.len() != if case == "identity" { 1 } else { 2 }
            })
        {
            return Err(
                format!("ORCID {case}: actual work refusal or optional container changed").into(),
            );
        }
    }
    Ok(())
}

fn associations() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut changed = false;
        let (_root, fetch) = replay("variant-population/gwas-positive", |source, recording| {
            if source == "gwas"
                && let Some(rows) =
                    recording.pointer_mut("/exchanges/0/body/value/_embedded/associations")
            {
                changed = true;
                let Some(mut row) = rows.get(0).cloned() else {
                    return;
                };
                replace(&mut row, "/p_value", &(json!({"invalid":true})));
                if case == "identity" {
                    replace(&mut row, "/association_id", &(json!(false)));
                }
                *rows = if case == "absent" {
                    json!([])
                } else {
                    json!([row])
                };
                let total = u32::from(case != "absent");
                replace(
                    recording,
                    "/exchanges/0/body/value/page/totalElements",
                    &(json!(total)),
                );
                replace(
                    recording,
                    "/exchanges/0/body/value/page/totalPages",
                    &(json!(total)),
                );
            }
        })?;
        if !changed {
            return Err("GWAS native association rows missing".into());
        }
        let leg = fetch.leg(&gwas::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = gwas::associations(&leg, &"rs113488022".parse()?, 0, 3, 0)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if page.refusals != expected
            || page.rows.len() != usize::from(case == "optional")
            || page.received != u32::from(case != "absent")
        {
            return Err(format!(
                "GWAS {case}: actual association refusal or native received count changed"
            )
            .into());
        }
    }
    Ok(())
}
