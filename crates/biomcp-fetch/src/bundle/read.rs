//! Read admitted bundle rows from the selected store.
use super::{Bundle, BundleInfo, Find, Row, store};
use crate::{Leg, Mode, RequestError};
impl Leg<'_> {
    /// Inspect the admitted publisher dataset date without synchronization or a full row scan.
    pub fn stored_data_date<R: Row>(
        &self,
        bundle: &Bundle<R>,
    ) -> Result<Option<Box<str>>, RequestError> {
        self.bundle_source(bundle)?;
        let cache = self.bundle_cache()?;
        data_date::<R>(cache.bundle_first_row(bundle.name, self.budget())?)
    }
    /// Inspect the selected store metadata without reading rows, importing or synchronizing.
    pub fn stored_bundle_state<R: Row>(
        &self,
        bundle: &Bundle<R>,
    ) -> Result<biomcp_types::LocalBundleState, RequestError> {
        use biomcp_types::LocalBundleState;
        self.bundle_source(bundle)?;
        let Some(cache) = self.fetch.cache.as_deref() else {
            return Ok(LocalBundleState::Missing);
        };
        Ok(match cache.bundle_info(bundle.name, self.budget())? {
            None => LocalBundleState::Missing,
            Some(meta) if meta.fresh_until > crate::cache::now() => LocalBundleState::Fresh,
            Some(_) => LocalBundleState::Stale,
        })
    }

    /// Find checked rows, synchronizing absent or stale data when policy permits.
    pub fn rows<R: Row>(&self, bundle: &Bundle<R>, find: Find<'_>) -> Result<Vec<R>, RequestError> {
        self.bundle_source(bundle)?;
        if self.fetch.cache.is_none() && matches!(self.fetch.mode, Mode::Offline) {
            return Err(self.failure(biomcp_types::Reason::Offline, None).into());
        }
        let cache = self.bundle_cache()?;
        // One command can make several indexed lookups against the same bundle.
        // Fresh revalidates once for this leg, including its concurrent children.
        let mut refreshed = self.fresh.then(|| {
            self.refreshed_bundles
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        let old = if matches!(self.fetch.mode, Mode::Record { .. }) {
            None
        } else {
            self.import_predecessor(cache, bundle.name)?;
            cache.bundle_info(bundle.name, self.budget())?
        };
        let replay = matches!(self.fetch.mode, Mode::Replay { .. });
        let offline = matches!(self.fetch.mode, Mode::Offline);
        let fresh = old
            .as_ref()
            .is_some_and(|m| m.fresh_until > crate::cache::now());
        if offline && old.is_none() {
            return Err(self.failure(biomcp_types::Reason::Offline, None).into());
        }
        let force_fresh = refreshed
            .as_ref()
            .is_some_and(|names| !names.contains(bundle.name));
        let forced = replay || force_fresh || matches!(self.fetch.mode, Mode::Record { .. });
        let background = old.is_some() && !fresh && !forced && self.background_read(bundle);
        if !offline && !background && (!fresh || forced) {
            match self.sync_for_read(bundle) {
                Ok(_) => {
                    if let Some(names) = &mut refreshed {
                        names.insert(bundle.name);
                    }
                }
                Err(RequestError::Source(error))
                    if old.is_some()
                        && matches!(self.fetch.mode, Mode::Live { .. })
                        && !self.fresh =>
                {
                    let mut report = self
                        .report
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    report.stale = true;
                    report.failures.push(error);
                }
                Err(error) => return Err(error),
            }
        }
        if !forced && fresh {
            let url = format!("{}{}", self.spec.base.trim_end_matches('/'), bundle.path);
            self.observed(
                &crate::retry::identity(&url),
                Some(200),
                0,
                crate::Shape::default(),
                std::time::Duration::ZERO,
                ("bundle", true, &url, crate::Provenance::default(), None),
            );
        }
        if offline && !fresh {
            self.report
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stale = true;
        }
        let rows = cache.bundle_rows(bundle.name, find, self.budget())?;
        let rows = decode(rows)?;
        Ok(rows)
    }
    // Live readers share the last complete snapshot. Only an absent snapshot
    // waits for its first publisher, within the existing leg deadline.
    fn sync_for_read<R: Row>(&self, bundle: &Bundle<R>) -> Result<BundleInfo, RequestError> {
        let cache = self.bundle_cache()?;
        let live = matches!(self.fetch.mode, Mode::Live { .. });
        let mut waited = false;
        loop {
            let active = live && cache.syncing(bundle.name, self.budget())?;
            if active && self.fresh {
                self.wait_for_bundle(bundle, waited)?;
                waited = true;
                continue;
            }
            if active || waited {
                if let Some(meta) = cache.bundle_info(bundle.name, self.budget())?
                    && (!self.fresh || meta.fresh_until > crate::cache::now())
                {
                    if meta.fresh_until <= crate::cache::now() {
                        self.report
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .stale = true;
                    }
                    return Ok(store::info(bundle, &meta));
                }
                if active {
                    self.wait_for_bundle(bundle, waited)?;
                    waited = true;
                    continue;
                }
            }
            match self
                .sync_bundle(bundle, true)
                .and_then(|result| result.published(self))
            {
                Err(RequestError::Source(error))
                    if live
                        && error.reason == biomcp_types::Reason::Syncing
                        && cache.syncing(bundle.name, self.budget())? => {}
                result => return result,
            }
        }
    }
    fn wait_for_bundle<R: Row>(
        &self,
        bundle: &Bundle<R>,
        waited: bool,
    ) -> Result<(), RequestError> {
        if !waited && self.fetch.config.observations.is_some() {
            let url = format!("{}{}", self.spec.base.trim_end_matches('/'), bundle.path);
            self.observed(
                &crate::retry::identity(&url),
                None,
                0,
                crate::Shape::default(),
                std::time::Duration::ZERO,
                (
                    "bundle-wait",
                    false,
                    &url,
                    crate::Provenance::default(),
                    None,
                ),
            );
        }
        if crate::retry::wait(std::time::Duration::from_millis(10), self.budget()) {
            Ok(())
        } else {
            Err(self.failure(biomcp_types::Reason::Syncing, None).into())
        }
    }
    /// Read only the last validated local bundle, even when fresh or replay is selected.
    /// A first live reader waits for an active publisher within its deadline.
    /// Missing data without an active publisher still requires explicit synchronization.
    pub fn stored_rows<R: Row>(
        &self,
        bundle: &Bundle<R>,
        find: Find<'_>,
    ) -> Result<(Vec<R>, BundleInfo), RequestError> {
        self.bundle_source(bundle)?;
        let cache = self.bundle_cache()?;
        self.import_predecessor(cache, bundle.name)?;
        let mut snapshot = cache.stored_bundle(bundle.name, find, self.budget())?;
        let mut waiting = false;
        while snapshot.is_none() {
            if !matches!(self.fetch.mode, Mode::Live { .. })
                || !cache.syncing(bundle.name, self.budget())?
            {
                // A publisher may have committed between the snapshot read and
                // the lease check. Read once more before declaring it absent.
                snapshot = cache.stored_bundle(bundle.name, find, self.budget())?;
                break;
            }
            self.wait_for_bundle(bundle, waiting)?;
            waiting = true;
            snapshot = cache.stored_bundle(bundle.name, find, self.budget())?;
        }
        let store::StoredBundle { meta, rows, sample } =
            snapshot.ok_or_else(|| self.failure(biomcp_types::Reason::Syncing, None))?;
        let rows = decode(rows)?;
        if meta.fresh_until <= crate::cache::now() {
            self.report
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stale = true;
        }
        let mut info = store::info(bundle, &meta);
        info.data_date = data_date::<R>(sample)?;
        if let Some(date) = &info.data_date {
            self.data_date(date);
        }
        Ok((rows, info))
    }
    fn import_predecessor(
        &self,
        cache: &crate::cache::Cache,
        name: &str,
    ) -> Result<(), RequestError> {
        if matches!(self.fetch.mode, Mode::Replay { .. })
            || self.fetch.config.replay_cache.is_some()
            || self.fetch.config.cache_predecessors.is_empty()
            || cache.bundle_info(name, self.budget())?.is_some()
        {
            return Ok(());
        }
        for path in &self.fetch.config.cache_predecessors {
            match path.try_exists() {
                Ok(false) => continue,
                Err(_) => return Err(crate::CacheError::Filesystem.into()),
                Ok(true) => {}
            }
            if cache.import_bundle(path, name, &self.fetch.config.cache_owners, self.budget())? {
                break;
            }
        }
        Ok(())
    }
    pub(super) fn bundle_source<R: Row>(&self, bundle: &Bundle<R>) -> Result<(), RequestError> {
        self.check_source()?;
        if self.licensed() {
            return Err(self.failure(biomcp_types::Reason::Rejected, None).into());
        }
        self.spec.admit(
            &format!("{}{}", self.spec.base.trim_end_matches('/'), bundle.path),
            false,
        )?;
        if self.budget().expired() {
            return Err(self
                .failure(biomcp_types::Reason::DeadlineExceeded, None)
                .into());
        }
        if !std::ptr::eq(self.spec, bundle.spec) {
            return Err(self.failure(biomcp_types::Reason::Rejected, None).into());
        }
        Ok(())
    }
    pub(super) fn bundle_cache(&self) -> Result<&crate::cache::Cache, RequestError> {
        self.fetch
            .cache
            .as_deref()
            .ok_or(crate::CacheError::Internal.into())
    }
}

fn decode<R: Row>(rows: Vec<String>) -> Result<Vec<R>, RequestError> {
    rows.into_iter()
        .map(|row| serde_json::from_str(&row).map_err(|_| crate::CacheError::Encoding.into()))
        .collect()
}

fn data_date<R: Row>(row: Option<String>) -> Result<Option<Box<str>>, RequestError> {
    row.map(|row| {
        serde_json::from_str::<R>(&row)
            .map(|row| row.data_date().map(Into::into))
            .map_err(|_| crate::CacheError::Encoding.into())
    })
    .transpose()
    .map(Option::flatten)
}
