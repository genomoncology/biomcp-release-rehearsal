use super::{BundleRejection, Keys, Row, RowFormat};
use crate::BadReply;
use std::collections::BTreeSet;
#[derive(Default)]
pub(super) struct ParsedRows {
    pub rows: Vec<Parsed>,
    pub rejections: Vec<BundleRejection>,
}
impl std::ops::Deref for ParsedRows {
    type Target = [Parsed];
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}
impl ParsedRows {
    pub(super) fn reject(&mut self, row: usize, reason: &str) {
        self.rejections.push(BundleRejection {
            row: row as u64,
            reason: reason.into(),
        });
    }
    pub(super) fn admit<R: Row>(&mut self, value: serde_json::Value, row: usize) {
        let Ok(decoded) = serde_json::from_value::<R>(value.clone()) else {
            self.reject(row, "cannot decode row");
            return;
        };
        self.accept(value.to_string(), decoded.keys(), row);
    }
    pub(super) fn accept(&mut self, value: String, keys: Keys, row: usize) {
        if !keys.exact.iter().any(|(_, key)| !key.trim().is_empty())
            && !keys.words.iter().any(|key| super::query_word(key))
        {
            self.reject(row, "no usable query key");
            return;
        }
        self.rows.push(Parsed {
            number: row as u64,
            row: value,
            keys,
        });
    }
}
pub(super) struct Parsed {
    pub number: u64,
    pub row: String,
    pub keys: Keys,
}
pub(super) fn tsv<R: Row>(body: &[u8]) -> Result<ParsedRows, BadReply> {
    let text = std::str::from_utf8(body).map_err(|_| BadReply)?;
    let mut lines = text.lines();
    let header = lines.next().ok_or(BadReply)?;
    let names: Vec<_> = header.split('\t').collect();
    let unique: BTreeSet<_> = names.iter().collect();
    if names
        .iter()
        .any(|name| name.is_empty() || name.contains('"'))
        || unique.len() != names.len()
    {
        return Err(BadReply);
    }
    let mut rows = ParsedRows::default();
    for (index, line) in lines.enumerate() {
        let values = line
            .split('\t')
            .map(tsv_cell)
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != names.len() {
            rows.reject(index + 2, "wrong number of columns");
            continue;
        }
        let mut object = serde_json::Map::new();
        for (name, value) in names.iter().zip(values) {
            if !value.is_empty() {
                object.insert((*name).to_owned(), value.into());
            }
        }
        let value = serde_json::Value::Object(object);
        rows.admit::<R>(value, index + 2);
    }
    Ok(rows)
}

fn tsv_cell(value: &str) -> Result<&str, BadReply> {
    if value.starts_with('"') || value.ends_with('"') {
        value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .filter(|v| !v.contains('"'))
            .ok_or(BadReply)
    } else if value.contains('"') {
        Err(BadReply)
    } else {
        Ok(value)
    }
}

pub(super) fn part_rows<R: Row>(
    body: &[u8],
    format: RowFormat,
    part: &str,
) -> Result<ParsedRows, BadReply> {
    let rows = rows::<R>(body, format)?;
    for row in rows.iter() {
        let decoded: R = serde_json::from_str(&row.row).map_err(|_| BadReply)?;
        decoded.check_part(part)?;
    }
    Ok(rows)
}

