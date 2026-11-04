//! Event-driven local HTTP server; each observed request owns its response release.
#![expect(
    clippy::unwrap_used,
    reason = "Fixture setup and worker failures must fail the contract test"
)]
use biomcp_fetch::{BadReply, KeyUse, Quota, Reply, SourceSpec};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

#[derive(Debug, Deserialize)]
pub(super) struct Answer {
    pub(super) answer: bool,
}
impl Reply for Answer {
    fn check(&self) -> Result<(), BadReply> {
        if self.answer { Ok(()) } else { Err(BadReply) }
    }
}

pub(super) struct Request<T> {
    pub(super) text: String,
    pub(super) body: Vec<u8>,
    observation: T,
    pub(super) reply: mpsc::Sender<Response>,
}
pub(super) struct Response {
    pub(super) bytes: Vec<u8>,
    pub(super) continuation: Option<mpsc::Receiver<Vec<u8>>>,
}
impl<T> std::ops::Deref for Request<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.observation
    }
}
impl<T> Request<T> {
    pub(super) fn respond(self, status: u16, headers: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status} Stub\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
            body.len()
        );
        self.raw(&response);
    }
    pub(super) fn raw(self, response: &str) {
        self.bytes(response.as_bytes().to_vec());
    }
    pub(super) fn bytes(self, response: Vec<u8>) {
        let _ = self.reply.send(Response {
            bytes: response,
            continuation: None,
        });
    }
    pub(super) fn ok(self) {
        self.respond(200, "", r#"{"answer":true}"#);
    }
}

pub(super) struct Server<T = ()> {
    pub(super) base: &'static str,
    pub(super) requests: mpsc::Receiver<Request<T>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl<T: Default + Send + 'static> Server<T> {
    pub(super) fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let (sent, requests) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut handlers = Vec::new();
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut byte = [0];
                while stream.read(&mut byte).unwrap_or(0) == 1 {
                    bytes.push(byte[0]);
                    assert!(
                        bytes.len() <= 64 * 1024,
                        "request headers exceed fixture limit"
                    );
                    if bytes.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }
                if bytes.is_empty() {
                    break;
                }
                assert!(
                    bytes.len() <= 64 * 1024,
                    "request headers exceed fixture limit"
                );
                let text = String::from_utf8(bytes).unwrap();
                let length = text
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                assert!(
                    length <= 16 * 1024 * 1024,
                    "request body exceeds fixture limit"
                );
                let mut body = vec![0; length];
                stream.read_exact(&mut body).unwrap();
                let (reply, receive) = mpsc::channel();
                if sent
                    .send(Request {
                        text,
                        body,
                        observation: T::default(),
                        reply,
                    })
                    .is_err()
                {
                    break;
                }
                handlers.push(thread::spawn(move || {
                    if let Ok(response) = receive.recv_timeout(Duration::from_secs(10)) {
                        let _ = stream.write_all(&response.bytes);
                        if let Some(continuation) = response.continuation
                            && let Ok(bytes) = continuation.recv_timeout(Duration::from_secs(10))
                        {
                            let _ = stream.write_all(&bytes);
                        }
                    }
                }));
            }
            for handler in handlers {
                handler.join().unwrap();
            }
        });
        Self {
            base,
            requests,
            worker: Some(worker),
        }
    }
    pub(super) fn next(&self) -> Request<T> {
        let request = self.requests.recv_timeout(Duration::from_secs(10)).unwrap();
        if request.text.starts_with("GET ") || request.text.starts_with("HEAD ") {
            assert!(
                request.body.is_empty(),
                "GET and HEAD bodies must stay empty"
            );
        } else {
            assert!(
                request.text.starts_with("POST "),
                "The local server must receive the complete declared request"
            );
        }
        request
    }
    pub(super) fn spec(&self, quota: &'static Quota, key: KeyUse) -> &'static SourceSpec {
        Box::leak(Box::new(SourceSpec {
            destinations: biomcp_fetch::Destinations::Primary,
            user_agent: None,
            terms: biomcp_fetch::Terms {
                name: "Synthetic test data",
                url: "https://example.test/terms",
                usage: biomcp_types::Use::Open,
            },
            source: biomcp_types::Source::PubMed,
            base: self.base,
            key,
            quota,
            deadline: Duration::from_secs(10),
        }))
    }
}
impl<T> Drop for Server<T> {
    fn drop(&mut self) {
        // Drop any unclaimed replies before joining their handlers.
        while self.requests.try_recv().is_ok() {}
        let _ = TcpStream::connect(self.base.trim_start_matches("http://"));
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
