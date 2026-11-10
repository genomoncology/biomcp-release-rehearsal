//! Hosted tasks verify a cursor with no dependency on the issuing task's store.
use crate::{
    Ctx, source_snapshot,
    store::{SNAPSHOT_VALIDITY_SECONDS, Store, snapshot_scope},
};
use biodata::Text;
use biomcp_fetch::{CacheError, RequestError, TrialCursorKey};
use biomcp_types::{Source, TrialSearch};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(super) const PREFIX: &str = "biomcp:ctgov-page:v2:";
const NAMESPACE: &str = "ctgov-continuation/2";

type Restored = (TrialSearch, Vec<Source>);

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    // The native query already carries its broader-matching marker when needed.
    native: Text,
    expires: i64,
    sources: Vec<Source>,
    scope: Option<[u8; 32]>,
}

pub(super) fn retain(
    cx: &Ctx<'_>,
    key: &TrialCursorKey,
    query: &TrialSearch,
    sources: &[Source],
) -> Result<Text, RequestError> {
    let native = query.next_page().ok_or(CacheError::Encoding)?.clone();
    let scope = snapshot_scope(cx.fetch.cache_access()?, sources)?;
    let payload = serde_json::to_vec(&Cursor {
        native,
        expires: Store::now().saturating_add(SNAPSHOT_VALIDITY_SECONDS),
        sources: sources.to_vec(),
        scope,
    })
    .map_err(|_| CacheError::Encoding)?;
    let envelope = key.seal(payload, &binding(query)?)?;
    Ok(Text::new(&format!("{PREFIX}{envelope}")).map_err(|_| CacheError::Encoding)?)
}

pub(super) fn restore(
    cx: &Ctx<'_>,
    query: &TrialSearch,
    envelope: &str,
) -> Result<Option<Restored>, RequestError> {
    let Some(key) = cx.fetch.trial_cursor_key() else {
        return Ok(None);
    };
    let Some(payload) = key.open(envelope, &binding(query)?) else {
        return Ok(None);
    };
    let Ok(cursor) = serde_json::from_slice::<Cursor>(&payload) else {
        return Ok(None);
    };
    if cursor.expires <= Store::now()
        || cx.fetch.disabled(Source::ClinicalTrialsGov)
        || cursor
            .sources
            .iter()
            .any(|source| cx.fetch.disabled(*source))
        || !snapshot_scope(cx.fetch.cache_access()?, &cursor.sources)
            .is_ok_and(|scope| scope == cursor.scope)
    {
        return Ok(None);
    }
    Ok(Some((
        query.at_page(cursor.native, query.offset()),
        cursor.sources,
    )))
}

// Authenticate the public selectors without putting a second copy in the token.
fn binding(query: &TrialSearch) -> Result<[u8; 32], RequestError> {
    let selectors = source_snapshot::key(NAMESPACE, &query.first_page())?;
    let mut hash = Sha256::new();
    hash.update(selectors);
    hash.update(query.offset().to_be_bytes());
    Ok(hash.finalize().into())
}
