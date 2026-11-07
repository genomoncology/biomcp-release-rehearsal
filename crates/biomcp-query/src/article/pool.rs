//! A page pool survives partial answers; only whole answers become search defaults.
use super::page;
use crate::{
    Ctx, Outcome,
    store::{Pool, Store},
};
use biomcp_fetch::{CacheError, RequestError};
use biomcp_types::Status;
use sha2::{Digest, Sha256};

pub(super) struct Keys {
    pub identity: [u8; 32],
    pub current: [u8; 32],
}

pub(super) fn key(cx: &Ctx<'_>, namespace: &[u8], identity: &[u8]) -> Result<Keys, CacheError> {
    let disabled: Vec<_> = cx.fetch.disabled_sources().collect();
    let policy = serde_json::to_vec(&disabled).map_err(|_| CacheError::Encoding)?;
    let mut hash = Sha256::new();
    hash.update(namespace);
    hash.update(identity);
    let identity = hash.clone().finalize().into();
    hash.update([0]);
    hash.update(policy);
    Ok(Keys {
        identity,
        current: hash.finalize().into(),
    })
}

pub(super) fn read<'a>(
    cx: &Ctx<'a>,
    cursor: page::Cursor<'_>,
    store: &mut Store<'a>,
    pool: &Pool,
    out: &mut Outcome,
    current: Option<&[u8]>,
) -> Result<(), RequestError> {
    let result = page::read(cx, cursor, store, pool, out);
    let complete = result.is_ok() && out.status() == Status::Complete;
    let update = match (complete, current) {
        // A new pool is unpublished, so a partial page has no mapping to remove.
        (false, Some(_)) => return result,
        (false, None) => store.unpublish_current(pool),
        (true, Some(key)) => store.publish_current(pool, key),
        (true, None) => return result,
    };
    crate::publish::keep(update.map_err(Into::into), out)?;
    result
}
