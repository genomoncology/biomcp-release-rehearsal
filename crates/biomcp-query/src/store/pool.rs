use super::{CacheError, Store, db};
use biodata::{Pmid, Text};

use biomcp_types::ArticleHeader;
use biomcp_types::{ArticleItem, PoolInfo, Problem, SourceOutcome};
use rusqlite::{OptionalExtension, params};
/// Immutable pool metadata and original provider outcomes.
pub(crate) struct Pool {
    pub info: PoolInfo,
    pub key: Vec<u8>,
    pub sources: Vec<SourceOutcome>,
    pub problems: Vec<Problem>,
    pub dropped: bool,
}
/// Fixed identity and independently hydrated metadata.
pub(crate) struct StoredItem {
    pub item: ArticleItem,
    pub checked: bool,
    pub problems: Vec<Problem>,
    pub outcome: Option<SourceOutcome>,
}
impl Store<'_> {
    pub(crate) fn find(&self, key: &[u8], id: Option<&str>) -> Result<Option<Pool>, CacheError> {
        self.read(|connection| {
            let sql = if id.is_some() {
                "SELECT id,request_key,created,expires,size,outcomes FROM pool WHERE id=?1 AND expires>?2"
            } else {
                "SELECT p.id,p.request_key,p.created,p.expires,p.size,p.outcomes FROM pool p JOIN pool_current c ON \
                c.pool=p.id WHERE c.request_key=?1 AND p.expires>?2"
            };
            let parameter = if let Some(id) = id {
                rusqlite::types::Value::Text(id.into())
            } else {
                rusqlite::types::Value::Blob(key.into())
            };
            let row: Option<(String, Vec<u8>, i64, i64, u32, String)> = connection
                .query_row(sql, params![parameter, Self::now()], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
                })
                .optional()
                .map_err(|error| db(error))?;
            row.map(|(id, key, created, expires, size, sources)| {
                let Some(metadata) = self.policy().decode(&sources)? else { return Ok(None) };
                Ok(Some(Pool {
                    info: PoolInfo {
                        id: Text::new(&id).map_err(|_| CacheError::Limit)?,
                        size,
                        created: u64::try_from(created).map_err(|_| CacheError::Limit)?,
                        expires: u64::try_from(expires).map_err(|_| CacheError::Limit)?,
                    },

                    key,
                    sources: metadata.sources,
                    problems: metadata.problems,
                    dropped: metadata.dropped,
                }))
            })
            .transpose()
            .map(Option::flatten)
        })
    }
    pub(crate) fn publish(
        &mut self,
        key: &[u8],
        items: &[ArticleItem],
        sources: &[SourceOutcome],
        problems: &[Problem],
        dropped: bool,
    ) -> Result<Pool, CacheError> {
        let global: Vec<_> = problems
            .iter()
            .filter(|p| p.record.is_none())
            .cloned()
            .collect();
        let outcomes = self.policy().encode(sources, &global, dropped)?;
        self.write(|tx| {
            Self::collect_expired(tx)?;
            let created = Self::now();
            let expires = created.saturating_add(86400);
            let id: String = tx.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0)).map_err(|error| db(error))?;
            let size = u32::try_from(items.len()).map_err(|_| CacheError::Limit)?;
            tx.execute(
                "INSERT INTO pool(id,request_key,created,expires,size,outcomes) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id, key, created, expires, size, outcomes],
            )
            .map_err(|error| db(error))?;
            {
                let mut insert = tx
                    .prepare("INSERT INTO pool_item(pool,rank,identity,item,header,checked,problems) VALUES (?1,?2,?3,?4,?5,?6,?7)")
                    .map_err(|error| db(error))?;
                for (rank, item) in items.iter().enumerate() {
                    let identity = match &item.key {
                        Some(key) => serde_json::to_string(key).map_err(|_| CacheError::Encoding)?,
                        None => format!("unidentified:{rank}"),
                    };
                    let mut identity_item = item.clone();
                    identity_item.header = None;
                    let payload = serde_json::to_string(&identity_item).map_err(|_| CacheError::Encoding)?;
                    let header = item.header.as_ref().map(serde_json::to_string).transpose().map_err(|_| CacheError::Encoding)?;
                    let own: Vec<_> = problems
                        .iter()
                        .filter(|p| p.record.as_ref().is_some_and(|r| item.pmid.is_some_and(|id| r.as_str() == id.to_string())))
                        .collect();
                    let own = serde_json::to_string(&own).map_err(|_| CacheError::Encoding)?;
                    insert
                        .execute(params![id, rank as u32, identity, payload, header, item.pmid.is_none(), own])
                        .map_err(|error| db(error))?;
                }
            }
            Ok(Pool {
                info: PoolInfo {
                    id: Text::new(&id).map_err(|_| CacheError::Limit)?,
                    size,
                    created: u64::try_from(created).map_err(|_| CacheError::Limit)?,
                    expires: u64::try_from(expires).map_err(|_| CacheError::Limit)?,
                },
                key: key.into(),
                sources: sources.into(),
                problems: global,
                dropped,
            })
        })
    }
    /// Make a checked whole article answer available to new searches.
    pub(crate) fn publish_current(&mut self, pool: &Pool, key: &[u8]) -> Result<(), CacheError> {
        let policy = self.policy();
        self.write(|tx| {
            policy.authorize(tx, super::ownership::Table::Pool, pool.info.id.as_str())?;
            tx.execute(
                "INSERT INTO pool_current(request_key,pool) VALUES (?1,?2) ON CONFLICT(request_key) DO UPDATE SET pool=excluded.pool",
                params![key, pool.info.id.as_str()],
            ).map_err(db)?;
            Ok(())
        })
    }
    /// Remove this pool's reusable answer for one request key, retaining its explicit pages.
    pub(crate) fn unpublish_current(&mut self, pool: &Pool, key: &[u8]) -> Result<(), CacheError> {
        let policy = self.policy();
        self.write(|tx| {
            policy.authorize(tx, super::ownership::Table::Pool, pool.info.id.as_str())?;
            tx.execute(
                "DELETE FROM pool_current WHERE pool=?1 AND request_key=?2",
                params![pool.info.id.as_str(), key],
            )
            .map_err(db)?;
            Ok(())
        })
    }
    /// Admit both source records and read the current window in one SQLite snapshot.
    pub(crate) fn find_source_pool(
        &self,
        key: &[u8],
        id: Option<&str>,
        disabled: impl Fn(biomcp_types::Source) -> bool,
    ) -> Result<Option<(Pool, String)>, CacheError> {
        self.read(|connection| {
            if !connection.is_autocommit() {
                return self.read_source_pool(connection, key, id, disabled);
            }
            let tx = connection.unchecked_transaction().map_err(db)?;
            let found = self.read_source_pool(&tx, key, id, disabled)?;
            tx.commit().map_err(db)?;
            Ok(found)
        })
    }
    fn read_source_pool(
        &self,
        connection: &rusqlite::Connection,
        key: &[u8],
        id: Option<&str>,
        disabled: impl Fn(biomcp_types::Source) -> bool,
    ) -> Result<Option<(Pool, String)>, CacheError> {
        let explicit = id.is_some();
        let selector = if explicit {
            "p.id=?1"
        } else {
            "p.id=(SELECT pool FROM pool_current WHERE request_key=?1)"
        };
        let sql = format!(
            "SELECT p.id,p.request_key,p.created,p.expires,p.size,p.outcomes,s.outcomes \
            FROM pool p JOIN trial_snapshot s ON s.id=p.id AND s.request_key=p.request_key \
            WHERE {selector} AND p.expires>?2 AND s.expires>?2"
        );
        let parameter = id.map_or_else(
            || rusqlite::types::Value::Blob(crate::source_snapshot::current_key(key)),
            |id| rusqlite::types::Value::Text(id.into()),
        );
        let row = connection
            .query_row(&sql, params![parameter, Self::now()], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Vec<u8>>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, u32>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .optional()
            .map_err(db)?;
        let Some((id, key, created, expires, size, outcomes, snapshot)) = row else {
            return Ok(None);
        };
        let Some(metadata) = self.policy().decode(&outcomes)? else {
            return Ok(None);
        };
        let Some(snapshot) = self.policy().decode(&snapshot)? else {
            return Ok(None);
        };
        if metadata
            .sources
            .iter()
            .chain(&snapshot.sources)
            .any(|source| disabled(source.source))
            || !explicit
                && (metadata.dropped
                    || snapshot.dropped
                    || !crate::source_snapshot::sources_whole(&metadata.sources)
                    || !crate::source_snapshot::sources_whole(&snapshot.sources))
        {
            return Ok(None);
        }
        let payload = connection
            .query_row(
                "SELECT payload FROM trial_snapshot WHERE id=?1",
                [&id],
                |r| r.get(0),
            )
            .map_err(db)?;
        Ok(Some((
            Pool {
                info: PoolInfo {
                    id: Text::new(&id).map_err(|_| CacheError::Limit)?,
                    size,
                    created: u64::try_from(created).map_err(|_| CacheError::Limit)?,
                    expires: u64::try_from(expires).map_err(|_| CacheError::Limit)?,
                },
                key,
                sources: metadata.sources,
                problems: metadata.problems,
                dropped: metadata.dropped,
            },
            payload,
        )))
    }
    /// Publish arbitrary source snapshot payload and its fixed-lifetime pool metadata together.
    pub(crate) fn publish_source_pool(
        &mut self,
        key: &[u8],
        payload: &str,
        size: u32,
        sources: &[SourceOutcome],
        incomplete: bool,
    ) -> Result<Pool, CacheError> {
        let outcomes = self.policy().encode(sources, &[], incomplete)?;
        self.write(|tx| {
            Self::collect_expired(tx)?;
            let created = Self::now();
            let expires = created.saturating_add(86400);
            let id: String = tx.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0)).map_err(|error| db(error))?;
            tx.execute(
                "INSERT INTO pool(id,request_key,created,expires,size,outcomes) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id, key, created, expires, size, outcomes],
            )
            .map_err(|error| db(error))?;
            tx.execute(
                "INSERT INTO trial_snapshot(id,request_key,expires,payload,outcomes) VALUES (?1,?2,?3,?4,?5)",
                params![id, key, expires, payload, outcomes],
            )
            .map_err(|error| db(error))?;
            if !incomplete && crate::source_snapshot::sources_whole(sources) {
                tx.execute(
                    "INSERT INTO pool_current(request_key,pool) VALUES (?1,?2) ON CONFLICT(request_key) DO UPDATE SET pool=excluded.pool",
                    params![crate::source_snapshot::current_key(key), id],
                ).map_err(db)?;
            }
            Ok(Pool {
                info: PoolInfo {
                    id: Text::new(&id).map_err(|_| CacheError::Limit)?,
                    size,
                    created: u64::try_from(created).map_err(|_| CacheError::Limit)?,
                    expires: u64::try_from(expires).map_err(|_| CacheError::Limit)?,
                },
                key: key.into(),
                sources: sources.into(),
                problems: Vec::new(),
                dropped: incomplete,
            })
        })
    }
    /// Extend a source window without renewing its published identity or lifetime.
    pub(crate) fn extend_source_pool(
        &mut self,
        mut pool: Pool,
        payload: &str,
        size: u32,
        sources: &[SourceOutcome],
        incomplete: bool,
    ) -> Result<Pool, CacheError> {
        let policy = self.policy();
        let outcomes = policy.encode(sources, &[], incomplete)?;
        self.write(|tx| {
            let id = pool.info.id.as_str();
            policy.authorize_update(tx, super::ownership::Table::Pool, id, &outcomes)?;
            policy.authorize_update(tx, super::ownership::Table::Snapshot, id, &outcomes)?;
            let changed = tx
                .execute(
                    "UPDATE pool SET size=?3,outcomes=?4 WHERE id=?1 AND request_key=?2 AND expires>?5 AND size<=?3",
                    params![id, pool.key, size, outcomes, Self::now()],
                )
                .map_err(|error| db(error))?;
            if changed == 1 {
                let snapshot = tx
                    .execute(
                        "UPDATE trial_snapshot SET payload=?3,outcomes=?4 WHERE id=?1 AND request_key=?2 AND expires>?5",
                        params![id, pool.key, payload, outcomes, Self::now()],
                    )
                    .map_err(|error| db(error))?;
                if snapshot != 1 {
                    return Err(CacheError::Internal);
                }
                if incomplete || !crate::source_snapshot::sources_whole(sources) {
                    tx.execute("DELETE FROM pool_current WHERE request_key=?1 AND pool=?2",
                        params![crate::source_snapshot::current_key(&pool.key), id]).map_err(db)?;
                }
            }
            let (size, outcomes): (u32, String) = tx
                .query_row(
                    "SELECT size,outcomes FROM pool WHERE id=?1 AND request_key=?2 AND expires>?3",
                    params![id, pool.key, Self::now()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|error| db(error))?;
            pool.info.size = size;
            let metadata = policy.decode(&outcomes)?.ok_or(CacheError::Internal)?;
            pool.sources = metadata.sources;
            pool.dropped = metadata.dropped;
            Ok(pool)
        })
    }
    pub(crate) fn page(
        &self,
        pool: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<StoredItem>, CacheError> {
        self.read(|connection| {
            self.policy().authorize(connection, super::ownership::Table::Pool, pool)?;
            let mut statement = connection
                .prepare(
                    "SELECT item,checked,header,problems,outcome FROM pool_item WHERE pool=?1 ORDER BY rank LIMIT ?2 \
                OFFSET ?3",
                )
                .map_err(|error| db(error))?;
            let rows = statement
                .query_map(params![pool, limit, offset], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, bool>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?,
                    ))
                })
                .map_err(|error| db(error))?;
            rows.map(|row| {
                let (id, checked, header, problems, outcome) = row.map_err(|error| db(error))?;
                let mut item: ArticleItem = serde_json::from_str(&id).map_err(|_| CacheError::Encoding)?;
                item.header = header.map(|h| serde_json::from_str(&h).map_err(|_| CacheError::Encoding)).transpose()?;
                Ok(StoredItem {
                    item,
                    checked,
                    problems: serde_json::from_str(&problems).map_err(|_| CacheError::Encoding)?,
                    outcome: outcome.map(|v| super::labels::decode(&v)).transpose()?.flatten(),
                })
            })
            .collect()
        })
    }

    pub(crate) fn rewrite_article_text(
        &mut self,
        pool: &str,
        rows: &[biomcp_types::ArticleItem],
    ) -> Result<(), CacheError> {
        let policy = self.policy();
        self.write(|tx| {
            policy.authorize(tx, super::ownership::Table::Pool, pool)?;
            for row in rows {
                let mut item = row.clone();
                item.header = None;
                let identity = item
                    .key
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|_| CacheError::Encoding)?;
                if let Some(identity) = identity {
                    tx.execute(
                        "UPDATE pool_item SET item=?3 WHERE pool=?1 AND identity=?2",
                        params![
                            pool,
                            identity,
                            serde_json::to_string(&item).map_err(|_| CacheError::Encoding)?
                        ],
                    )
                    .map_err(db)?;
                }
            }
            Ok(())
        })
    }
    pub(crate) fn hydrate(
        &mut self,
        pool: &str,
        ids: &[Pmid],
        headers: &[ArticleHeader],
        problems: &[Problem],
        outcome: Option<&SourceOutcome>,
        dropped: bool,
    ) -> Result<(), CacheError> {
        let policy = self.policy();
        self.write(|tx| {
            policy.authorize_hydration(tx, pool, outcome)?;
            if dropped {
                let raw: String = tx.query_row("SELECT outcomes FROM pool WHERE id=?1", [pool], |r| r.get(0)).map_err(|error| db(error))?;
                let metadata = policy.decode(&raw)?.ok_or(CacheError::Internal)?;
                let raw = policy.encode(&metadata.sources, &metadata.problems, true)?;
                tx.execute("UPDATE pool SET outcomes=?2 WHERE id=?1", params![pool, raw]).map_err(|error| db(error))?;
            }
            let outcome = outcome.map(serde_json::to_string).transpose().map_err(|_| CacheError::Encoding)?;
            let names: Vec<_> = ids.iter().map(ToString::to_string).collect();
            for (index, id) in ids.iter().enumerate() {
                let header = headers
                    .iter()
                    .find(|h| h.pmid == Some(*id))
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|_| CacheError::Encoding)?;
                let name = id.to_string();
                let identity = serde_json::to_string(&biomcp_types::ArticleKey::Pubmed(*id)).map_err(|_| CacheError::Encoding)?;
                let problems: Vec<_> = problems
                    .iter()
                    .filter(|p| {
                        let own = p.record.as_ref().is_some_and(|r| r.as_str() == name);
                        let unmatched = p.record.as_ref().is_none_or(|r| !names.iter().any(|n| r.as_str() == n));
                        own || (index == 0 && unmatched)
                    })
                    .collect();
                let old: String = tx
                    .query_row("SELECT problems FROM pool_item WHERE pool=?1 AND identity=?2", params![pool, identity], |r| {
                        r.get(0)
                    })
                    .map_err(|error| db(error))?;
                let mut merged: Vec<Problem> = serde_json::from_str(&old).map_err(|_| CacheError::Encoding)?;
                merged.retain(|p| p.source != biomcp_types::Source::PubMed);
                for problem in problems {
                    if !merged.contains(problem) {
                        merged.push(problem.clone());
                    }
                }
                let problems = serde_json::to_string(&merged).map_err(|_| CacheError::Encoding)?;
                // A concurrent accepted header always wins over a missing or later response.
                tx.execute(
                    "UPDATE pool_item SET checked=1,header=?3,problems=?4,outcome=?5 WHERE pool=?1 AND identity=?2 AND \
                    header IS NULL",
                    params![pool, identity, header, problems, outcome],
                )
                .map_err(|error| db(error))?;
            }
            Ok(())
        })
    }
}
