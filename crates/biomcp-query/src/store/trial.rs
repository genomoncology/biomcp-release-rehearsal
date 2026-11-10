//! Source snapshots use the existing store owner, deadlines and source admission.
use super::{CacheError, Store, db};
use biodata::Text;
use biomcp_types::{Source, SourceOutcome};
use rusqlite::{OptionalExtension, params};

type Snapshot = (String, Vec<SourceOutcome>);
pub(crate) const SNAPSHOT_VALIDITY_SECONDS: i64 = 86400;
impl Store<'_> {
    /// Read source dependencies before any saved protocol content.
    #[cfg(test)]
    pub(crate) fn source_snapshot(
        &self,
        id: &str,
        key: &[u8],
        disabled: impl Fn(Source) -> bool,
    ) -> Result<Option<String>, CacheError> {
        self.source_snapshot_with_sources(id, key, disabled)
            .map(|saved| saved.map(|(payload, _)| payload))
    }

    pub(crate) fn source_snapshot_with_sources(
        &self,
        id: &str,
        key: &[u8],
        disabled: impl Fn(Source) -> bool,
    ) -> Result<Option<Snapshot>, CacheError> {
        self.read(|cx| {
            let outcomes: Option<String> = cx
                .query_row(
                    "SELECT outcomes FROM trial_snapshot WHERE id=?1 AND request_key=?2 AND expires>?3",
                    params![id, key, Self::now()],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|error| db(error))?;
            let Some(outcomes) = outcomes else { return Ok(None) };
            let Some(metadata) = self.policy().decode(&outcomes)? else { return Ok(None) };
            if metadata.sources.iter().any(|s| disabled(s.source)) {
                return Ok(None);
            }
            cx.query_row("SELECT payload FROM trial_snapshot WHERE id=?1", params![id], |r| r.get(0))
                .map(|payload| Some((payload, metadata.sources)))
                .map_err(|error| db(error))
        })
    }
    pub(crate) fn save_source_snapshot(
        &mut self,
        key: &[u8],
        payload: &str,
        sources: &[SourceOutcome],
    ) -> Result<Text, CacheError> {
        let outcomes = self.policy().encode(sources, &[], false)?;
        let id = self
            .policy()
            .snapshot_identity(key, payload, sources)?
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let hosted = self._access.is_hosted();
        let policy = self.policy();
        self.write(|tx| {
            Self::collect_expired(tx)?;
            if hosted {
                let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM trial_snapshot WHERE id=?1)", [&id], |r| r.get(0)).map_err(|error| db(error))?;
                if exists {
                    policy.authorize_update(tx, super::ownership::Table::Snapshot, &id, &outcomes)?;
                }
            }
            tx.execute(
                "INSERT INTO trial_snapshot(id,request_key,expires,payload,outcomes) VALUES (?1,?2,?3,?4,?5) ON \
                CONFLICT(id) DO UPDATE SET expires=excluded.expires",
                params![id, key, Self::now().saturating_add(SNAPSHOT_VALIDITY_SECONDS), payload, outcomes],
            )
            .map_err(|error| db(error))?;
            Text::new(&id).map_err(|_| CacheError::Limit)
        })
    }
}
