//! HTTP syntax belongs to httparse; reads have size and absolute time bounds.
use std::{
    collections::BTreeSet,
    io::{self, Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
pub(super) const MAX_HEAD: usize = 16 * 1024;
pub(super) const MAX_BODY: usize = super::super::framing::MAX_FRAME_BYTES;
pub(super) const READ_BUDGET: Duration = Duration::from_secs(2);
pub(super) struct Request {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) headers: Vec<(String, String)>,
    pub(super) body: Vec<u8>,
}
impl Request {
    pub(super) fn declared_use(&self) -> io::Result<crate::command::DeclaredUse> {
        crate::command::DeclaredUse::parse(self.one("x-biomcp-use")?).ok_or_else(malformed)
    }
    /// An empty value leaves nothing out; every listed name must be a known source.
    pub(super) fn excluded_sources(&self) -> io::Result<BTreeSet<biomcp_types::Source>> {
        let value = self
            .one(crate::command::EXCLUDE_HEADER)?
            .unwrap_or_default();
        if value.trim().is_empty() {
            return Ok(BTreeSet::new());
        }
        value
            .split(',')
            .map(|name| biomcp_query::sources::switchable_source(name.trim()).ok_or_else(malformed))
            .collect()
    }
    pub(super) fn source_keys(&self) -> io::Result<biomcp_fetch::Secrets> {
        let mut secrets = biomcp_fetch::Secrets::default();
        for name in biomcp_query::sources::keys() {
            if let Some(value) = self.one(&name.header())? {
                if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
                    return Err(malformed());
                }
                secrets.insert(name, biomcp_fetch::Secret::new(value));
            }
        }
        Ok(secrets)
    }
    pub(super) fn one(&self, name: &str) -> io::Result<Option<&str>> {
        let mut values = self
            .headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case(name));
        let first = values.next().map(|(_, value)| value.as_str());
        if values.next().is_some() {
            return Err(malformed());
        }
        Ok(first)
    }
}
pub(super) fn read(stream: &mut TcpStream) -> io::Result<Request> {
    let end = Instant::now() + READ_BUDGET;
    let (mut request, bytes, offset) = head(stream, end)?;
    let mut buffer = [0; 4096];
    let encoding = request.one("transfer-encoding").map_err(|_| malformed())?;
    let length = request.one("content-length").map_err(|_| malformed())?;
    if let Some(encoding) = encoding {
        if length.is_some() || !encoding.eq_ignore_ascii_case("chunked") {
            return Err(malformed());
        }
        request.body = super::chunked::read(
            stream,
            bytes.get(offset..).ok_or_else(malformed)?.to_vec(),
            end,
        )?;
        return Ok(request);
    }
    let length = length
        .map(|n| {
            if n.bytes().all(|c| c.is_ascii_digit()) {
                n.parse::<usize>().map_err(|_| malformed())
            } else {
                Err(malformed())
            }
        })
        .transpose()?
        .unwrap_or(0);
    if length > MAX_BODY {
        return Err(malformed());
    }
    request.body = bytes.get(offset..).ok_or_else(malformed)?.to_vec();
    if request.body.len() > length {
        return Err(malformed());
    }
    while request.body.len() < length {
        timeout(stream, end)?;
        let available = buffer.len().min(length - request.body.len());
        let n = stream.read(buffer.get_mut(..available).ok_or_else(malformed)?)?;
        if n == 0 {
            return Err(malformed());
        }
        request
            .body
            .extend_from_slice(buffer.get(..n).ok_or_else(malformed)?);
    }
    Ok(request)
}
fn head(stream: &mut TcpStream, end: Instant) -> io::Result<(Request, Vec<u8>, usize)> {
    let mut bytes = Vec::with_capacity(MAX_HEAD);
    let mut buffer = [0; 4096];
    let (request, offset) = loop {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut parsed = httparse::Request::new(&mut headers);
        match parsed.parse(&bytes).map_err(|_| malformed())? {
            httparse::Status::Complete(offset) => {
                if offset > MAX_HEAD || parsed.version != Some(1) {
                    return Err(malformed());
                }
                let request = Request {
                    method: parsed.method.ok_or_else(malformed)?.into(),
                    path: parsed.path.ok_or_else(malformed)?.into(),
                    headers: parsed
                        .headers
                        .iter()
                        .map(|header| {
                            Ok((
                                header.name.into(),
                                std::str::from_utf8(header.value)
                                    .map_err(|_| malformed())?
                                    .to_owned(),
                            ))
                        })
                        .collect::<io::Result<_>>()?,
                    body: Vec::new(),
                };
                break (request, offset);
            }
            httparse::Status::Partial => {
                if bytes.len() >= MAX_HEAD {
                    return Err(malformed());
                }
                timeout(stream, end)?;
                let available = buffer.len().min(MAX_HEAD - bytes.len());
                let n = stream.read(buffer.get_mut(..available).ok_or_else(malformed)?)?;
                if n == 0 {
                    return Err(malformed());
                }
                bytes.extend_from_slice(buffer.get(..n).ok_or_else(malformed)?);
            }
        }
    };
    Ok((request, bytes, offset))
}

