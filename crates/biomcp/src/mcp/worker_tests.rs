//! A public request cannot intentionally break a worker invariant.
#[test]
fn panicking_worker_completes_with_internal_error() {
    use super::super::{Event, Server};
    use serde_json::json;
    let (sender, receiver) = std::sync::mpsc::sync_channel(32);
    let mut server = Server {
        sender,
        active: std::collections::BTreeMap::new(),
        legacy: false,
        globals: crate::grammar::Globals::default(),
        root: std::sync::Arc::new(crate::grammar::Root::new(clap::Command::new(
            "worker-fixture",
        ))),
        settings: std::sync::Arc::new(Ok(None)),
        fetch: Err(crate::config::StartupError::Local),
        recovery: None,
    };
    server
        .start_call("panic".into(), json!("panic"), true, None, || {
            panic!("injected worker failure")
        })
        .unwrap();
    let Event::Completed(key, outcome) = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap()
    else {
        panic!("worker did not send its completion")
    };
    let reply = server.complete(&key, outcome).unwrap().unwrap();
    assert_eq!(reply.get("id"), Some(&json!("panic")));
    assert_eq!(reply.pointer("/error/code"), Some(&json!(-32603)));
    assert!(
        server.active.is_empty(),
        "EOF would retain an occupied slot"
    );
}

#[test]
fn serialized_result_character_boundaries() {
    use super::super::protocol::within_cap;
    use serde_json::json;
    // Numeric boundary safety table stays here because a source reply is duplicated
    // in the public tool result, which cannot express every one-character boundary.
    for (characters, allowed) in [(149_999, true), (150_000, false), (150_001, false)] {
        let value = json!({"text":"é".repeat(characters - 11)});
        assert_eq!(
            value.to_string().chars().count(),
            characters,
            "table must hit exact serialized boundary"
        );
        assert_eq!(
            within_cap(&value),
            allowed,
            "Unicode scalar boundary differs"
        );
    }
    let escaped = json!({"text":"\\".repeat(75_000)});
    assert!(
        !within_cap(&escaped),
        "JSON escapes must count toward the cap"
    );
}
