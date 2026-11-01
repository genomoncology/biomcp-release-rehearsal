//! A local CONNECT peer terminates TLS with a synthetic private trust anchor.
use crate::fixture_evidence as evidence;

/// Create the shared retained evidence directory.
pub fn evidence_directory() -> std::io::Result<std::path::PathBuf> {
    evidence::directory()
}

/// Retain the replay inputs before their temporary owner is dropped.
pub fn retain_replay_owner(owner: &tempfile::TempDir) -> std::io::Result<()> {
    evidence::retain_replay_owner(owner)
}

use rustls::{ServerConfig, ServerConnection, StreamOwned, pki_types::PrivatePkcs8KeyDer};
use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, mpsc},
    thread,
};

type Requests = Vec<(String, String)>;

#[derive(Debug)]
/// A local CONNECT peer with owned worker and retained failure evidence.
pub struct Stub {
    /// The ephemeral local proxy address.
    pub address: std::net::SocketAddr,
    /// The synthetic PEM trust anchor.
    pub certificate: String,
    stop: mpsc::Sender<()>,
    handshakes: mpsc::Receiver<rustls::HandshakeKind>,
    evidence: Arc<evidence::Evidence>,
    worker: Option<thread::JoinHandle<Result<Requests, String>>>,
}
impl Stub {
    /// Attach an isolated environment to failure evidence.
    pub fn retain(&self, root: Arc<tempfile::TempDir>) {
        self.evidence.attach(root);
    }

