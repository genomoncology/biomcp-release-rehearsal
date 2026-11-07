use super::{Calls, Service};
use crate::{command, grammar::Globals};
use serde_json::{Value, json};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
const CALLS: usize = 300;
const SYNCS: usize = 64;
const WINDOW: Duration = Duration::from_secs(5 * 60);
/// A client can reconnect before the server frees its last slot, and health polls also take
/// one. This run measures sync under call load, so admission limits stay out of its way;
/// their refusals have their own tests.
const ADMITTED: usize = crate::config::DEFAULT_HTTP_CAPACITY * 2 + 1;

type Sample = (f64, Result<(), String>);
struct Measurements {
    samples: Vec<Sample>,
    syncs: Vec<Result<u64, String>>,
    health: Vec<Result<u16, String>>,
}
struct Control {
    running: AtomicBool,
    next: AtomicUsize,
    barrier: Barrier,
    started: Instant,
}
#[test]
#[ignore = "Explicit load measurement; run make hosted-http-load"]
fn hosted_http_sync_load() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let (listener, service) = load_service(directory.path())?;
    let initial_rows = sync(&service)?;
    let (stop, stopped) = mpsc::sync_channel(1);
    let server_service = service.clone();
    let server = std::thread::spawn(move || {
        super::super::connections::run(listener, server_service, stopped)
    });
    let control = Control {
        running: AtomicBool::new(true),
        next: AtomicUsize::new(0),
        barrier: Barrier::new(crate::config::DEFAULT_HTTP_CAPACITY + 1),
        started: Instant::now(),
    };
    let measurements = collect(&service, &control);
    let _ = stop.send(());
    server
        .join()
        .map_err(|_| io::Error::other("HTTP load server stopped"))??;
    report(&measurements, initial_rows, control.started)?;
    Ok(())
}
fn load_service(
    directory: &Path,
) -> Result<(TcpListener, Arc<Service>), Box<dyn std::error::Error>> {
    let records = directory.join("records");
    fixtures(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/replies/gene/main"),
        &records,
    )?;
    let config = directory.join("config.json");
    std::fs::write(&config, b"{}")?;
    let globals = Globals {
        config: Some(config),
        hosted: true,
        disable_sources: vec!["oncokb".into()],
        replay: Some(records.clone()),
        ..Globals::default()
    };
    let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        mode: biomcp_fetch::Mode::Replay { dir: records },
        cache_owners: vec![biomcp_query::store::OWNER],
        disabled_sources: [biomcp_types::Source::Oncokb].into_iter().collect(),
        ..Default::default()
    })?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let service = Arc::new(Service {
        hosted_max_use: Default::default(),
        calls: Calls::default(),
        connections: super::super::admission::Slots::new(ADMITTED),
        max_calls: ADMITTED,
        max_connections: ADMITTED,
        address,
        hosts: Vec::new(),
        globals,
        root: super::tests::test_service().root,
        settings: Arc::new(Ok(None)),
        fetch: Ok(Arc::new(fetch)),
        recovery: None,
    });
    Ok((listener, service))
}
fn collect(service: &Service, control: &Control) -> Measurements {
    std::thread::scope(|scope| {
        let sync = scope.spawn(|| sync_worker(service, control));
        let health = scope.spawn(|| health_worker(service, control));
        let workers: Vec<_> = (0..crate::config::DEFAULT_HTTP_CAPACITY)
            .map(|_| scope.spawn(|| call_worker(service.address, control)))
            .collect();
        let samples = workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("load client stopped"))
            .collect();
        control.running.store(false, Ordering::Release);
        Measurements {
            samples,
            syncs: sync.join().expect("sync worker stopped"),
            health: health.join().expect("health worker stopped"),
        }
    })
}
fn sync_worker(service: &Service, control: &Control) -> Vec<Result<u64, String>> {
    control.barrier.wait();
    let mut results = Vec::new();
    while control.running.load(Ordering::Acquire)
        && results.len() < SYNCS
        && control.started.elapsed() < WINDOW
    {
        results.push(sync(service).map_err(|error| error.to_string()));
    }
    results
}
fn health_worker(service: &Service, control: &Control) -> Vec<Result<u16, String>> {
    let mut codes = Vec::new();
    while control.running.load(Ordering::Acquire) && control.started.elapsed() < WINDOW {
        codes.push(
            request(service.address, None)
                .map(|(code, _)| code)
                .map_err(|error| error.to_string()),
        );
        let end = Instant::now() + Duration::from_secs(1);
        while control.running.load(Ordering::Acquire) && Instant::now() < end {
            std::thread::park_timeout(Duration::from_millis(20));
        }
    }
    if control.started.elapsed() >= WINDOW {
        control.running.store(false, Ordering::Release);
        service.calls.cancel();
    }
    codes
}
fn call_worker(address: SocketAddr, control: &Control) -> Vec<Sample> {
    control.barrier.wait();
    let mut samples = Vec::new();
    loop {
        let index = control.next.fetch_add(1, Ordering::AcqRel);
        if index >= CALLS
            || !control.running.load(Ordering::Acquire)
            || control.started.elapsed() >= WINDOW
        {
            break;
        }
        let args = if index.is_multiple_of(2) {
            json!(["search", "gene", "kinase"])
        } else {
            json!(["get", "gene", "BRAF"])
        };
        let body = json!({"jsonrpc":"2.0","id":index,"method":"tools/call",
            "params":{"name":"biomcp","arguments":{"args":args}}})
        .to_string();
        let began = Instant::now();
        let result = request(address, Some(&body)).and_then(|(status, value)| {
            let has_rows = value
                .pointer("/result/structuredContent/result/items")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty());
            if status != 200 || !has_rows || value.pointer("/result/isError") != Some(&json!(false))
            {
                Err(io::Error::other(format!("HTTP {status}: {value}")))
            } else {
                Ok(())
            }
        });
        samples.push((
            began.elapsed().as_secs_f64(),
            result.map_err(|error| error.to_string()),
        ));
    }
    samples
}
fn report(measurements: &Measurements, initial_rows: u64, started: Instant) -> io::Result<()> {
    let mut latencies = measurements
        .samples
        .iter()
        .map(|(elapsed, _)| *elapsed)
        .collect::<Vec<_>>();
    latencies.sort_by(f64::total_cmp);
    let percentile = |percent: usize| {
        latencies
            .get((latencies.len() * percent).div_ceil(100).saturating_sub(1))
            .copied()
    };
    let failures = measurements
        .samples
        .iter()
        .filter_map(|(_, result)| result.as_ref().err())
        .collect::<Vec<_>>();
    let report = json!({"benchmark":"hosted-http-sync", "host":std::env::consts::OS,
    "cache":"cold replay store, then shared warm Fetch with concurrent repeated sync",
    "publisher_delay":"zero; measure local shared cache work",
    "acquisition":"measurement time assigned to copied captures for cache admission",
    "latency_seconds":latencies,
    "workers":crate::config::DEFAULT_HTTP_CAPACITY,"call_limit":CALLS,"window_seconds":WINDOW.as_secs(),
    "elapsed_seconds":started.elapsed().as_secs_f64(),"completed_calls":measurements.samples.len(),
    "p50_seconds":percentile(50),"p95_seconds":percentile(95),"max_seconds":latencies.last(),
    "failed_calls":failures,"initial_sync_rows":initial_rows,
    "sync_completed":measurements.syncs.iter().filter(|result| result.is_ok()).count(),
    "sync_failed":measurements.syncs.iter().filter_map(|result| result.as_ref().err()).collect::<Vec<_>>(),
    "health_statuses":measurements.health});
    let mut output = serde_json::to_vec(&report)?;
    output.push(b'\n');
    crate::output::bytes(&output)?;
    if measurements.samples.is_empty()
        || !failures.is_empty()
        || measurements.syncs.is_empty()
        || measurements.syncs.iter().any(Result::is_err)
        || measurements.health.is_empty()
        || measurements
            .health
            .iter()
            .any(|result| !matches!(result, Ok(200)))
    {
        return Err(io::Error::other(
            "load measurement contains failed or missing work",
        ));
    }
    Ok(())
}
fn sync(service: &Service) -> io::Result<u64> {
    let fetch = service
        .fetch
        .as_ref()
        .map_err(|_| io::Error::other("Fetch unavailable"))?;
    let outcome = command::execute(
        &service.root,
        &service.settings,
        fetch,
        biomcp_query::Action::Sync {
            bundle: biomcp_query::Bundle::named("hgnc")
                .ok_or_else(|| io::Error::other("HGNC missing"))?,
        },
        &service.globals,
        false,
        command::Encoding::Mcp,
    )?;
    if !matches!(outcome.status, crate::exit::Status::Success) {
        return Err(io::Error::other(format!(
            "sync failed: {}",
            outcome.value()?
        )));
    }
    outcome
        .value()?
        .pointer("/result/rows")
        .and_then(Value::as_u64)
        .filter(|rows| *rows > 0)
        .ok_or_else(|| io::Error::other("sync imported no rows"))
}
fn request(address: SocketAddr, body: Option<&str>) -> io::Result<(u16, Value)> {
    let mut socket = TcpStream::connect(address)?;
    socket.set_read_timeout(Some(Duration::from_secs(30)))?;
    let (method, path) = if body.is_some() {
        ("POST", "/mcp")
    } else {
        ("GET", "/health")
    };
    let body = body.unwrap_or_default();
    write!(
        socket,
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\n\
    MCP-Protocol-Version: 2025-11-25\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )?;
    let mut response = String::new();
    if let Err(error) = socket.read_to_string(&mut response)
        && error.kind() != io::ErrorKind::ConnectionReset
    {
        return Err(error);
    }
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("HTTP reply missing"))?;
    let length = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .ok_or_else(|| io::Error::other("HTTP response length missing"))?;
    if body.len() != length {
        return Err(io::Error::other("incomplete HTTP response body"));
    }
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|status| status.parse().ok())
        .ok_or_else(|| io::Error::other("HTTP status missing"))?;
    Ok((status, serde_json::from_str(body)?))
}
fn fixtures(source: &Path, target: &Path) -> io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fixtures(&entry.path(), &destination)?;
        } else {
            let mut value: Value = serde_json::from_slice(&std::fs::read(entry.path())?)?;
            let exchanges = value
                .get_mut("exchanges")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| io::Error::other("recorded exchanges missing"))?;
            for exchange in exchanges.iter_mut() {
                exchange["delay_ms"] = json!(0);
                // Give copied captures a current acquisition time so real cache writes run.
                exchange["fetched"] = json!(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(io::Error::other)?
                        .as_secs()
                );
            }
            let original = exchanges.clone();
            for _ in 0..SYNCS {
                exchanges.extend(original.clone());
            }
            std::fs::write(destination, serde_json::to_vec(&value)?)?;
        }
    }
    Ok(())
}
