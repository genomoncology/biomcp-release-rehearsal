//! JSON reader bindings for the source-owned body registry.
use biomcp_types::{Envelope, ResponseBody};
use serde::de::DeserializeOwned;

type Reader = fn(&[u8]) -> Result<Vec<u8>, serde_json::Error>;
fn read_write<T>(bytes: &[u8]) -> Result<Vec<u8>, serde_json::Error>
where
    T: ResponseBody + DeserializeOwned,
    T::Query: DeserializeOwned,
{
    let reply: Envelope<T> = serde_json::from_slice(bytes)?;
    serde_json::to_vec(&reply)
}
macro_rules! readers {
    ($($method:ident: $body:ty => $name:literal),+ $(,)?) => {
        &[$((<$body as ResponseBody>::SCHEMA, read_write::<$body> as Reader)),+]
    };
}
/// JSON readers indexed by the published reply schema name.
pub const READERS: &[(&str, Reader)] = biomcp_types::reply_types!(readers);
