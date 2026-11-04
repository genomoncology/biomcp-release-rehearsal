use super::{Encoding, Outcome};
use biomcp_types::{Envelope, ErrorEnvelope, Status};
use std::io;

pub(super) fn clean(fetch: &biomcp_fetch::Fetch, encoding: Encoding) -> io::Result<Outcome> {
    match biomcp_query::store::clean(fetch, crate::config::BUDGET) {
        Ok(counts) => encoding.encode(
            Envelope::new(Status::Complete, biomcp_types::EmptyQuery {}, Some(counts)),
            crate::exit::Status::Success,
        ),
        Err(_) => encoding.encode(ErrorEnvelope::local(), crate::exit::Status::Local),
    }
}

pub(super) fn health(fetch: &biomcp_fetch::Fetch, encoding: Encoding) -> io::Result<Outcome> {
    match biomcp_query::health::run(fetch, crate::config::BUDGET) {
        Ok(result) => encoding.encode(
            Envelope::new(Status::Complete, biomcp_types::EmptyQuery {}, Some(result)),
            crate::exit::Status::Success,
        ),
        Err(_) => encoding.encode(ErrorEnvelope::local(), crate::exit::Status::Local),
    }
}
