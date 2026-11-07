//! Prepare one bounded credential-free body before cache or recording access.
use crate::{Arg, InputError, KeyUse, declaration::Method};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostMetadata {
    pub(crate) request_content_type: String,
    pub(crate) request_body_sha256: String,
}
pub(crate) struct Prepared {
    pub(crate) url: String,
    pub(crate) outgoing: Vec<(String, crate::Secret)>,
    pub(crate) identity: [u8; 32],
    pub(crate) body: Vec<u8>,
    pub(crate) post: Option<PostMetadata>,
    pub(crate) method: Method,
    pub(crate) idempotent: bool,
    pub(crate) identity_encoding: bool,
    pub(crate) double_gzip: bool,
    pub(crate) stall_limit: Option<std::time::Duration>,
    pub(crate) publisher: bool,
    pub(crate) exact_count: bool,
}
impl Prepared {
    pub(crate) fn new(url: String, body: Vec<u8>, method: Method, idempotent: bool) -> Self {
        let post = match method {
            Method::Get | Method::Head => None,
            Method::Form | Method::Json => Some(PostMetadata {
                request_content_type: match method {
                    Method::Form => "application/x-www-form-urlencoded",
                    _ => "application/json",
                }
                .into(),
                request_body_sha256: Sha256::digest(&body)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            }),
        };
        let identity = if let Some(metadata) = &post {
            let mut hash = Sha256::new();
            hash.update(b"POST\0");
            for value in [
                url.as_bytes(),
                metadata.request_content_type.as_bytes(),
                metadata.request_body_sha256.as_bytes(),
            ] {
                hash.update((value.len() as u64).to_be_bytes());
                hash.update(value);
            }
            hash.finalize().into()
        } else if method == Method::Head {
            Sha256::digest([b"HEAD\0".as_slice(), url.as_bytes()].concat()).into()
        } else {
            crate::retry::identity(&url)
        };
        Self {
            outgoing: Vec::new(),
            url,
            identity,
            body,
            post,
            method,
            idempotent,
            identity_encoding: false,
            double_gzip: false,
            stall_limit: None,
            publisher: false,
            exact_count: false,
        }
    }
}
impl Prepared {
    pub(crate) fn recording(&self) -> crate::record::Request<'_> {
        if self.method == Method::Head {
            crate::record::Request::Head
        } else {
            self.post.as_ref().into()
        }
    }
}
struct Bounded {
    bytes: Vec<u8>,
    limit: u64,
    overflow: bool,
}
impl Bounded {
    fn new(limit: u64) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            overflow: false,
        }
    }
    fn encoded(&mut self, value: &str) -> std::io::Result<()> {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'*' | b'-' | b'.' | b'_') {
                self.write_all(&[byte])?;
            } else if byte == b' ' {
                self.write_all(b"+")?;
            } else {
                self.write_all(&[
                    b'%',
                    *HEX.get(usize::from(byte >> 4)).unwrap_or(&b'0'),
                    *HEX.get(usize::from(byte & 15)).unwrap_or(&b'0'),
                ])?;
            }
        }
        Ok(())
    }
}
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() as u64 > self.limit.saturating_sub(self.bytes.len() as u64) {
            self.overflow = true;
            return Err(std::io::ErrorKind::FileTooLarge.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn json<B: Serialize>(body: &B, limit: u64) -> Result<Vec<u8>, InputError> {
    let mut writer = Bounded::new(limit);
    let result = serde_json::to_writer(&mut writer, body);
    if writer.overflow {
        return Err(InputError::BodyLimit);
    }
    result.map_err(|_| InputError::Serialization)?;
    Ok(writer.bytes)
}
pub(crate) fn form(
    params: &[(&str, Arg<'_>)],
    key: KeyUse,
    limit: u64,
) -> Result<Vec<u8>, InputError> {
    let mut writer = Bounded::new(limit);
    let mut first = true;
    for (name, value) in params.iter().filter(|(name, value)| {
        Some(*name) != crate::retry::credential_query(key) && !matches!(value, Arg::Contact(_))
    }) {
        if !first {
            writer.write_all(b"&").map_err(|_| InputError::BodyLimit)?;
        }
        first = false;
        writer
            .encoded(name)
            .and_then(|()| writer.write_all(b"="))
            .map_err(|_| InputError::BodyLimit)?;
        let result = match value {
            Arg::Contact(_) => Ok(()),
            Arg::Text(text) => writer.encoded(text),
            Arg::Num(number) => writer.encoded(&number.to_string()),
            Arg::Ids(ids) => {
                let mut ids: Vec<_> = ids.iter().map(AsRef::as_ref).collect();
                ids.sort_unstable();
                write_ids(&mut writer, &ids)
            }
        };
        result.map_err(|_| InputError::BodyLimit)?;
    }
    Ok(writer.bytes)
}
fn write_ids(writer: &mut Bounded, ids: &[&str]) -> std::io::Result<()> {
    for (index, value) in ids.iter().enumerate() {
        if index > 0 {
            writer.encoded(",")?;
        }
        writer.encoded(value)?;
    }
    Ok(())
}
