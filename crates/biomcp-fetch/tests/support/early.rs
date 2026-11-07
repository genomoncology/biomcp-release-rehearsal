//! Requests whose caller may spend its short budget before sending them.
use super::support::{Request, Server};
use std::{
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

/// Wait for a request; a caller that stopped first never sent one.
pub(super) fn next_unless<T>(server: &Server<T>, stopped: impl Fn() -> bool) -> Option<Request<T>> {
    let started = Instant::now();
    loop {
        match server.requests.recv_timeout(Duration::from_millis(10)) {
            Ok(request) => return Some(request),
            Err(RecvTimeoutError::Timeout) if stopped() => return server.requests.try_recv().ok(),
            Err(RecvTimeoutError::Timeout) => assert!(
                started.elapsed() < Duration::from_secs(30),
                "caller neither sent its request nor stopped"
            ),
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    }
}
