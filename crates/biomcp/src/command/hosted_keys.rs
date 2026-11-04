//! Hosted key errors name only the declared source, never caller input.
use super::{Encoding, Outcome};
use serde_json::Value;
use std::io;
pub(super) fn outcome(outcome: Outcome, hosted: bool, encoding: Encoding) -> io::Result<Outcome> {
    if hosted {
        let source = outcome
            .value()?
            .get("sources")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|row| {
                row.get("reason").and_then(Value::as_str)
                    == Some(biomcp_types::Reason::NoKey.into())
            })
            .and_then(|row| row.get("source"))
            .and_then(|source| serde_json::from_value(source.clone()).ok());
        if let Some(source) = source {
            return encoding.encode(
                biomcp_types::ErrorEnvelope::from(biomcp_types::UsageError::HostedKey(source)),
                crate::exit::Status::Usage,
            );
        }
    }
    Ok(outcome)
}
