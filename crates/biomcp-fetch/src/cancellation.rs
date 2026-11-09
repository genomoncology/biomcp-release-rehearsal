//! Independent command transports retain shared data while allowing socket shutdown.
use crate::{Fetch, FetchError, Mode, network::Agents};
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};
#[expect(
    clippy::disallowed_types,
    reason = "Cancel all request workers through one command-owned atomic flag"
)]
/// A command's cancellation handle cannot shut down another command's connections.
pub struct Cancellation {
    agents: Arc<Mutex<Vec<Weak<Agents>>>>,
    cancelled: Arc<AtomicBool>,
    end: Option<std::time::Instant>,
}
impl std::fmt::Debug for Cancellation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cancellation").finish_non_exhaustive()
    }
}
impl Cancellation {
    #[expect(
        clippy::disallowed_types,
        reason = "One command registers only its live isolated connection owners"
    )]
    pub(crate) fn new(end: Option<std::time::Instant>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(Vec::new())),
            cancelled: Arc::new(AtomicBool::new(false)),
            end,
        }
    }
    /// Start one cancellable wall-clock scope before transport construction.
    pub fn with_budget(budget: std::time::Duration) -> Self {
        let now = std::time::Instant::now();
        Self::new(Some(now.checked_add(budget).unwrap_or(now)))
    }
    /// Return the wall-clock budget left, independent of any replay clock.
    pub fn remaining(&self) -> std::time::Duration {
        if self.is_cancelled() {
            return std::time::Duration::ZERO;
        }
        self.end.map_or(std::time::Duration::MAX, |end| {
            end.saturating_duration_since(std::time::Instant::now())
        })
    }
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    /// Interrupt active socket work and prevent further requests in this command.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        for agents in self.agents.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            if let Some(agents) = agents.upgrade() {
                agents.shutdown();
            }
        }
    }
}
impl Clone for Cancellation {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            cancelled: self.cancelled.clone(),
            end: self.end,
        }
    }
}
impl Fetch {
    /// Isolate cancellable connections while retaining quotas, cache, recordings and observations.
    pub fn cancellable(&self) -> Result<(Self, Cancellation), FetchError> {
        let cancellation = Cancellation::new(None);
        let fetch = self.with_cancellation(&cancellation)?;
        Ok((fetch, cancellation))
    }
    /// Bind this transport's isolated connections to an existing command scope.
    pub fn with_cancellation(&self, cancellation: &Cancellation) -> Result<Self, FetchError> {
        let agents = Arc::new(Agents::new(
            &self.config.network,
            self.config.proxy.as_ref(),
            self.tls.clone(),
        )?);
        let mut owners = cancellation
            .agents
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        owners.retain(|owner| owner.strong_count() > 0);
        owners.push(Arc::downgrade(&agents));
        if cancellation.cancelled.load(Ordering::Acquire) {
            agents.shutdown();
        }
        let mut fetch = self.clone();
        fetch.agents = agents;
        fetch.cancelled = Some(cancellation.cancelled.clone());
        fetch.call_end = cancellation.end;
        Ok(fetch)
    }
    /// Restrict this command to stored replies while retaining its shared observations sink.
    pub fn offline(&self) -> Self {
        let mut fetch = self.clone();
        fetch.mode = Mode::Offline;
        fetch
    }
}