    /// Start a synthetic HGNC publisher behind a local CONNECT proxy.
    pub fn new() -> Result<Self, String> {
        Self::for_source(
            "storage.googleapis.com",
            "text/tab-separated-values",
            |request| body(request.contains("withdrawn.txt")),
        )
    }
    /// Serve synthetic responses for one publisher host.
    pub fn for_source(
        host: &'static str,
        content_type: &'static str,
        reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
    ) -> Result<Self, String> {
        Self::for_sources(&[host], move |_| content_type, reply)
    }
    /// Serve responses with a controlled content length.
    pub fn with_length(
        host: &'static str,
        content_type: impl Fn(&str) -> &'static str + Send + 'static,
        reply: fn(&str) -> Result<String, String>,
        length: fn(&str, usize) -> usize,
    ) -> Result<Self, String> {
        Self::with_status(host, content_type, reply, length, |_| 200)
    }
    /// Serve responses with controlled status and content length.
    pub fn with_status(
        host: &'static str,
        content_type: impl Fn(&str) -> &'static str + Send + 'static,
        reply: fn(&str) -> Result<String, String>,
        length: fn(&str, usize) -> usize,
        status: fn(&str) -> u16,
    ) -> Result<Self, String> {
        Self::with_hosts(&[host], content_type, reply, length, status)
    }
    /// Serve synthetic responses for several publisher hosts.
    pub fn for_sources(
        hosts: &[&str],
        content_type: impl Fn(&str) -> &'static str + Send + 'static,
        reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
    ) -> Result<Self, String> {
        Self::with_hosts(hosts, content_type, reply, |_, bytes| bytes, |_| 200)
    }
    /// Serve one publisher with controlled response status.
    pub fn for_source_status(
        host: &'static str,
        content_type: &'static str,
        reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
        status: impl FnMut(&str) -> u16 + Send + 'static,
    ) -> Result<Self, String> {
        Self::with_hosts(
            &[host],
            move |_| content_type,
            reply,
            |_, bytes| bytes,
            status,
        )
    }
    fn with_hosts(
        hosts: &[&str],
        content_type: impl Fn(&str) -> &'static str + Send + 'static,
        reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
        length: fn(&str, usize) -> usize,
        status: impl FnMut(&str) -> u16 + Send + 'static,
    ) -> Result<Self, String> {
        Self::with_headers(hosts, content_type, reply, length, status, |_| {
            String::new()
        })
    }
    /// Serve CPIC responses with caller-controlled body, range and status.
    pub fn counted(
        reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
        range: impl FnMut(&str) -> String + Send + 'static,
        status: impl FnMut(&str) -> u16 + Send + 'static,
    ) -> Result<Self, String> {
        Self::with_headers(
            &["api.cpicpgx.org"],
            |_| "application/json",
            reply,
            |_, bytes| bytes,
            status,
            range,
        )
    }
    /// Start a CONNECT peer with caller-controlled response headers.
    pub fn with_headers(
        hosts: &[&str],
        content_type: impl Fn(&str) -> &'static str + Send + 'static,
        mut reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
        length: fn(&str, usize) -> usize,
        mut status: impl FnMut(&str) -> u16 + Send + 'static,
        mut range: impl FnMut(&str) -> String + Send + 'static,
    ) -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let (stop, stopping) = mpsc::channel();
        let (handshake, handshakes) = mpsc::channel();
        let (certificate, config) = identity(hosts)?;
        let evidence = evidence::Evidence::new(address, hosts);
        let observed = Arc::clone(&evidence);
        let hosts = hosts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let worker = thread::spawn(move || {
            let mut requests = Vec::new();
            for stream in listener.incoming() {
                if stopping.try_recv().is_ok() {
                    break;
                }
                let mut socket = observed.accept(stream)?;
                let connect = observed.check("CONNECT header", header(&mut socket))?;
                let host = hosts
                    .iter()
                    .find(|host| connect.starts_with(&format!("CONNECT {host}:443 ")))
                    .ok_or("wrong CONNECT destination")?;
                observed.destination(host);
                observed.check(
                    "CONNECT reply",
                    socket.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n"),
                )?;
                let connection =
                    ServerConnection::new(config.clone()).map_err(|e| e.to_string())?;
                let mut tls = StreamOwned::new(connection, socket);
                // An untrusted client closes its handshake before sending HTTP.
                if let Ok(request) = observed.check("TLS request", source_request(&mut tls)) {
                    if let Some(kind) = tls.conn.handshake_kind() {
                        handshake.send(kind).map_err(|e| e.to_string())?;
                    }
                    let body = reply(&request)?;
                    let content_type = content_type(&request);
                    let status = status(&request);
                    observed.check("TLS reply", write!(
                        tls,
                        "HTTP/1.1 {status} Synthetic\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{}Connection: \
                        close\r\n\r\n{body}",
                        length(&request, body.len()),
                        range(&request)
                    ))?;
                    observed.check("TLS flush", tls.flush())?;
                    requests.push((connect, request));
                }
            }
            Ok(requests)
        });
        Ok(Self {
            address,
            certificate,
            stop,
            handshakes,
            evidence,
            worker: Some(worker),
        })
    }
    /// Return completed TLS handshakes observed since the last read.
    pub fn handshakes(&self) -> Vec<rustls::HandshakeKind> {
        self.handshakes.try_iter().collect()
    }
    /// Stop and join the peer, returning all observed requests.
    pub fn finish(mut self) -> Result<Vec<(String, String)>, String> {
        let _ = self.stop.send(());
        let _ = TcpStream::connect(self.address);
        let result = self
            .worker
            .take()
            .ok_or("missing worker")?
            .join()
            .map_err(|_| "TLS worker panicked".to_owned())
            .and_then(|result| result);
        result.map_err(|error| format!("{error}; {}", self.evidence.preserve("worker failed")))
    }
}
impl Drop for Stub {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        let _ = TcpStream::connect(self.address);
        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(Err(_)) | Err(_) => {
                    self.evidence.preserve("worker failed during cleanup");
                }
                Ok(Ok(requests)) if !requests.is_empty() => {
                    self.evidence.preserve("case ended before fixture finish");
                }
                Ok(Ok(_)) => {}
            }
        }
    }
}
fn header(input: &mut impl Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") && bytes.len() < 8192 {
        let mut byte = [0];
        input.read_exact(&mut byte).map_err(|e| e.to_string())?;
        bytes.extend(byte);
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
/// Read the synthetic HGNC response from the public fixtures.
pub fn body(withdrawn: bool) -> Result<String, String> {
    let root = super::fixtures::fixtures().join("replies/gene/main/hgnc");
    for file in fs::read_dir(root).map_err(|e| e.to_string())? {
        let value: Value = serde_json::from_slice(
            &fs::read(file.map_err(|e| e.to_string())?.path()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if value
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| url.ends_with("withdrawn.txt") == withdrawn)
        {
            return value
                .pointer("/exchanges/0/body/value")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or("missing fixture body".into());
        }
    }
    Err("missing HGNC fixture".into())
}

/// Issue a synthetic TLS identity for the given test hosts.
pub fn identity(hosts: &[&str]) -> Result<(String, Arc<ServerConfig>), String> {
    use rcgen::{
        BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, KeyPair,
        KeyUsagePurpose,
    };
    let now = time::OffsetDateTime::now_utc();
    let mut ca = CertificateParams::default();
    // Windows must distinguish the signer from the leaf's default subject name.
    ca.distinguished_name
        .push(rcgen::DnType::CommonName, "Synthetic local test authority");
    ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca.not_before = now - time::Duration::days(1);
    ca.not_after = now + time::Duration::days(2);
    let ca_key = KeyPair::generate().map_err(|e| e.to_string())?;
    let ca = ca.self_signed(&ca_key).map_err(|e| e.to_string())?;
    let mut leaf = CertificateParams::new(hosts.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .map_err(|e| e.to_string())?;
    leaf.not_before = now - time::Duration::days(1);
    leaf.not_after = now + time::Duration::days(1);
    leaf.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().map_err(|e| e.to_string())?;
    let cert = leaf
        .signed_by(&key, &ca, &ca_key)
        .map_err(|e| e.to_string())?;
    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.der().clone()],
                PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
            )
            .map_err(|e| e.to_string())?;
    Ok((ca.pem(), Arc::new(config)))
}

/// Read a bounded HTTP request including its body.
pub fn source_request(input: &mut impl Read) -> Result<String, String> {
    let mut request = header(input)?;
    let length = request
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                .and_then(|(_, v)| v.trim().parse::<usize>().ok())
        })
        .unwrap_or(0);
    if length > 1024 * 1024 {
        return Err("synthetic source request body exceeds bound".into());
    }
    let mut body = vec![0; length];
    input.read_exact(&mut body).map_err(|e| e.to_string())?;
    request.push_str(&String::from_utf8(body).map_err(|e| e.to_string())?);
    Ok(request)
}
