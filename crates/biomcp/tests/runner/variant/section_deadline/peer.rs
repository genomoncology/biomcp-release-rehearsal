//! One joined local peer holds two independent provider replies behind request events.
use super::super::super::{launch, network_stub, variant_hotspots};
use rustls::{ServerConnection, StreamOwned};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};
type Tls = StreamOwned<ServerConnection, TcpStream>;
type Events = Vec<&'static str>;
const IDENTITY: &str = concat!(
    "GET /v1/variant/chr7%3Ag.140753336A%3ET?fields=_id%2Cdbsnp.rsid%2Cdbsnp.gene.symbol%2Cclinvar.variant_id%2Csnpeff.ann",
    "%2Cchrom%2Chg19%2Chg38%2Cclingen.caid",
    "&assembly=hg38 HTTP/1.1",
);
const CLINVAR: &str = concat!(
    "GET /entrez/eutils/efetch.fcgi?db=clinvar&id=13961&rettype=vcv&is_variationid=true",
    "&retmode=xml&tool=biomcp HTTP/1.1",
);
const HOTSPOTS: &str = "GET /api/hotspots/single/byGene/BRAF HTTP/1.1";
const HOSTS: &[&str] = &[
    "myvariant.info",
    "eutils.ncbi.nlm.nih.gov",
    "www.cancerhotspots.org",
];
pub(super) struct Peer {
    pub(super) address: SocketAddr,
    pub(super) certificate: String,
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<Result<Events, String>>>,
}
impl Peer {
    pub(super) fn start(directory: &Path) -> Result<Self, String> {
        fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let (certificate, config) = network_stub::identity(HOSTS)?;
        let (stop, stopping) = mpsc::channel();
        let directory = directory.to_owned();
        let worker = thread::spawn(move || serve(listener, stopping, config, &directory));
        Ok(Self {
            address,
            certificate,
            stop,
            worker: Some(worker),
        })
    }
    pub(super) fn finish(mut self) -> Result<Events, String> {
        self.join()
    }
    fn join(&mut self) -> Result<Events, String> {
        let _ = self.stop.send(());
        let _ = TcpStream::connect(self.address);
        self.worker
            .take()
            .ok_or("missing rendezvous worker")?
            .join()
            .map_err(|_| "rendezvous worker panicked")?
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let _ = self.join();
        }
    }
}
fn serve(
    listener: TcpListener,
    stopping: mpsc::Receiver<()>,
    config: std::sync::Arc<rustls::ServerConfig>,
    directory: &Path,
) -> Result<Events, String> {
    let mut events = Vec::new();
    let mut held = Vec::new();
    for socket in listener.incoming() {
        if stopping.try_recv().is_ok() {
            break;
        }
        let (mut tls, host, request) = accept(socket.map_err(|e| e.to_string())?, config.clone())?;
        let (kind, content_type, body) = body(host, &request)?;
        fs::write(
            directory.join(format!("{kind}.json")),
            json!({"host":host,"request":request,"body":body}).to_string(),
        )
        .map_err(|e| e.to_string())?;
        events.push(kind);
        if kind == "identity" {
            reply(&mut tls, content_type, &body)?;
        } else {
            held.push((tls, kind, content_type, body));
            if held.len() == 2 {
                ready(&events)?;
                for (mut tls, kind, content_type, body) in held.drain(..) {
                    match reply(&mut tls, content_type, &body) {
                        Ok(()) => events.push(if kind == "clinvar" {
                            "release-clinvar"
                        } else {
                            "release-hotspots"
                        }),
                        Err(error) => {
                            fs::write(directory.join(format!("{kind}-closed.txt")), error)
                                .map_err(|e| e.to_string())?;
                            events.push(if kind == "clinvar" {
                                "closed-clinvar"
                            } else {
                                "closed-hotspots"
                            });
                        }
                    }
                }
                break;
            }
        }
    }
    fs::write(
        directory.join("events.json"),
        serde_json::to_vec(&events).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(events)
}
fn accept(
    socket: TcpStream,
    config: std::sync::Arc<rustls::ServerConfig>,
) -> Result<(Tls, &'static str, String), String> {
    socket
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| e.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| e.to_string())?;
    let mut socket = socket;
    let connect = network_stub::source_request(&mut socket)?;
    let host = HOSTS
        .iter()
        .copied()
        .find(|host| connect.starts_with(&format!("CONNECT {host}:443 ")))
        .ok_or_else(|| format!("unexpected rendezvous destination: {connect}"))?;
    socket
        .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
        .map_err(|e| e.to_string())?;
    let connection = ServerConnection::new(config).map_err(|e| e.to_string())?;
    let mut tls = StreamOwned::new(connection, socket);
    let request = network_stub::source_request(&mut tls)?;
    Ok((tls, host, request))
}
fn body(host: &str, request: &str) -> Result<(&'static str, &'static str, String), String> {
    let line = request.lines().next().ok_or("missing request line")?;
    match host {
        "myvariant.info" if line == IDENTITY => Ok((
            "identity",
            "application/json",
            variant_hotspots::body(request)?,
        )),
        "www.cancerhotspots.org" if line == HOTSPOTS => Ok((
            "hotspots",
            "application/json",
            variant_hotspots::body(request)?,
        )),
        "eutils.ncbi.nlm.nih.gov" if line == CLINVAR => {
            let record: Value = serde_json::from_str(&launch::fixture_text(
                "replies/variant/assertions/domains/clinvar/6a9b3417762c8df2c211a1274e35df0d0b977ecdeb574ebf4d16d85fd90a2def.json",
            )?).map_err(|e| e.to_string())?;
            let xml = record
                .pointer("/exchanges/0/body/value")
                .and_then(Value::as_str)
                .ok_or("ClinVar body")?;
            Ok(("clinvar", "application/xml", xml.to_owned()))
        }
        _ => Err(format!("unexpected rendezvous request: {host} {request}")),
    }
}
fn reply(tls: &mut Tls, content_type: &str, body: &str) -> Result<(), String> {
    write!(tls, "HTTP/1.1 200 Synthetic\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|e| e.to_string())?;
    tls.flush().map_err(|e| e.to_string())
}
fn ready(events: &[&str]) -> Result<(), String> {
    if events.iter().filter(|e| **e == "clinvar").count() != 1
        || events.iter().filter(|e| **e == "hotspots").count() != 1
    {
        return Err(format!(
            "both independent source requests must precede release: {events:?}"
        ));
    }
    Ok(())
}

pub(super) fn rendezvous(events: &[&str]) -> Result<(), String> {
    ready(events)?;
    let first_release = events
        .iter()
        .position(|e| e.starts_with("release-"))
        .ok_or("missing release event")?;
    ready(events.get(..first_release).ok_or("release ordinal")?)?;
    for expected in ["identity", "release-clinvar", "release-hotspots"] {
        if events.iter().filter(|event| **event == expected).count() != 1 {
            return Err(format!(
                "missing or repeated peer event {expected}: {events:?}"
            ));
        }
    }
    Ok(())
}
