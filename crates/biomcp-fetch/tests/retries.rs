//! Approved retries and deadline distinctions through the public transport API.
#![expect(
    clippy::disallowed_methods,
    reason = "Scoped callers exercise shared public transport state against controlled local HTTP peers"
)]
use biomcp_fetch::{Input, Read};

mod support;
#[path = "support/timing.rs"]
mod timing;
use biomcp_fetch::{Fetch, FetchConfig, KeyUse, Quota, Rate, SourceKey};
use biomcp_types::Reason;
use std::{
    thread,
    time::{Duration, SystemTime},
};
use support::Answer;
use timing::Server;

fn quota() -> &'static Quota {
    Box::leak(Box::new(Quota::new(
        SourceKey::declare("retry"),
        Rate::per_second(1000),
        None,
        1,
    )))
}

#[test]
fn only_approved_get_statuses_retry_and_exhaust_after_three_attempts() {
    for (status, attempts, reason) in [
        (429, 3, Reason::RateLimited),
        (500, 3, Reason::Unavailable),
        (502, 3, Reason::Unavailable),
        (503, 3, Reason::Unavailable),
        (504, 3, Reason::Unavailable),
        (501, 1, Reason::Unavailable),
        (505, 1, Reason::Unavailable),
        (408, 1, Reason::Rejected),
        (400, 1, Reason::Rejected),
        (404, 1, Reason::NotFound),
    ] {
        let server = Server::new();
        let spec = server.spec(quota(), KeyUse::None);
        let fetch = Fetch::open(FetchConfig::default()).unwrap();
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(5)))
                    .call(
                        &spec.get("/answer"),
                        Input::Params(&[]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            for _ in 0..attempts {
                server
                    .next()
                    .respond(status, "Retry-After: 0\r\n", "untrusted provider text");
            }
            let error = worker.join().unwrap().unwrap_err();
            let biomcp_fetch::RequestError::Source(error) = error else {
                panic!("expected provider failure: {error}");
            };
            assert_eq!((error.reason, error.status), (reason, Some(status)));
            assert!(server.requests.try_recv().is_err());
        });
    }
}

#[test]
fn retry_after_seconds_dates_and_malformed_fallback_replace_backoff() {
    let future = httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(2));
    for (value, minimum) in [
        ("1".to_owned(), 950),
        (future, 0),
        ("Sun, 06 Nov 1994 08:49:37 GMT".to_owned(), 0),
        ("bad date".to_owned(), 245),
    ] {
        let server = Server::new();
        let spec = server.spec(quota(), KeyUse::None);
        let fetch = Fetch::open(FetchConfig::default()).unwrap();
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(5)))
                    .call(
                        &spec.get("/answer"),
                        Input::Params(&[]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            let first = server.next();
            let started = first.started;
            first.respond(429, &format!("Retry-After: {value}\r\n"), "");
            let retry = server.next();
            assert!(retry.started.duration_since(started) >= Duration::from_millis(minimum));
            retry.ok();
            assert!(worker.join().unwrap().unwrap().answer);
            assert!(server.requests.try_recv().is_err());
        });
    }
}

#[test]
fn provider_wait_beyond_budget_is_rate_limited_with_typed_duration() {
    for value in [
        "3600".to_owned(),
        httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(3600)),
    ] {
        let server = Server::new();
        let spec = server.spec(quota(), KeyUse::None);
        let fetch = Fetch::open(FetchConfig::default()).unwrap();
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(5)))
                    .call(
                        &spec.get("/answer"),
                        Input::Params(&[]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            server
                .next()
                .respond(503, &format!("Retry-After: {value}\r\n"), "");
            let error = worker.join().unwrap().unwrap_err();
            let biomcp_fetch::RequestError::Source(error) = error else {
                panic!("expected provider failure: {error}");
            };
            assert_eq!(
                (error.reason, error.status),
                (Reason::RateLimited, Some(503))
            );
            assert!(error.retry_after.unwrap() > Duration::from_secs(3500));
            assert!(server.requests.try_recv().is_err());
        });
    }
}

#[test]
fn ordinary_backoff_expiry_is_deadline_exceeded_and_releases_permit() {
    let server = Server::new();
    let spec = server.spec(quota(), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_millis(100)))
                .call(
                    &spec.get("/answer"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        server.next().respond(503, "Retry-After: malformed\r\n", "");
        let error = worker.join().unwrap().unwrap_err();
        let biomcp_fetch::RequestError::Source(error) = error else {
            panic!("expected provider failure: {error}");
        };
        assert_eq!(error.reason, Reason::DeadlineExceeded);
        assert_eq!(error.retry_after, None);
        assert!(server.requests.try_recv().is_err());
        let next = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/next"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        server.next().ok();
        assert!(next.join().unwrap().unwrap().answer);
    });
}

#[test]
fn retry_backoff_releases_in_flight_permit_before_reacquiring() {
    let server = Server::new();
    let spec = server.spec(quota(), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    thread::scope(|scope| {
        let retrying = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/retry"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        server.next().respond(503, "Retry-After: 1\r\n", "");
        let peer = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/peer"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let next = server.next();
        assert!(next.text.contains("/peer"));
        next.ok();
        assert!(peer.join().unwrap().unwrap().answer);
        let retry = server.next();
        assert!(retry.text.contains("/retry"));
        retry.ok();
        assert!(retrying.join().unwrap().unwrap().answer);
    });
}

#[test]
fn body_truncation_decode_and_check_failures_do_not_retry_and_release_permits() {
    for response in [
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{",
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{",
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 16\r\nConnection: close\r\n\r\n{\"answer\":false}",
    ] {
        let server = Server::new();
        let spec = server.spec(quota(), KeyUse::None);
        let fetch = Fetch::open(FetchConfig::default()).unwrap();
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(5)))
                    .call(
                        &spec.get("/bad"),
                        Input::Params(&[]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            server.next().raw(response);
            assert_eq!(
                source(worker.join().unwrap().unwrap_err()).reason,
                Reason::BadReply
            );
            assert!(server.requests.try_recv().is_err());
            let peer = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(5)))
                    .call(
                        &spec.get("/peer"),
                        Input::Params(&[]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            server.next().ok();
            assert!(peer.join().unwrap().unwrap().answer);
        });
    }
}

#[expect(
    clippy::panic,
    reason = "A cache error must never satisfy a provider-failure assertion"
)]
fn source(error: biomcp_fetch::RequestError) -> biomcp_fetch::SourceError {
    let biomcp_fetch::RequestError::Source(error) = error else {
        panic!("expected provider failure: {error}");
    };
    error
}
