//! Checked evidence continuation state belongs to the existing per-command store.
use super::{CacheError, Store, db};
use biodata::Text;
use biomcp_types::{Source, SourceOutcome};
use rusqlite::{OptionalExtension, params};
/// Source admission happens before the private snapshot payload is read.
pub(crate) enum EvidenceState {
    Missing,
    Disabled,
    Ready(String),
}
impl Store<'_> {
    pub(crate) fn evidence(
        &self,
        id: &str,
        key: &[u8],
        disabled: impl Fn(Source) -> bool,
    ) -> Result<EvidenceState, CacheError> {
        self.read(|cx| {
            let outcomes: Option<String> = cx
                .query_row(
                    "SELECT outcomes FROM citation_evidence WHERE id=?1 AND request_key=?2 AND expires>?3",
                    params![id, key, Self::now()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| db(error))?;
            let Some(outcomes) = outcomes else { return Ok(EvidenceState::Missing) };
            let Some(metadata) = self.policy().decode(&outcomes)? else { return Ok(EvidenceState::Missing) };
            if metadata.sources.iter().any(|s| disabled(s.source) && s.reason != Some(biomcp_types::Reason::Disabled)) {
                return Ok(EvidenceState::Disabled);
            }
            cx.query_row("SELECT payload FROM citation_evidence WHERE id=?1", params![id], |row| row.get(0))
                .map(EvidenceState::Ready)
                .map_err(|error| db(error))
        })
    }
    pub(crate) fn save_evidence(
        &mut self,
        id: Option<&Text>,
        key: &[u8],
        payload: &str,
        sources: &[SourceOutcome],
    ) -> Result<Text, CacheError> {
        let outcomes = self.policy().encode(sources, &[], false)?;
        let policy = self.policy();
        self.write(|tx| {
            if let Some(id) = id {
                policy.authorize_update(tx, super::ownership::Table::Evidence, id.as_str(), &outcomes)?;
            }
            Self::collect_expired(tx)?;
            let id = match id {
                Some(id) => id.to_string(),
                None => tx.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0)).map_err(|error| db(error))?,
            };
            tx.execute(
                "INSERT INTO citation_evidence(id,request_key,expires,payload,outcomes) VALUES (?1,?2,?3,?4,?5) ON \
                CONFLICT(id) DO UPDATE SET payload=excluded.payload,outcomes=excluded.outcomes",
                params![id, key, Self::now().saturating_add(86400), payload, outcomes],
            )
            .map_err(|error| db(error))?;
            Text::new(&id).map_err(|_| CacheError::Limit)
        })
    }
}
