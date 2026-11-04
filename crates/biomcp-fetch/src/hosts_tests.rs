//! Check publisher origin tables at the private admission boundary.
#[test]
fn declared_origins_reject_wildcards_credentials_paths_and_queries() {
    for origin in [
        "https://*.example",
        "https://api.example/path",
        "https://api.example?x=1",
        "https://user@api.example",
    ] {
        assert!(
            super::secondary(origin).is_err(),
            "unsafe declared origin: {origin}"
        );
    }
    assert_eq!(
        super::secondary("https://cdn.example")
            .expect("exact publisher origin")
            .host_str(),
        Some("cdn.example")
    );
}