fn timeout(stream: &TcpStream, end: Instant) -> io::Result<()> {
    let left = end
        .checked_duration_since(Instant::now())
        .filter(|v| !v.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "HTTP read budget"))?;
    stream.set_read_timeout(Some(left))
}
pub(super) fn malformed() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Invalid HTTP request")
}
// Send the fixed refusal together before closing unread caller bytes.
pub(super) fn unavailable(stream: &mut TcpStream) -> io::Result<()> {
    const BODY: &str = r#"{"error":"Service unavailable"}"#;
    let response = format!(
        "HTTP/1.1 503 Response\r\nContent-Type: application/json\r\nConnection: close\r\n\
         Cache-Control: no-store\r\nContent-Length: {}\r\n\r\n{BODY}",
        BODY.len()
    );
    stream.set_nonblocking(true)?;
    stream.write_all(response.as_bytes())
}
pub(super) fn write(
    stream: &mut TcpStream,
    status: u16,
    body: &[u8],
    revision: Option<&str>,
    allow: Option<&str>,
) -> io::Result<()> {
    let end = Instant::now() + READ_BUDGET;
    let revision = revision
        .map(|revision| format!("MCP-Protocol-Version: {revision}\r\n"))
        .unwrap_or_default();
    let allow = allow
        .map(|method| format!("Allow: {method}\r\n"))
        .unwrap_or_default();
    let head = format!(
        "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nConnection: \
            close\r\nCache-Control: no-store\r\n{allow}{revision}Content-Length: {}\r\n\r\n",
        body.len()
    );
    write_part(stream, head.as_bytes(), end)?;
    write_part(stream, body, end)
}

pub(super) fn read_more(
    stream: &mut TcpStream,
    bytes: &mut Vec<u8>,
    cap: usize,
    end: Instant,
) -> io::Result<()> {
    if bytes.len() >= cap {
        return Err(malformed());
    }
    timeout(stream, end)?;
    let mut buffer = [0; 4096];
    let available = buffer.len().min(cap - bytes.len());
    let read = stream.read(buffer.get_mut(..available).ok_or_else(malformed)?)?;
    if read == 0 {
        return Err(malformed());
    }
    bytes.extend_from_slice(buffer.get(..read).ok_or_else(malformed)?);
    Ok(())
}

fn write_part(stream: &mut TcpStream, mut bytes: &[u8], end: Instant) -> io::Result<()> {
    while !bytes.is_empty() {
        let left = end
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "HTTP write budget"))?;
        stream.set_write_timeout(Some(left))?;
        let written = stream.write(bytes)?;
        if written == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero));
        }
        bytes = bytes.get(written..).ok_or_else(malformed)?;
    }
    Ok(())
}
