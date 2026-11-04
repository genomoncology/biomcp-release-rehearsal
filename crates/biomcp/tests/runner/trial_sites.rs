//! Old trial snapshot owner keys must refuse before decoding incompatible payloads.
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial("trial-sites/sites/old-snapshot-owner-expires", || {
        old_owner().map_err(|e| e.to_string().into())
    })
}
fn old_owner() -> Check {
    for version in [1, 2, 3, 4] {
        old_version(version)?;
    }
    Ok(())
}
fn old_version(version: u32) -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("trial/get/NCT03734029", &replay, |_, _| {})?;
    let args = ["get", "trial", "NCT03734029", "locations"].map(str::to_owned);
    let first = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    let next = super::action(&first, "page")?;
    // Reclassify this row as an old owner's incompatible payload. The key lookup
    // must reject it before decode, even though the bytes are intentionally invalid.
    let database = env.root().join("case-cache").join("replay.sqlite");
    let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        cache: Some(database),
        cache_owners: vec![biomcp_query::store::OWNER],
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })?;
    let db = fetch.cache_access()?.connect()?;
    use sha2::{Digest, Sha256};
    let old_key = if version == 4 {
        // This source snapshot predates the posted-results projection, not the source data.
        serde_json::to_vec(&("biomcp.source_snapshot/1", "trial", "NCT03734029"))?
    } else {
        Sha256::digest(format!("biomcp.trial_snapshot/{version}\0NCT03734029").as_bytes()).to_vec()
    };
    db.execute(
        "UPDATE trial_snapshot SET request_key=?1,payload='[\"old positional snapshot\"]'",
        [old_key],
    )?;
    let expired = command(&env, &next, &env.globals(None, true), 3)?;
    if expired.get("status").and_then(serde_json::Value::as_str) != Some("expired")
        || super::action(&expired, "restart").is_err()
    {
        return Err("old snapshot did not expire with restart".into());
    }
    Ok(())
}
