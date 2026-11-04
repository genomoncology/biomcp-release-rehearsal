//! Bounded source response reads shared by ordinary and counted JSON.
use crate::{Leg, request::Failure};
use biomcp_types::Reason;
use std::io::Read;
impl Leg<'_> {
    pub(crate) fn read_body(
        &self,
        response: &mut ureq::http::Response<ureq::Body>,
        status: u16,
        recording: bool,
        limit: u64,
        overflow_reason: Reason,
    ) -> Result<Vec<u8>, Failure> {
        let mut body = Vec::new();
        if recording || (200..300).contains(&status) {
            response
                .body_mut()
                .as_reader()
                .take(limit + 1)
                .read_to_end(&mut body)
                .map_err(|error| {
                    Failure::Local(
                        self.failure(
                            if error.kind() == std::io::ErrorKind::TimedOut
                                || self.budget().expired()
                            {
                                Reason::DeadlineExceeded
                            } else {
                                Reason::BadReply
                            },
                            Some(status),
                        )
                        .into(),
                    )
                })?;
            if body.len() as u64 > limit {
                return Err(Failure::Local(
                    self.failure(overflow_reason, Some(status)).into(),
                ));
            }
        }
        self.report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .downloaded_bytes += body.len() as u64;
        Ok(body)
    }
}
