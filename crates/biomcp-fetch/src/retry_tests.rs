use super::*;
use crate::{Credential, SecretName};
fn identity(url: &str, params: &[(&str, Arg<'_>)], key: KeyUse) -> [u8; 32] {
    super::identity(&canonical(url, params, key))
}

#[test]
fn only_connect_timeout_and_reset_are_retryable_transport_errors() {
    for (error, expected) in [
        (ureq::Error::Timeout(ureq::Timeout::Connect), true),
        (ureq::Error::Timeout(ureq::Timeout::Global), false),
        (ureq::Error::Timeout(ureq::Timeout::RecvResponse), false),
        (
            ureq::Error::Io(std::io::ErrorKind::ConnectionReset.into()),
            true,
        ),
        (ureq::Error::Io(std::io::ErrorKind::TimedOut.into()), false),
        (
            ureq::Error::Io(std::io::ErrorKind::UnexpectedEof.into()),
            false,
        ),
        (ureq::Error::ConnectionFailed, false),
    ] {
        assert_eq!(retry_error(&error), expected);
    }
}

#[test]
fn internal_identity_excludes_credential_query_and_preserves_request_values() {
    let credential = KeyUse::Optional(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NcbiApiKey,
        place: KeyPlace::Query("api_key"),
    });
    let first = identity(
        "https://example.test/answer",
        &[
            ("q", Arg::Text("a b")),
            ("api_key", Arg::Text("first-placeholder")),
        ],
        credential,
    );
    let second = identity(
        "https://example.test/answer",
        &[
            ("q", Arg::Text("a b")),
            ("api_key", Arg::Text("second-placeholder")),
        ],
        credential,
    );
    assert_eq!(first, second);
    assert_ne!(
        first,
        identity(
            "https://example.test/answer",
            &[("q", Arg::Text("a+b"))],
            credential
        )
    );
    assert_eq!(backoff(&first, 1) - backoff(&first, 0), BASE_BACKOFF);
}

#[test]
fn retry_after_boundaries_accept_only_seconds_and_http_dates() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(784111777);
    for (value, expected) in [
        (" 0 ", Some(Duration::ZERO)),
        ("18446744073709551615", Some(Duration::from_secs(u64::MAX))),
        ("18446744073709551616", None),
        ("+1", None),
        ("-1", None),
        ("1.5", None),
        ("", None),
        ("Sun, 06 Nov 1994 08:49:37 GMT", Some(Duration::ZERO)),
        (
            "Sunday, 06-Nov-94 08:49:38 GMT",
            Some(Duration::from_secs(1)),
        ),
        ("Sun Nov  6 08:49:39 1994", Some(Duration::from_secs(2))),
    ] {
        assert_eq!(retry_after(value, now), expected, "{value}");
    }
}
