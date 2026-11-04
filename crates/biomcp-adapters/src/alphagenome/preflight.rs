//! Inspect protobuf lengths without allocating before invoking the generated decoder.
use super::{CodecError, Limits};
type Result<T> = std::result::Result<T, CodecError>;
fn codec_error() -> CodecError {
    CodecError::BadReply
}
pub(super) fn varint(bytes: &mut &[u8]) -> Result<u64> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let (byte, rest) = bytes.split_first().ok_or_else(codec_error)?;
        *bytes = rest;
        if shift == 63 && *byte > 1 {
            return Err(codec_error());
        }
        value |= u64::from(byte & 127) << shift;
        if byte & 128 == 0 {
            return Ok(value);
        }
    }
    Err(codec_error())
}
pub(super) fn length<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8]> {
    let len = usize::try_from(varint(bytes)?).map_err(|_| codec_error())?;
    let (head, tail) = bytes.split_at_checked(len).ok_or_else(codec_error)?;
    *bytes = tail;
    Ok(head)
}
pub(super) fn chunk(mut bytes: &[u8]) -> Result<()> {
    let mut seen = 0u8;
    while !bytes.is_empty() {
        let key = varint(&mut bytes)?;
        let bit = match key {
            10 => {
                length(&mut bytes)?;
                1
            }
            16 => {
                if varint(&mut bytes)? > 1 {
                    return Err(codec_error());
                }
                2
            }
            _ => return Err(codec_error()),
        };
        if seen & bit != 0 {
            return Err(codec_error());
        }
        seen |= bit;
    }
    Ok(())
}
pub(super) fn descriptor(mut bytes: &[u8], limits: Limits) -> Result<()> {
    let mut rank = 0usize;
    let mut seen = 0u8;
    while !bytes.is_empty() {
        let key = varint(&mut bytes)?;
        let bit = match key {
            8 => {
                dimension(&mut bytes, &mut rank, limits)?;
                0
            }
            10 => {
                let mut packed = length(&mut bytes)?;
                while !packed.is_empty() {
                    dimension(&mut packed, &mut rank, limits)?;
                }
                0
            }
            16 => {
                if varint(&mut bytes)? > i32::MAX as u64 {
                    return Err(codec_error());
                }
                1
            }
            26 => {
                chunk(length(&mut bytes)?)?;
                2
            }
            32 => {
                varint(&mut bytes)?;
                2
            }
            _ => return Err(codec_error()),
        };
        if seen & bit != 0 {
            return Err(codec_error());
        }
        seen |= bit;
    }
    if seen != 3 {
        return Err(codec_error());
    }
    Ok(())
}
fn dimension(bytes: &mut &[u8], rank: &mut usize, limits: Limits) -> Result<()> {
    *rank = rank.checked_add(1).ok_or_else(codec_error)?;
    if *rank > limits.rank || varint(bytes)? > limits.dimension as u64 {
        return Err(codec_error());
    }
    Ok(())
}
