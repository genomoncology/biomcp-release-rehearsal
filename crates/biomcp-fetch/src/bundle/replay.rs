//! Recorded bundle transfers obey the same bounded status retry policy as live transfers.
use super::{Bundle, Row};
use crate::{Leg, RecordError, RequestError, record::Exchange, retry};
use biomcp_types::Reason;
use std::time::Duration;

pub(super) fn exchange<R: Row>(
    leg: &Leg<'_>,
    bundle: &Bundle<R>,
    key: &[u8; 32],
    url: &str,
) -> Result<Exchange, RequestError> {
    for attempt in 0..retry::ATTEMPTS {
        let permit = leg
            .acquire_quota()
            .map_err(|delay| leg.quota_failure(delay))?
            .ok_or_else(|| leg.failure(Reason::Syncing, None))?;
        let exchange = leg
            .fetch
            .recordings
            .replay_bundle(leg.spec.source, key, url)
            .map_err(RequestError::Recording)?
            .ok_or(RequestError::Recording(RecordError::Missing))?;
        let delay = Duration::from_millis(exchange.delay_ms);
        if !leg.budget().advance(delay) {
            return Err(leg.failure(Reason::DeadlineExceeded, None).into());
        }
        leg.observe_bundle(
            (key, url),
            Some(exchange.status),
            exchange.body.bytes(),
            bundle.format,
            delay,
        );
        if exchange.status == 200 {
            return Ok(exchange);
        }
        drop(permit);
        retry(
            leg,
            key,
            exchange.status,
            exchange.retry_after_ms.map(Duration::from_millis),
            attempt,
        )?;
    }
    Err(leg.failure(Reason::Unavailable, None).into())
}

// Both transports preserve the provider's failure and delay without waiting past the budget.
pub(super) fn retry(
    leg: &Leg<'_>,
    key: &[u8; 32],
    status: u16,
    delay: Option<Duration>,
    attempt: u32,
) -> Result<(), RequestError> {
    let mut error = leg.failure(crate::retry::status_reason(status), Some(status));
    error.retry_after = delay;
    if !crate::retry::retry_status(status)
        || attempt + 1 == crate::retry::ATTEMPTS
        || delay.is_some_and(|wait| wait >= leg.budget().remaining())
    {
        return Err(error.into());
    }
    if !crate::retry::wait(
        delay.unwrap_or_else(|| crate::retry::backoff(key, attempt)),
        leg.budget(),
    ) {
        if status != 429 {
            error.reason = Reason::Syncing;
        }
        return Err(error.into());
    }
    Ok(())
}
