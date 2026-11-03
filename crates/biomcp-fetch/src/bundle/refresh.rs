//! Server-owned bundle refreshes retain the ordinary publisher lock and quotas.
use super::{Bundle, Row};
use crate::{BundleRefresh, Leg};
use std::{io::Write, thread::JoinHandle};

impl Leg<'_> {
    pub(super) fn background_read<R: Row>(&self, bundle: &Bundle<R>) -> bool {
        if !matches!(self.fetch.mode, crate::Mode::Live { .. })
            || self.fetch.background_refresh.is_none()
        {
            return false;
        }
        self.report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stale = true;
        self.refresh_in_background(bundle);
        true
    }
    fn refresh_in_background<R: Row>(&self, bundle: &Bundle<R>) {
        let Some(owner) = &self.fetch.background_refresh else {
            return;
        };
        let mut running = owner.running.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(worker) = running.get(bundle.name) {
            if !worker.is_finished() {
                return;
            }
            if let Some(worker) = running.remove(bundle.name) {
                join(bundle.name, worker);
            }
        }
        let mut fetch = self.fetch.clone();
        // Keep caller policy, shared quotas and store, without retaining the
        // owner from its own thread or inheriting a completed call's sockets.
        fetch.background_refresh = None;
        fetch.agents = owner.agents.clone();
        fetch.cancelled = None;
        fetch.call_end = None;
        let (spec, name, path, format, fresh) = (
            bundle.spec,
            bundle.name,
            bundle.path,
            bundle.format,
            bundle.fresh,
        );
        let worker = std::thread::Builder::new()
            .name("bundle-refresh".into())
            .spawn(move || {
                let bundle = Bundle::<R> {
                    spec,
                    name,
                    path,
                    format,
                    fresh,
                    row: std::marker::PhantomData,
                };
                let deadline = fetch.deadline(fetch.config.sync_budget);
                let mut leg = fetch.leg(spec, deadline);
                leg.deadline = deadline;
                let result = leg.bundle_cache().and_then(|cache| {
                    if cache
                        .bundle_info(name, leg.budget())?
                        .is_some_and(|meta| meta.fresh_until > crate::cache::now())
                    {
                        return Ok(());
                    }
                    leg.sync_bundle(&bundle, true)?.published(&leg).map(|_| ())
                });
                if let Err(error) = result {
                    diagnostic(name, error);
                }
            });
        match worker {
            Ok(worker) => {
                running.insert(name, worker);
            }
            Err(_) => diagnostic(name, "worker unavailable"),
        }
    }
}
fn join(name: &str, worker: JoinHandle<()>) {
    if worker.join().is_err() {
        diagnostic(name, "worker stopped");
    }
}
impl Drop for BundleRefresh {
    fn drop(&mut self) {
        let running = self.running.get_mut().unwrap_or_else(|e| e.into_inner());
        for (name, worker) in std::mem::take(running) {
            join(name, worker);
        }
    }
}

fn diagnostic(name: &str, error: impl std::fmt::Display) {
    // Losing stderr cannot replace a caller's already returned stale result.
    writeln!(std::io::stderr().lock(), "bundle refresh {name}: {error}").unwrap_or_default();
}
