//! Native synthetic replies exercise source boundaries without live publisher access.
use super::{families::Check, trial};
use crate::network_stub::Stub;
use crate::value::replace;
use biomcp_adapters::{Refusal, chembl, hotspots, litsense, mychem, ols};
use biomcp_fetch::{Certificates, Fetch, FetchConfig, Network, Secret};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial(
            "cell-line-chembl/refused-cell-records-carry-window-facts",
            || cells().map_err(|e| e.to_string().into()),
        ),
        trial(
            "disease-ols/refused-ontology-candidates-carry-facts",
            || ontology().map_err(|e| e.to_string().into()),
        ),
        trial(
            "variant-hotspots/refused-hotspot-identities-carry-facts",
            || residues().map_err(|e| e.to_string().into()),
        ),
        trial("drug-mychem/refused-drug-hits-carry-facts", || {
            drugs().map_err(|e| e.to_string().into())
        }),
        trial("article-search/refused-sentences-carry-facts", || {
            sentences().map_err(|e| e.to_string().into())
        }),
    ]
}

fn cells() -> Check {
    for invalid in [false, true] {
        let row = json!({"cell_chembl_id":"CHEMBL3307715", "cell_id":if invalid {0}else{1},
            "cellosaurus_id":"CVCL_2119", "cell_description":{"invalid":true}});
        let (stub, fetch) = stubbed(
            "www.ebi.ac.uk",
            json!({"cell_lines":[row],
            "page_meta":{"total_count":2,"offset":0,"limit":1,"next":"/next","previous":null}}),
        )?;
        let leg = fetch.leg(&chembl::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = chembl::cells(&leg, &"CVCL_2119".parse()?, 1, 0)?;
        if page.window.refusals()
            != if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            }
            || page.window.rows.len() != usize::from(!invalid)
            || page.window.received() != 1
            || page.window.next_offset() != Some(1)
            || page.problems.is_empty()
        {
            return Err("ChEMBL cell lost actual refusal or native continuation".into());
        }
        if stub.finish()?.len() != 1 {
            return Err("ChEMBL cell boundary issued unexpected requests".into());
        }
    }
    Ok(())
}

pub(super) fn stubbed(host: &'static str, body: Value) -> Check<(Stub, Fetch)> {
    let body = body.to_string();
    let stub = Stub::for_source(host, "application/json", move |_| Ok(body.clone()))?;
    let fetch = configured(&stub)?;
    Ok((stub, fetch))
}

pub(super) fn configured(stub: &Stub) -> Check<Fetch> {
    Ok(Fetch::open(FetchConfig {
        proxy: Some(Secret::new(format!("http://{}", stub.address))),
        network: Network::new(
            None,
            None,
            "",
            Certificates::from_pem(stub.certificate.as_bytes())?,
        )?,
        ..Default::default()
    })?)
}

fn ontology() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row =
            json!({"obo_id":"MONDO:0000001","ontology_name":"mondo","label":{"invalid":true}});
        if case == "identity" {
            replace(&mut row, "/obo_id", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let (stub, fetch) = stubbed(
            "www.ebi.ac.uk",
            json!({"response":{"numFound":1,"start":0,"docs":rows}}),
        )?;
        let leg = fetch.leg(&ols::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query =
            serde_json::from_value(json!({"text":"synthetic","ontology":"mondo","limit":5}))?;
        let page = ols::search(&leg, &query)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if page.rows.refusals != expected
            || page.rows.value.len() != usize::from(case == "optional")
        {
            return Err(
                format!("OLS {case}: actual concept refusal or optional label changed").into(),
            );
        }
        if stub.finish()?.len() != 1 {
            return Err("OLS boundary issued unexpected requests".into());
        }
    }
    Ok(())
}

fn residues() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row = json!({"hugoSymbol":"BRAF","residue":"V600","transcriptId":{"invalid":true},"tumorCount":12});
        if case == "identity" {
            replace(&mut row, "/hugoSymbol", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let (stub, fetch) = stubbed("www.cancerhotspots.org", rows)?;
        let leg = fetch.leg(&hotspots::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = hotspots::gene(&leg, &"BRAF".parse()?)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(format!(
                "Hotspots {case}: actual identity refusal or optional transcript changed"
            )
            .into());
        }
        if stub.finish()?.len() != 1 {
            return Err("Hotspots boundary issued unexpected requests".into());
        }
    }
    Ok(())
}

fn drugs() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row = json!({"_id":"CHEMBL1","_version":{"invalid":true}});
        if case == "identity" {
            replace(&mut row, "/_id", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let (stub, fetch) = stubbed("mychem.info", json!({"total":1,"hits":rows}))?;
        let leg = fetch.leg(&mychem::SPEC, fetch.deadline(Duration::from_secs(20)));
        let (mapped, received) = mychem::search(&leg, "synthetic", 5, 0)?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected
            || mapped.value.items.len() != usize::from(case == "optional")
            || received != u32::from(case != "absent")
        {
            return Err(format!(
                "MyChem {case}: actual hit refusal or native received count changed"
            )
            .into());
        }
        if stub.finish()?.len() != 1 {
            return Err("MyChem boundary issued unexpected requests".into());
        }
    }
    Ok(())
}

fn sentences() -> Check {
    for case in ["optional", "identity", "absent"] {
        let mut row = json!({"pmid":33123537,"text":{"invalid":true}});
        if case == "identity" {
            replace(&mut row, "/pmid", &(json!(false)));
        }
        let rows = if case == "absent" {
            json!([])
        } else {
            json!([row])
        };
        let (stub, fetch) = stubbed("www.ncbi.nlm.nih.gov", rows)?;
        let leg = fetch.leg(&litsense::SPEC, fetch.deadline(Duration::from_secs(20)));
        let mapped = litsense::search(&leg, "synthetic")?;
        let expected = if case == "identity" {
            Refusal::Refused
        } else {
            Refusal::None
        };
        if mapped.refusals != expected || mapped.value.len() != usize::from(case == "optional") {
            return Err(format!(
                "LitSense {case}: actual sentence refusal or optional text changed"
            )
            .into());
        }
        if stub.finish()?.len() != 1 {
            return Err("LitSense boundary issued unexpected requests".into());
        }
    }
    Ok(())
}
