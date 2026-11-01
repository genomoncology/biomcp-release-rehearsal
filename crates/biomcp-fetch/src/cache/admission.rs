//! An explicit schema admission is visible only while it owns the reply connection.
use crate::{CacheError, Deadline};
use rusqlite::Connection;
#[expect(
    clippy::disallowed_types,
    reason = "Shared probes distinguish the current explicit owner of this cache connection"
)]
use std::sync::RwLock;
use std::sync::{MutexGuard, RwLockWriteGuard, TryLockError};
use std::time::Duration;

pub(super) struct Owner<'guard, 'connection> {
    _connection: &'guard MutexGuard<'connection, Connection>,
    _admission: RwLockWriteGuard<'guard, ()>,
}
impl<'guard, 'connection> Owner<'guard, 'connection> {
    #[expect(
        clippy::disallowed_types,
        reason = "Mark only an acquired schema admission, never a queued connection waiter"
    )]
    pub(super) fn enter(
        active: &'guard RwLock<()>,
        connection: &'guard MutexGuard<'connection, Connection>,
        deadline: Deadline<'_>,
    ) -> Result<Self, CacheError> {
        let wall_bound = deadline.local();
        loop {
            if deadline.expired() || wall_bound.expired() {
                return Err(CacheError::Busy);
            }
            match active.try_write() {
                Ok(admission) => {
                    return Ok(Self {
                        _connection: connection,
                        _admission: admission,
                    });
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::park_timeout(Duration::from_millis(1).min(deadline.remaining()));
                }
                Err(TryLockError::Poisoned(_)) => return Err(CacheError::Internal),
            }
        }
    }
}
