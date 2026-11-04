//! Private snapshot encoding; absent legacy facts remain non-refusals.
use super::Refusal;
use serde::{Deserialize, Deserializer, Serializer};

pub(crate) fn none() -> Refusal {
    Refusal::None
}

pub(crate) fn serialize<S: Serializer>(value: &Refusal, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bool(*value == Refusal::Refused)
}

pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Refusal, D::Error> {
    Ok(if bool::deserialize(deserializer)? {
        Refusal::Refused
    } else {
        Refusal::None
    })
}
