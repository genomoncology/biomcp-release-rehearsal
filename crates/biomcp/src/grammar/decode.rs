//! Decode each field once and retain only its grammar-owned identity on failure.
use biomcp_types::{CheckedQuery, QueryInvalid};
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess};
use serde_json::Value;

/// Decode one checked query and retain only fixed guidance and field identities.
pub(super) fn decode_query<T: CheckedQuery>(value: Value) -> Result<T, QueryInvalid> {
    let Value::Object(fields) = value else {
        return Err(QueryInvalid::default());
    };
    let message = T::USAGE_MESSAGE;
    T::from_fields(serde::de::value::MapAccessDeserializer::new(Fields {
        fields: fields.into_iter(),
        pending: None,
        invalid_field: T::invalid_field,
    }))
    .map_err(|cause| {
        if cause.message().is_none() {
            if let Some(message) = message {
                cause.with_message(message)
            } else {
                cause
            }
        } else {
            cause
        }
    })
}
struct Fields {
    fields: serde_json::map::IntoIter,
    pending: Option<(String, Value)>,
    invalid_field: fn(&str) -> QueryInvalid,
}
impl<'de> MapAccess<'de> for Fields {
    type Error = QueryInvalid;
    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let Some((key, value)) = self.fields.next() else {
            return Ok(None);
        };
        self.pending = Some((key.clone(), value));
        seed.deserialize(key.into_deserializer()).map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let (key, value) = self.pending.take().ok_or_else(QueryInvalid::default)?;
        seed.deserialize(value)
            .map_err(|_| (self.invalid_field)(&key))
    }
}
