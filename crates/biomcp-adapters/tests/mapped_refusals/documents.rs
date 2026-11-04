//! Document admission and native annotation references expose their real record decisions.
use super::{
    families::{Check, replay},
    synthetic::{configured, stubbed},
    trial,
};
use crate::network_stub::Stub;
use crate::value::replace;
use biomcp_adapters::{Refusal, clinpgx, europepmc, pmc_oa};
use libtest_mimic::Trial;
use serde_json::json;
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "clinpgx-annotations/refused-reference-records-carry-facts",
            || references().map_err(|e| e.to_string().into()),
        ),
        trial(
            "article-preprints/refused-preprint-documents-carry-facts",
            || preprints().map_err(|e| e.to_string().into()),
        ),
        trial(
            "article-fulltext/refused-open-documents-carry-facts",
            || documents().map_err(|e| e.to_string().into()),
        ),
    ]
}

fn references() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row =
            json!({"objCls":"GuidelineAnnotation","id":"PA166134421","name":{"invalid":true}});
        if case == "identity" {
            replace(&mut row, "/id", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let (stub, fetch) = stubbed("api.clinpgx.org", json!({"status":"success","data":rows}))?;
        let leg = fetch.leg(&clinpgx::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(
            json!({"gene":"DPYD","source":"clinpgx","section":"annotations","kind":"guideline"}),
        )?;
        let mapped = clinpgx::references(&leg, &query)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(format!(
                "ClinPGx {case}: actual reference refusal or optional name changed"
            )
            .into());
        }
        if stub.finish()?.len() != 1 {
            return Err("ClinPGx reference made unexpected requests".into());
        }
    }
    Ok(())
}

fn preprints() -> Check {
    for invalid in [false, true] {
        let stub = Stub::for_sources(
            &["www.ebi.ac.uk"],
            |request| {
                if request.contains("fullTextXML") {
                    "application/xml"
                } else {
                    "application/json"
                }
            },
            move |request| {
                if request.contains("fullTextXML") {
                    Ok(concat!(
                        "<article><front><article-meta><title-group><article-title>Synthetic article</article-title>",
                        "</title-group></article-meta></front><body><sec><title>Results</title>",
                        "<p>Synthetic source paragraph.</p></sec></body></article>"
                    ).into())
                } else {
                    Ok(json!({"result":{
                        "id":"PPR1064174","source":"PPR",
                        "license":if invalid{json!(false)}else{json!("CC BY")},
                        "title":{"invalid":true}
                    }})
                    .to_string())
                }
            },
        )?;
        let fetch = configured(&stub)?;
        let leg = fetch.leg(&europepmc::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = europepmc::read(&leg, &"PPR1064174".parse()?)?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || mapped.value.is_some() == invalid
        {
            return Err(
                "Europe PMC actual document refusal or retained optional title changed".into(),
            );
        }
        if stub.finish()?.len() != if invalid { 1 } else { 2 } {
            return Err("Europe PMC document boundary made unexpected requests".into());
        }
    }
    Ok(())
}

fn documents() -> Check {
    for invalid in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("article/fulltext/live-all", |source, recording| {
            if source == "pmc-oa"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && body.get("license_code").is_some()
            {
                changed = true;
                if invalid {
                    replace(body, "/license_code", &(json!(false)));
                } else {
                    replace(body, "/is_manuscript", &(json!({"invalid":true})));
                }
            }
        })?;
        if !changed {
            return Err("PMC OA native document record missing".into());
        }
        let leg = fetch.leg(&pmc_oa::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = pmc_oa::read(
            &leg,
            "PMC7573235".parse()?,
            Some(biodata::Pmid::new(33123537)?),
        )?;
        if mapped.refusals
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || mapped.value.is_some() == invalid
        {
            return Err(
                "PMC OA actual document refusal or retained optional manuscript flag changed"
                    .into(),
            );
        }
    }
    Ok(())
}
