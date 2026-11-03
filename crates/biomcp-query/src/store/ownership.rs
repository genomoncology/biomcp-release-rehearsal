//! Stored source provenance admits caller-owned payloads before reads or updates.
use super::{CacheError, Store};
use biomcp_fetch::{CacheAccess, KeyUse};
use biomcp_types::{KeyTier, Problem, SourceOutcome};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Metadata {
    pub(super) sources: Vec<SourceOutcome>,
    pub(super) problems: Vec<Problem>,
    #[serde(default)]
    pub(super) dropped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ownership: Option<[u8; 32]>,
}

enum Ownership {
    Public,
    Caller([u8; 32]),
    Missing,
}

pub(super) struct Policy(CacheAccess);

impl Store<'_> {
    pub(super) fn policy(&self) -> Policy {
        Policy(self._access.clone())
    }
}

pub(super) enum Table {
    Pool,
    Evidence,
    Snapshot,
}
impl Table {
    fn select(&self) -> &'static str {
        match self {
            Self::Pool => "SELECT outcomes FROM pool WHERE id=?1 AND expires>?2",
            Self::Evidence => "SELECT outcomes FROM citation_evidence WHERE id=?1 AND expires>?2",
            Self::Snapshot => "SELECT outcomes FROM trial_snapshot WHERE id=?1 AND expires>?2",
        }
    }
}
impl Policy {
    pub(super) fn authorize_hydration(
        &self,
        cx: &Connection,
        id: &str,
        outcome: Option<&SourceOutcome>,
    ) -> Result<(), CacheError> {
        if !self.0.is_hosted() {
            return Ok(());
        }
        self.authorize(cx, Table::Pool, id)?;
        let Some(outcome) = outcome else {
            return Ok(());
        };
        let text: String = cx
            .query_row(
                Table::Pool.select(),
                rusqlite::params![id, Store::now()],
                |r| r.get(0),
            )
            .map_err(super::db)?;
        let mut metadata = self.decode(&text)?.ok_or(CacheError::Internal)?;
        metadata.sources.push(outcome.clone());
        let next = self.encode(&metadata.sources, &metadata.problems, metadata.dropped)?;
        self.authorize_update(cx, Table::Pool, id, &next)
    }
    pub(super) fn authorize_update(
        &self,
        cx: &Connection,
        table: Table,
        id: &str,
        next: &str,
    ) -> Result<(), CacheError> {
        if !self.0.is_hosted() {
            return Ok(());
        }
        let text: String = cx
            .query_row(table.select(), rusqlite::params![id, Store::now()], |r| {
                r.get(0)
            })
            .map_err(super::db)?;
        let old = self.decode(&text)?.ok_or(CacheError::Internal)?;
        let new = self.decode(next)?.ok_or(CacheError::Internal)?;
        if self.0.is_hosted() && old.ownership != new.ownership {
            return Err(CacheError::Internal);
        }
        Ok(())
    }
    pub(super) fn authorize(
        &self,
        cx: &Connection,
        table: Table,
        id: &str,
    ) -> Result<(), CacheError> {
        if !self.0.is_hosted() {
            return Ok(());
        }
        let text: Option<String> = cx
            .query_row(table.select(), rusqlite::params![id, Store::now()], |r| {
                r.get(0)
            })
            .optional()
            .map_err(super::db)?;
        if text
            .map(|text| self.decode(&text))
            .transpose()?
            .flatten()
            .is_none()
        {
            return Err(CacheError::Internal);
        }
        Ok(())
    }

    fn ownership(&self, sources: &[SourceOutcome]) -> Ownership {
        if !self.0.is_hosted() {
            return Ownership::Public;
        }
        let names: std::collections::BTreeSet<_> = biomcp_adapters::registry::requests()
            .filter(|spec| sources.iter().any(|source| source.source == spec.source))
            .filter_map(|spec| match spec.key {
                KeyUse::Optional(c) | KeyUse::Required(c) if c.tier == KeyTier::NoFallback => {
                    Some(c.secret)
                }
                _ => None,
            })
            .collect();
        if names.is_empty() {
            return Ownership::Public;
        }
        let mut identity: [u8; 32] = Sha256::digest(b"biomcp/query-ownership/v1\0").into();
        for name in names {
            let mut hash = Sha256::new();
            hash.update(identity);
            hash.update(name.config().as_bytes());
            let Some(scoped) = self.0.namespace(name, hash.finalize().into()) else {
                return Ownership::Missing;
            };
            identity = scoped;
        }
        Ownership::Caller(identity)
    }

    pub(super) fn encode(
        &self,
        sources: &[SourceOutcome],
        problems: &[Problem],
        dropped: bool,
    ) -> Result<String, CacheError> {
        let ownership = match self.ownership(sources) {
            Ownership::Public if problems.is_empty() && !dropped => {
                return serde_json::to_string(sources).map_err(|_| CacheError::Encoding);
            }
            Ownership::Public => None,
            Ownership::Caller(identity) => Some(identity),
            Ownership::Missing => return Err(CacheError::Internal),
        };
        serde_json::to_string(&Metadata {
            sources: sources.into(),
            problems: problems.into(),
            dropped,
            ownership,
        })
        .map_err(|_| CacheError::Encoding)
    }

    pub(super) fn decode(&self, text: &str) -> Result<Option<Metadata>, CacheError> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|_| CacheError::Encoding)?;
        let metadata = if value.is_array() {
            Metadata {
                sources: match super::labels::decode(text)? {
                    Some(sources) => sources,
                    None => return Ok(None),
                },
                problems: Vec::new(),
                dropped: false,
                ownership: None,
            }
        } else {
            match super::labels::decode(text)? {
                Some(metadata) => metadata,
                None => return Ok(None),
            }
        };
        if !self.0.is_hosted() {
            return Ok(Some(metadata));
        }
        let admitted = match self.ownership(&metadata.sources) {
            Ownership::Public => metadata.ownership.is_none(),
            Ownership::Caller(identity) => metadata.ownership == Some(identity),
            Ownership::Missing => false,
        };
        Ok(admitted.then_some(metadata))
    }

    pub(super) fn snapshot_identity(
        &self,
        key: &[u8],
        payload: &str,
        sources: &[SourceOutcome],
    ) -> Result<[u8; 32], CacheError> {
        let identity = Sha256::digest([key, payload.as_bytes()].concat()).into();
        match self.ownership(sources) {
            Ownership::Public => Ok(identity),
            Ownership::Missing => Err(CacheError::Internal),
            Ownership::Caller(scope) => {
                let mut hash = Sha256::new();
                hash.update(b"biomcp/query-snapshot/v1\0");
                hash.update(scope);
                hash.update(identity);
                Ok(hash.finalize().into())
            }
        }
    }
}
