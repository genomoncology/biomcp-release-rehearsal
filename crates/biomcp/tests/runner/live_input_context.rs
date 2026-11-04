//! Prove diagnostic request equality through the public checked fetch boundary.
use super::super::super::{launch::Environment, network_stub::Stub};
use biomcp_fetch::{Arg, BadReply, Certificates, Fetch, FetchConfig, Input, Network, Read, Reply};
use std::{fs, sync::Arc, time::Duration};

#[derive(serde::Deserialize)]
struct Checked {
    answer: bool,
}
impl Reply for Checked {
    fn check(&self) -> Result<(), BadReply> {
        if self.answer { Ok(()) } else { Err(BadReply) }
    }
    fn input_parameter() -> Option<(&'static str, &'static str)> {
        Some(("pmid", "id"))
    }
}

pub(super) fn prove() -> Result<(), Box<dyn std::error::Error>> {
    let environment = Environment::new()?;
    let stub = Stub::for_sources(
        &["eutils.ncbi.nlm.nih.gov"],
        |_| "application/json",
        |_| Ok(r#"{"answer":true}"#.into()),
    )?;
    stub.retain(environment.shared_root());
    let path = environment.root().join("input-context.jsonl");
    let fetch = Fetch::open(FetchConfig {
        observations: Some(Arc::new(biomcp_fetch::Observations::create(&path)?)),
        network: Network::new(
            None,
            Some(biomcp_fetch::Secret::new(format!(
                "http://{}",
                stub.address
            ))),
            "",
            Certificates::from_pem(stub.certificate.as_bytes())?,
        )?,
        ..FetchConfig::default()
    })?;
    let spec = biomcp_query::sources::requests().find(|spec| spec.source == biomcp_types::Source::PubMed).ok_or("missing source declaration")?;
    for post in [false, true] {
        for (id, mode) in [("1", "fixed"), ("2", "fixed"), ("2", "changed")] {
            let endpoint = if post {
                spec.post_form("/input-context")
            } else {
                spec.get("/input-context")
            };
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(15)))
                .call(
                    &endpoint,
                    Input::Params(&[("id", Arg::Text(id)), ("mode", Arg::Text(mode))]),
                    Read::<Checked>::json(),
                )?;
        }
    }
    let rows = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str::<biomcp_fetch::Observation>)
        .collect::<Result<Vec<_>, _>>()?;
    let contexts = rows
        .iter()
        .map(|row| row.provenance.input_context.as_deref())
        .collect::<Vec<_>>();
    let [get, get_ids, get_changed, post, post_ids, post_changed] = contexts.as_slice() else {
        return Err("missing request-context observations".into());
    };
    if get.is_none()
        || post.is_none()
        || get != get_ids
        || post != post_ids
        || get == get_changed
        || post == post_changed
        || get == post
    {
        return Err("request context ignored method/non-ID inputs or retained ID values".into());
    }
    Ok(())
}
