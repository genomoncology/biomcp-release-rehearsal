//! Native quoted TSV has required empty cells and an independent decoded bound.
use super::{Row, parse::ParsedRows};
use crate::BadReply;
use std::{collections::BTreeSet, io::Read};
const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn rows<R: Row>(body: &[u8], gzip: bool, limit: u64) -> Result<ParsedRows, BadReply> {
    if limit == 0 || limit > MAX_DECODED_BYTES {
        return Err(BadReply);
    }
    let bytes = if gzip {
        let mut bytes = Vec::new();
        flate2::read::MultiGzDecoder::new(body)
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| BadReply)?;
        bytes
    } else {
        body.to_vec()
    };
    if bytes.len() as u64 > limit {
        return Err(BadReply);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| BadReply)?;
    check_quotes(text)?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(text.as_bytes());
    let names = reader.headers().map_err(|_| BadReply)?.clone();
    let unique: BTreeSet<_> = names.iter().collect();
    if names.is_empty() || names.iter().any(str::is_empty) || unique.len() != names.len() {
        return Err(BadReply);
    }
    let mut parsed = ParsedRows::default();
    for (index, record) in reader.records().enumerate() {
        let Ok(record) = record else {
            parsed.reject(index + 2, "wrong number of columns");
            continue;
        };
        let object: serde_json::Map<_, _> = names
            .iter()
            .zip(record.iter())
            .map(|(key, value)| (key.to_owned(), value.into()))
            .collect();
        let value = serde_json::Value::Object(object);
        let Ok(decoded) = serde_json::from_value::<R>(value.clone()) else {
            parsed.reject(index + 2, "cannot decode row");
            continue;
        };
        decoded.check_bundle(0)?;
        parsed.accept(value.to_string(), decoded.keys(), index + 2);
    }
    Ok(parsed)
}

// The publisher may use literal quotes inside unquoted fields. A quoted field,
// however, must close and may be followed only by its delimiter or record end.
fn check_quotes(text: &str) -> Result<(), BadReply> {
    let (mut start, mut quoted, mut closed) = (true, false, false);
    for byte in text.bytes() {
        if quoted {
            if byte == b'"' {
                quoted = false;
                closed = true;
            }
        } else if closed {
            match byte {
                b'"' => {
                    quoted = true;
                    closed = false;
                }
                b'\t' | b'\r' | b'\n' => {
                    start = true;
                    closed = false;
                }
                _ => return Err(BadReply),
            }
        } else if start && byte == b'"' {
            quoted = true;
            start = false;
        } else {
            start = matches!(byte, b'\t' | b'\r' | b'\n');
        }
    }
    if quoted { Err(BadReply) } else { Ok(()) }
}
