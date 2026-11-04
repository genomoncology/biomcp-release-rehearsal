//! Independent source requests rendezvous through the selected executable.
use super::super::{launch::Environment, network_stub};
use serde_json::{Value, json};
use std::fs;
mod peer;

pub(super) fn check() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let inputs = env.root().join("replay");
    let peer = peer::Peer::start(&inputs)?;
    let ca = inputs.join("source-ca.pem");
    fs::write(&ca, &peer.certificate)?;
    env.config(&json!({"proxy":format!("http://{}",peer.address),"ca_bundle":ca}))?;
    let args = [
        "get",
        "variant",
        "chr7:g.140753336A>T",
        "--build",
        "GRCh38",
        "hotspots",
        "clinvar",
    ]
    .map(str::to_owned);
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &args).output();
    let events = peer.finish();
    if let Ok(output) = &output {
        fs::write(inputs.join("child-stdout.json"), &output.stdout)?;
        fs::write(inputs.join("child-stderr.txt"), &output.stderr)?;
        fs::write(
            inputs.join("child-exit.json"),
            json!({"exit":output.status.code()}).to_string(),
        )?;
    }
    network_stub::retain_replay_owner(env.shared_root().as_ref())?;
    let output = output?;
    let events = events?;
    let outcome: Value = serde_json::from_slice(&output.stdout)?;
    for section in ["clinvar", "hotspots"] {
        if outcome.pointer(&format!("/result/candidates/0/sections/{section}/state"))
            != Some(&json!("data"))
        {
            let reason =
                outcome.pointer(&format!("/result/candidates/0/sections/{section}/reason"));
            return Err(format!("{section} did not complete: reason={reason:?}; exit={:?}; peer={events:?}; reply={outcome}", output.status.code()).into());
        }
    }
    if !output.status.success()
        || !output.stderr.is_empty()
        || outcome.get("status") != Some(&json!("complete"))
    {
        return Err(format!(
            "independent sections failed: exit={:?}; reply={outcome}; peer={events:?}",
            output.status.code()
        )
        .into());
    }
    let sources = outcome
        .get("sources")
        .and_then(Value::as_array)
        .ok_or("source outcomes")?;
    let order: Vec<_> = sources
        .iter()
        .filter_map(|s| s.get("source").and_then(Value::as_str))
        .collect();
    if order != ["myvariant", "clinvar", "hotspots"] {
        return Err(format!("declared section order changed: {outcome}").into());
    }
    peer::rendezvous(&events)?;
    Ok(())
}
