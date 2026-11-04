//! Expand one ZSTD frame through a caller-bounded output sink.
use super::CodecError;
use std::io::Read;

/// Decode exactly one frame under the configured window and the sink's output bound.
pub fn decompress(
    bytes: &[u8],
    window_log: u32,
    mut sink: impl FnMut(&[u8]) -> Result<(), CodecError>,
) -> Result<(), CodecError> {
    let mut decoder = zstd::stream::read::Decoder::with_buffer(bytes)
        .map_err(|_| CodecError::BadReply)?
        .single_frame();
    decoder
        .window_log_max(window_log)
        .map_err(|_| CodecError::BadReply)?;
    let mut buffer = [0u8; 4096];
    loop {
        let count = decoder
            .read(&mut buffer)
            .map_err(|_| CodecError::BadReply)?;
        if count == 0 {
            break;
        }
        sink(buffer.get(..count).ok_or(CodecError::BadReply)?)?;
    }
    if !decoder.finish().is_empty() {
        return Err(CodecError::BadReply);
    }
    Ok(())
}