pub(super) fn rows<R: Row>(body: &[u8], format: RowFormat) -> Result<ParsedRows, BadReply> {
    match format {
        RowFormat::Document { .. } => {
            let mut parsed = ParsedRows::default();
            for (index, row) in R::decode_document(body)?.into_iter().enumerate() {
                match serde_json::from_str::<R>(&row) {
                    Ok(decoded) => {
                        decoded.check_bundle(0)?;
                        parsed.accept(row, decoded.keys(), index + 2);
                    }
                    Err(_) => parsed.reject(index + 2, "cannot decode row"),
                }
            }
            Ok(parsed)
        }
        RowFormat::Json { rows, metadata } => json::<R>(body, rows, metadata),
        RowFormat::NativeTsv {
            gzip,
            max_decoded_bytes,
        } => super::native::rows::<R>(body, gzip, max_decoded_bytes),
        RowFormat::Tsv => tsv::<R>(body),
        RowFormat::Csv { banner_lines } => csv::<R>(body, banner_lines, false),
        RowFormat::NativeCsv { max_bytes } => {
            if max_bytes == 0
                || max_bytes > super::MAX_BUNDLE_BYTES
                || body.len() as u64 > max_bytes
            {
                return Err(BadReply);
            }
            csv::<R>(body, 0, true)
        }
    }
}
fn csv<R: Row>(body: &[u8], banner_lines: usize, native: bool) -> Result<ParsedRows, BadReply> {
    let mut text = std::str::from_utf8(body).map_err(|_| BadReply)?;
    for _ in 0..banner_lines {
        text = text.split_once('\n').ok_or(BadReply)?.1;
    }
    let mut records = records(text)?.into_iter();
    let names = records.next().ok_or(BadReply)?;
    let unique: BTreeSet<_> = names.iter().collect();
    if names.iter().any(String::is_empty) || unique.len() != names.len() {
        return Err(BadReply);
    }
    let mut rows = ParsedRows::default();
    for (index, values) in records.enumerate() {
        if banner_lines > 0
            && values
                .iter()
                .all(|v| !v.is_empty() && v.bytes().all(|c| c == b'+'))
        {
            continue;
        }
        if values.len() != names.len() {
            rows.reject(index + banner_lines + 2, "wrong number of columns");
            continue;
        }
        let object: serde_json::Map<_, _> = names
            .iter()
            .zip(values)
            .filter(|(_, v)| native || !v.is_empty())
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v)))
            .collect();
        let value = serde_json::Value::Object(object);
        rows.admit::<R>(value, index + banner_lines + 2);
    }
    Ok(rows)
}
// Quoting is a wire boundary: keep the complete logical-record state machine here.
fn records(text: &str) -> Result<Vec<Vec<String>>, BadReply> {
    let mut chars = text.chars().peekable();
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let (mut quoted, mut closed, mut started) = (false, false, false);
    while let Some(c) = chars.next() {
        started = true;
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cell.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                cell.push(c);
            }
            continue;
        }
        match c {
            '"' if cell.is_empty() && !closed => quoted = true,
            ',' => {
                row.push(std::mem::take(&mut cell));
                closed = false;
            }
            '\n' | '\r' => {
                if c == '\r' && chars.next() != Some('\n') {
                    return Err(BadReply);
                }
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
                closed = false;
                started = false;
            }
            '"' => return Err(BadReply),
            _ if closed => return Err(BadReply),
            _ => cell.push(c),
        }
    }
    if quoted {
        return Err(BadReply);
    }
    if started {
        row.push(cell);
        rows.push(row);
    }
    Ok(rows)
}

fn json<R: Row>(body: &[u8], field: &str, metadata: &str) -> Result<ParsedRows, BadReply> {
    let document: serde_json::Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    let meta = document
        .get(metadata)
        .filter(|v| v.is_object())
        .ok_or(BadReply)?;
    let values = document
        .get(field)
        .and_then(serde_json::Value::as_array)
        .filter(|rows| !rows.is_empty())
        .ok_or(BadReply)?;
    let mut ids = BTreeSet::new();
    let mut parsed = ParsedRows::default();
    for (index, value) in values.iter().enumerate() {
        let Some(object) = value.as_object() else {
            parsed.reject(index + 1, "cannot decode row");
            continue;
        };
        let mut object = object.clone();
        if object.insert("_bundle_meta".into(), meta.clone()).is_some() {
            return Err(BadReply);
        }
        let value = serde_json::Value::Object(object);
        let Ok(decoded) = serde_json::from_value::<R>(value.clone()) else {
            parsed.reject(index + 1, "cannot decode row");
            continue;
        };
        decoded.check_bundle(values.len())?;
        let keys = decoded.keys();
        for (kind, id) in &keys.exact {
            if matches!(kind, super::KeyKind::Id) && !ids.insert(id.to_ascii_lowercase()) {
                return Err(BadReply);
            }
        }
        parsed.accept(value.to_string(), keys, index + 1);
    }
    if parsed.is_empty() {
        return Err(BadReply);
    }
    Ok(parsed)
}
