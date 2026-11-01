use super::{Bundle, BundleInfo, Find, KeyKind, Row};
use crate::{
    CacheError, Deadline,
    cache::{Cache, db_error, now, write_transaction},
};
use rusqlite::{Connection, OptionalExtension, params};
const PART_PREFIX: &str = ".tmp";
const PART_RANDOM: usize = 6;
const METADATA: &str = "SELECT b.bytes,b.rows,b.fresh_until,b.etag,b.modified,c.revision,c.retrieved_at,\
    COALESCE((SELECT rejections FROM bundle_rejection WHERE name=b.name),'[]') FROM bundle b LEFT \
    JOIN bundle_content c ON c.name=b.name WHERE b.name=?1";
#[derive(Clone, Debug)]
pub(super) struct Meta {
    pub rejections: Vec<super::BundleRejection>,
    pub bytes: u64,
    pub rows: u64,
    pub fresh_until: i64,
    pub etag: Option<String>,
    pub modified: Option<String>,
    pub content_revision: Option<String>,
    pub retrieved_at: Option<i64>,
}
#[derive(Debug)]
pub(super) struct Part {
    pub path: String,
    pub bytes: u64,
    pub etag: Option<String>,
    pub total: Option<u64>,
}
impl Part {
    fn in_use(&self, folder: &std::path::Path) -> Result<bool, CacheError> {
        if !self.validate(folder)? {
            return Ok(false);
        }
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(_) => return Err(CacheError::Filesystem),
        };
        match file.try_lock() {
            Ok(()) => Ok(false),
            Err(std::fs::TryLockError::WouldBlock) => Ok(true),
            Err(std::fs::TryLockError::Error(_)) => Err(CacheError::Filesystem),
        }
    }
    fn validate(&self, folder: &std::path::Path) -> Result<bool, CacheError> {
        let path = std::path::Path::new(&self.path);
        let parent = path.parent().ok_or(CacheError::Limit)?;
        let valid_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(part_name);
        if !valid_name
            || parent.canonicalize().map_err(|_| CacheError::Filesystem)?
                != folder.canonicalize().map_err(|_| CacheError::Filesystem)?
        {
            return Err(CacheError::Limit);
        }
        match path.symlink_metadata() {
            Ok(meta) if meta.is_file() => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            _ => Err(CacheError::Filesystem),
        }
    }
}
pub(crate) fn part_name(name: &str) -> bool {
    name.strip_prefix(PART_PREFIX)
        .is_some_and(|n| n.len() == PART_RANDOM && n.bytes().all(|b| b.is_ascii_alphanumeric()))
}
pub(super) fn create_part(folder: &std::path::Path) -> Result<tempfile::NamedTempFile, CacheError> {
    let file = tempfile::Builder::new()
        .prefix(PART_PREFIX)
        .rand_bytes(PART_RANDOM)
        .tempfile_in(folder)
        .map_err(|_| CacheError::Filesystem)?;
    // The transfer file witnesses this holder's lifetime, including process death.
    file.as_file()
        .try_lock()
        .map_err(|_| CacheError::Filesystem)?;
    Ok(file)
}
pub(crate) fn initialize(c: &Connection) -> Result<(), CacheError> {
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS bundle(name TEXT PRIMARY KEY,bytes INTEGER NOT NULL,rows INTEGER NOT \
        NULL,fresh_until INTEGER NOT NULL,etag TEXT,modified TEXT);
    CREATE TABLE IF NOT EXISTS bundle_sync(name TEXT PRIMARY KEY,holder TEXT,lease_until INTEGER NOT NULL,url \
        TEXT NOT NULL,part TEXT NOT NULL,bytes INTEGER NOT NULL,etag TEXT,total INTEGER);
    CREATE TABLE IF NOT EXISTS bundle_row(bundle TEXT NOT NULL,n INTEGER NOT NULL,row TEXT NOT NULL,PRIMARY KEY(bundle,n));
    CREATE TABLE IF NOT EXISTS bundle_key(bundle TEXT NOT NULL,kind INTEGER NOT NULL,key TEXT NOT NULL,n INTEGER NOT NULL);
    CREATE INDEX IF NOT EXISTS bundle_key_find ON bundle_key(bundle,kind,key,n);
    CREATE VIRTUAL TABLE IF NOT EXISTS bundle_words USING fts5(bundle UNINDEXED,n UNINDEXED,words);
    CREATE TABLE IF NOT EXISTS bundle_rejection(name TEXT PRIMARY KEY,rejections TEXT NOT NULL);
    CREATE TRIGGER IF NOT EXISTS bundle_rejection_delete BEFORE DELETE ON bundle BEGIN DELETE FROM bundle_rejection WHERE name=OLD.name; END;
    CREATE TABLE IF NOT EXISTS bundle_content(name TEXT PRIMARY KEY,revision TEXT NOT NULL,retrieved_at INTEGER NOT NULL);
    CREATE TRIGGER IF NOT EXISTS bundle_content_insert BEFORE INSERT ON bundle BEGIN DELETE FROM \
        bundle_content WHERE name=NEW.name; END;
    CREATE TRIGGER IF NOT EXISTS bundle_content_update BEFORE UPDATE ON bundle BEGIN DELETE FROM \
        bundle_content WHERE name=OLD.name OR name=NEW.name; END;
    CREATE TRIGGER IF NOT EXISTS bundle_content_delete BEFORE DELETE ON bundle BEGIN DELETE FROM \
        bundle_content WHERE name=OLD.name; END;",
    )
    .map_err(db_error)
}
pub(super) fn kind(k: KeyKind) -> u8 {
    match k {
        KeyKind::Id => 0,
        KeyKind::Symbol => 1,
        KeyKind::PreviousSymbol => 2,
        KeyKind::Alias => 3,
        KeyKind::Xref => 4,
        KeyKind::Band => 5,
    }
}
pub(super) fn info<R: Row>(b: &Bundle<R>, m: &Meta) -> BundleInfo {
    BundleInfo {
        rejections: m.rejections.clone(),
        etag: m.etag.as_deref().map(Into::into),
        last_modified: m.modified.as_deref().map(Into::into),
        name: b.name,
        bytes: m.bytes,
        downloaded_bytes: 0,
        rows: m.rows,
        fresh: m.fresh_until > now(),
        content_revision: m.content_revision.as_deref().map(Into::into),
        retrieved_at: m.retrieved_at,
    }
}
impl Cache {
    pub(super) fn bundle_info(
        &self,
        name: &str,
        d: Deadline<'_>,
    ) -> Result<Option<Meta>, CacheError> {
        self.sql(d, |c| {
            c.query_row(METADATA, [name], metadata)
                .optional()
                .map_err(db_error)
        })
    }
    pub(super) fn bundle_rows(
        &self,
        name: &str,
        find: Find<'_>,
        d: Deadline<'_>,
    ) -> Result<Vec<String>, CacheError> {
        self.sql(d, |c| selected_rows(c, name, find))
    }
    pub(super) fn stored_bundle(
        &self,
        name: &str,
        find: Find<'_>,
        d: Deadline<'_>,
    ) -> Result<Option<(Meta, Vec<String>)>, CacheError> {
        self.sql(d, |c| {
            let tx = c.unchecked_transaction().map_err(db_error)?;
            let meta = tx
                .query_row(METADATA, [name], metadata)
                .optional()
                .map_err(db_error)?;
            let value = meta
                .map(|meta| selected_rows(&tx, name, find).map(|rows| (meta, rows)))
                .transpose()?;
            tx.commit().map_err(db_error)?;
            Ok(value)
        })
    }
    pub(super) fn syncing(&self, name: &str, d: Deadline<'_>) -> Result<bool, CacheError> {
        self.sql(d, |c| {
            let part = c.query_row(
                "SELECT part FROM bundle_sync WHERE name=?1 AND holder IS NOT NULL AND lease_until>?2",
                params![name, now()],
                |r| Ok(Part { path: r.get(0)?, bytes: 0, etag: None, total: None }),
            ).optional().map_err(db_error)?;
            part.map(|part| part.in_use(&self.1)).transpose().map(|used| used.unwrap_or(false))
        })
    }
    pub(super) fn claim(
        &self,
        name: &str,
        url: &str,
        holder: &str,
        path: &str,
        lease: i64,
        d: Deadline<'_>,
    ) -> Result<Option<Part>, CacheError> {
        self.write_sql(d, |c| {
            let tx = write_transaction(c)?;
            let old = tx
                .query_row(
                    "SELECT holder,lease_until,url,part,bytes,etag,total FROM bundle_sync WHERE name=?1",
                    [name],
                    |r| {
                        Ok((
                            r.get::<_, Option<String>>(0)?,
                            r.get::<_, i64>(1)?,
                            r.get::<_, String>(2)?,
                            Part {
                                path: r.get(3)?,
                                bytes: u64::from(r.get::<_, u32>(4)?),
                                etag: r.get(5)?,
                                total: r.get::<_, Option<u32>>(6)?.map(u64::from),
                            },
                        ))
                    },
                )
                .optional()
                .map_err(db_error)?;
            if let Some((Some(_), until, _, part)) = &old
                && *until > now()
                && part.in_use(&self.1)?
            {
                return Err(CacheError::Busy);
            }
            // Released and expired files transfer to this claim. A still-running
            // expired holder cannot retain or publish after this transaction.
            let part = old.map(|(h, _, u, mut p)| -> Result<_, CacheError> {
                if !p.validate(&self.1)? {
                    return Ok(None);
                }
                if h.is_some() || u != url {
                    p.bytes = 0;
                }
                Ok(Some(p))
            }).transpose()?.flatten();
            tx.execute(
                "INSERT INTO bundle_sync VALUES(?1,?2,?3,?4,?5,0,NULL,NULL) ON CONFLICT(name) DO UPDATE SET \
                holder=excluded.holder,lease_until=excluded.lease_until,url=excluded.url,part=excluded.part,bytes=0,\
                etag=NULL,total=NULL",
                params![name, holder, lease, url, path],
            )
            .map_err(db_error)?;
            tx.commit().map_err(db_error)?;
            Ok(part)
        })
    }
    pub(super) fn discard(
        &self,
        name: &str,
        holder: &str,
        d: Deadline<'_>,
    ) -> Result<(), CacheError> {
        self.write_sql(d, |c| {
            c.execute(
                "DELETE FROM bundle_sync WHERE name=?1 AND holder=?2",
                params![name, holder],
            )
            .map(|_| ())
            .map_err(db_error)
        })
    }
    pub(super) fn retain(
        &self,
        name: &str,
        holder: &str,
        p: &Part,
        d: Deadline<'_>,
    ) -> Result<bool, CacheError> {
        self.write_sql(d, |c| {
            c.execute(
                "UPDATE bundle_sync SET holder=NULL,lease_until=0,bytes=?3,etag=?4,total=?5 WHERE name=?1 AND \
            holder=?2",
                params![name, holder, p.bytes as i64, p.etag, p.total.map(|n| n as i64)],
            )
            .map(|n| n == 1)
            .map_err(db_error)
        })
    }
}
fn prefix_end(s: &str) -> Option<String> {
    let mut chars: Vec<_> = s.chars().collect();
    while let Some(c) = chars.pop() {
        if let Some(next) = char::from_u32(if c == '\u{d7ff}' {
            0xe000
        } else {
            u32::from(c) + 1
        }) {
            chars.push(next);
            return Some(chars.into_iter().collect());
        }
    }
    None
}

