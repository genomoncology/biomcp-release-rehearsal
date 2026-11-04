//! Own the lazy runtime, its bounded admission queue and its joined shutdown.
use super::{
    Limits,
    control::{CallGuard, CallState, Stop},
    rpc_call::{self, Event},
};
use crate::{LocalDeadline, Secret};
use biomcp_types::Reason;
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

pub(super) struct Command {
    pub(super) uri: ureq::http::Uri,
    pub(super) path: ureq::http::uri::PathAndQuery,
    pub(super) proxy: Option<ureq::Proxy>,
    pub(super) key: Option<(&'static str, Secret)>,
    pub(super) body: prost::bytes::Bytes,
    pub(super) limits: Limits,
    pub(super) deadline: LocalDeadline,
    pub(super) stop: Arc<Stop>,
    pub(super) events: mpsc::SyncSender<Event>,
    pub(super) done: mpsc::SyncSender<()>,
    pub(super) state: Arc<CallState>,
}
struct Running {
    sender: tokio::sync::mpsc::Sender<Command>,
    thread: thread::JoinHandle<()>,
}
#[expect(
    clippy::disallowed_types,
    reason = "Serialize lazy runtime initialization and join its sole worker thread"
)]
pub(crate) struct Worker {
    running: Mutex<Option<Running>>,
    stop: Arc<Stop>,
    tls: Arc<rustls::ClientConfig>,
    pub(super) limits: Limits,
}
impl Worker {
    #[expect(
        clippy::disallowed_types,
        reason = "Serialize lazy runtime initialization and join its sole worker thread"
    )]
    pub(crate) fn new(
        tls: Arc<rustls::ClientConfig>,
        limits: Limits,
    ) -> Result<Self, crate::FetchError> {
        limits
            .check()
            .map_err(|_| crate::FetchError::StreamingLimits)?;
        Ok(Self {
            running: Mutex::new(None),
            stop: Arc::default(),
            tls,
            limits,
        })
    }
    fn sender(&self) -> Result<tokio::sync::mpsc::Sender<Command>, Reason> {
        let mut running = self.running.lock().map_err(|_| Reason::Unavailable)?;
        if self.stop.stopped() {
            return Err(Reason::Unavailable);
        }
        if running.is_none() {
            let (sender, receiver) = tokio::sync::mpsc::channel(self.limits.queued_calls);
            let stop = self.stop.clone();
            let tls = self.tls.clone();
            let limits = self.limits;
            let thread = thread::Builder::new()
                .name("source-streams".into())
                .spawn(move || runtime(receiver, stop, tls, limits))
                .map_err(|_| Reason::Unavailable)?;
            *running = Some(Running { sender, thread });
        }
        running
            .as_ref()
            .map(|r| r.sender.clone())
            .ok_or(Reason::Unavailable)
    }
    pub(super) fn submit(
        &self,
        deadline: &LocalDeadline,
        make: impl FnOnce(
            mpsc::SyncSender<Event>,
            mpsc::SyncSender<()>,
            Arc<Stop>,
            Arc<CallState>,
        ) -> Command,
    ) -> Result<(mpsc::Receiver<Event>, CallGuard), Reason> {
        let sender = self.sender()?;
        loop {
            if deadline.expired() {
                return Err(Reason::DeadlineExceeded);
            }
            if self.stop.stopped() {
                return Err(Reason::Unavailable);
            }
            match sender.try_reserve() {
                Ok(slot) => {
                    let (events, receiver) = mpsc::sync_channel(1);
                    let (done, completed) = mpsc::sync_channel(1);
                    let stop = Arc::new(Stop::default());
                    let state = Arc::new(CallState::default());
                    slot.send(make(events, done, stop.clone(), state.clone()));
                    return Ok((
                        receiver,
                        CallGuard {
                            stop,
                            done: completed,
                            state,
                        },
                    ));
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                    return Err(Reason::Unavailable);
                }
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    thread::sleep(
                        deadline
                            .end
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(1)),
                    );
                }
            }
        }
    }
    pub(crate) fn shutdown(&self) {
        self.stop.cancel();
        let running = self
            .running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(running) = running {
            // A worker panic closes all response channels; callers receive a typed source failure.
            if running.thread.join().is_err() {
                self.stop.cancel();
            }
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn runtime(
    receiver: tokio::sync::mpsc::Receiver<Command>,
    stop: Arc<Stop>,
    tls: Arc<rustls::ClientConfig>,
    limits: Limits,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build();
    if let Ok(runtime) = runtime {
        let mut tls = (*tls).clone();
        tls.alpn_protocols = vec![b"h2".to_vec()];
        runtime.block_on(run(receiver, stop, Arc::new(tls), limits));
        // Only the documented system resolver may remain on its single blocking slot.
        runtime.shutdown_timeout(Duration::ZERO);
    }
}
async fn run(
    mut receiver: tokio::sync::mpsc::Receiver<Command>,
    stop: Arc<Stop>,
    tls: Arc<rustls::ClientConfig>,
    limits: Limits,
) {
    let mut calls = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = stop.wait() => break,
            Some(_) = calls.join_next(), if !calls.is_empty() => {},
            command = receiver.recv(), if calls.len() < limits.in_flight_streams => {
                let Some(command) = command else { break; };
                calls.spawn(run_call(command, stop.clone(), tls.clone()));
            },
        }
    }
    receiver.close();
    calls.abort_all();
    while calls.join_next().await.is_some() {}
}
async fn run_call(command: Command, stop: Arc<Stop>, tls: Arc<rustls::ClientConfig>) {
    if !command.state.begin() {
        return;
    }
    let result = super::control::bounded(
        &command.deadline,
        &command.stop,
        &stop,
        rpc_call::execute(&command, tls),
    )
    .await;
    // Retain the terminal reason even when the absolute deadline prevents event delivery.
    // Completion remains separate so the caller still waits for network cleanup.
    command.state.record(result);
    tokio::select! {
        _ = command.stop.wait() => {},
        _ = stop.wait() => {},
        _ = super::control::deadline(&command.deadline) => {},
        _ = rpc_call::deliver(&command.events, Event::End(result)) => {},
    }
    command.state.finish();
    if command.done.send(()).is_err() {
        command.stop.cancel();
    }
}
