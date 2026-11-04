//! Borrow protobuf fields before allocating provider messages.
use super::preflight;
use biomcp_fetch::BadReply;
#[derive(Debug)]
pub(super) enum Value<'a> {
    Number(u64),
    Bytes(&'a [u8]),
}
impl<'a> Value<'a> {
    pub(super) fn number(self) -> Result<u64, BadReply> {
        match self {
            Self::Number(v) => Ok(v),
            _ => Err(BadReply),
        }
    }
    pub(super) fn bytes(self) -> Result<&'a [u8], BadReply> {
        match self {
            Self::Bytes(v) => Ok(v),
            _ => Err(BadReply),
        }
    }
}
pub(super) fn fields<'a>(
    mut bytes: &'a [u8],
    mut visit: impl FnMut(u64, Value<'a>) -> Result<(), BadReply>,
) -> Result<(), BadReply> {
    while !bytes.is_empty() {
        let key = preflight::varint(&mut bytes).map_err(|_| BadReply)?;
        let value = match key & 7 {
            0 => Value::Number(preflight::varint(&mut bytes).map_err(|_| BadReply)?),
            2 => Value::Bytes(preflight::length(&mut bytes).map_err(|_| BadReply)?),
            _ => return Err(BadReply),
        };
        visit(key >> 3, value)?;
    }
    Ok(())
}
pub(super) fn single(bytes: &[u8]) -> Result<(u64, &[u8]), BadReply> {
    let mut found = None;
    fields(bytes, |tag, value| {
        if found.is_some() {
            return Err(BadReply);
        }
        found = Some((tag, value.bytes()?));
        Ok(())
    })?;
    found.ok_or(BadReply)
}