fn selected_rows(c: &Connection, name: &str, find: Find<'_>) -> Result<Vec<String>, CacheError> {
    let (query, a, b, k) = match find {
        Find::AnyExact(k, values) => (
            "SELECT row FROM bundle_row WHERE bundle=?1 AND n IN(SELECT n FROM bundle_key WHERE bundle=?1 AND \
                kind=?4 AND key IN(SELECT value FROM json_each(?2))) ORDER BY n",
            serde_json::to_string(
                &values
                    .iter()
                    .map(|v| v.to_ascii_lowercase())
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| CacheError::Encoding)?,
            None,
            kind(k),
        ),
        Find::Exact(k, v) => (
            "SELECT row FROM bundle_row WHERE bundle=?1 AND n IN(SELECT n FROM bundle_key WHERE bundle=?1 AND kind=?4 AND key=?2) ORDER BY n",
            v.to_ascii_lowercase(),
            None,
            kind(k),
        ),
        Find::Prefix(k, v) => {
            let v = v.to_ascii_lowercase();
            let end = prefix_end(&v);
            (
                "SELECT row FROM bundle_row WHERE bundle=?1 AND n IN(SELECT n FROM bundle_key WHERE bundle=?1 AND \
                    kind=?4 AND key>=?2 AND (?3 IS NULL OR key<?3)) ORDER BY n",
                v,
                end,
                kind(k),
            )
        }
        Find::Words(v) => {
            let words: Vec<_> = v
                .split_whitespace()
                .filter(|w| super::query_word(w))
                .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
                .collect();
            if words.is_empty() {
                return Ok(Vec::new());
            }
            (
                "SELECT row FROM bundle_row WHERE bundle=?1 AND n IN(SELECT n FROM bundle_words WHERE bundle=?1 AND words MATCH ?2) ORDER BY n",
                words.join(" AND "),
                None,
                0,
            )
        }
    };

    let mut q = c.prepare(query).map_err(db_error)?;
    let count = q.parameter_count();
    let mut out = Vec::new();
    let mut rows = if count == 2 {
        q.query(params![name, a])
    } else {
        q.query(params![name, a, b, k])
    }
    .map_err(db_error)?;
    while let Some(row) = rows.next().map_err(db_error)? {
        out.push(row.get(0).map_err(db_error)?);
    }
    Ok(out)
}

fn metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<Meta> {
    Ok(Meta {
        rejections: serde_json::from_str(&row.get::<_, String>(7)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        bytes: u64::from(row.get::<_, u32>(0)?),
        rows: u64::from(row.get::<_, u32>(1)?),
        fresh_until: row.get(2)?,
        etag: row.get(3)?,
        modified: row.get(4)?,
        content_revision: row.get(5)?,
        retrieved_at: row.get(6)?,
    })
}
