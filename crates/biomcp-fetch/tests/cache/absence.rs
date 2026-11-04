//! One replay cache owns the acquisition and absence regression fixture.
use super::*;
use biomcp_fetch::{Input, Read};

pub(super) fn legacy_json_absence_removes_a_positive() {
    use sha2::{Digest, Sha256};
    let directory = tempfile::tempdir().unwrap();
    let case = biomcp_fetch::ReplayCache::temporary().unwrap();
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    let url = format!("{}/answer", spec.base);
    let key = Sha256::digest(format!("GET\0{url}").as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let records = directory.path().join("pubmed");
    std::fs::create_dir(&records).unwrap();
    let file = records.join(format!("{key}.json"));
    let recording = serde_json::json!({"source":"pubmed","method":"GET","url":url,
        "exchanges":[{"status":200,"fetched":42,"content_type":"application/json","delay_ms":0,
        "retry_after_ms":null,"body":{"kind":"json","value":{"answer":true}}},
        {"status":404,"content_type":"application/json","delay_ms":0,
        "retry_after_ms":null,"body":{"kind":"json","value":{"absent":true}}}]});
    std::fs::write(&file, recording.to_string()).unwrap();
    let replay = replay_case(directory.path(), case.clone());
    assert!(
        get(&replay, spec, &endpoint, &[]).unwrap().answer,
        "known positive replay must seed this case cache"
    );
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        replay_cache: Some(case.clone()),
        ..Default::default()
    })
    .unwrap();
    assert!(
        get(&offline, spec, &endpoint, &[]).unwrap().answer,
        "offline must observe the same seeded case cache"
    );
    let leg = replay
        .leg(spec, replay.deadline(Duration::from_secs(10)))
        .fresh(true);
    let absence = leg
        .call(
            &endpoint.clone().json_absence(),
            Input::Params(&[]),
            Read::<Absence>::json(),
        )
        .unwrap();
    assert!(
        absence.value.absent,
        "legacy JSON404 must retain its checked source absence"
    );
    assert_eq!(
        absence.meta.fetched, None,
        "legacy recording metadata must retain unknown acquisition"
    );
    assert!(
        get(&offline, spec, &endpoint, &[]).is_err(),
        "missing-time JSON absence must evict the prior positive cache entry"
    );
    let unknown_positive = serde_json::json!({"source":"pubmed","method":"GET","url":url,
        "exchanges":[{"status":200,"content_type":"application/json","delay_ms":0,
        "retry_after_ms":null,"body":{"kind":"json","value":{"answer":true}}}]});
    std::fs::write(file, unknown_positive.to_string()).unwrap();
    let replay = replay_case(directory.path(), case);
    let leg = replay.leg(spec, replay.deadline(Duration::from_secs(10)));
    let answer = leg
        .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
        .unwrap();
    assert!(
        answer.value.answer,
        "unknown-time positive must retain its checked source answer"
    );
    assert_eq!(
        answer.meta.fetched, None,
        "legacy recording metadata must retain unknown acquisition"
    );
    assert!(
        get(&offline, spec, &endpoint, &[]).is_err(),
        "unknown positive acquisition must not establish cache freshness"
    );
}

fn replay_case(directory: &Path, case: biomcp_fetch::ReplayCache) -> Fetch {
    Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.into(),
        },
        replay_cache: Some(case),
        ..Default::default()
    })
    .unwrap()
}
