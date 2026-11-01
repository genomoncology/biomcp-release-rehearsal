//! Stage checked rows in bounded transactions on an independently owned connection.
use super::{
    Bundle, Row,
    parse::Parsed,
    store::{Meta, kind},
};
use crate::{
    CacheError, Deadline,
    cache::{Cache, db_error, sql_deadline, write_transaction},
};
use rusqlite::{Connection, params};

const BATCH_ROWS: usize = 256;
const TABLES: [&str; 3] = ["bundle_row", "bundle_key", "bundle_words"];
const STAGING: &str = "CREATE TEMP TABLE stage_row(bundle TEXT,n INTEGER,row TEXT);
CREATE TEMP TABLE stage_key(bundle TEXT,kind INTEGER,key TEXT,n INTEGER);
CREATE TEMP TABLE stage_words(bundle TEXT,n INTEGER,words TEXT);";

impl Cache {
    pub(super) fn publish<R: Row>(
        &self,
        bundle: &Bundle<R>,
        holder: Option<&str>,
        meta: &Meta,
        rows: Option<&[Parsed]>,
        deadline: Deadline<'_>,
    ) -> Result<bool, CacheError> {
        let connection = self.owned_connection()?;
        connection.execute_batch(STAGING).map_err(db_error)?;
        if let Some(rows) = rows {
            for (batch, chunk) in rows.chunks(BATCH_ROWS).enumerate() {
                sql_deadline(&connection, deadline)?;
                let tx = connection.unchecked_transaction().map_err(db_error)?;
                insert(&tx, bundle.name, batch * BATCH_ROWS, chunk)?;
                tx.commit().map_err(db_error)?;
            }
        }
        // Reserve the writer before reading ownership. A deferred read snapshot
        // cannot upgrade after a concurrent query owner commits a new WAL frame.
        self.publish_sql(deadline, || {
            sql_deadline(&connection, deadline)?;
            let tx = write_transaction(&connection)?;
            if let Some(holder) = holder {
                let owned: bool = tx
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM bundle_sync WHERE name=?1 AND holder=?2)",
                        params![bundle.name, holder],
                        |row| row.get(0),
                    )
                    .map_err(db_error)?;
                if !owned {
                    return Ok(false);
                }
            }
            if rows.is_some() {
                for table in TABLES {
                    tx.execute(
                        &format!("DELETE FROM {table} WHERE bundle=?1"),
                        [bundle.name],
                    )
                    .map_err(db_error)?;
                    let stage = table.replacen("bundle_", "stage_", 1);
                    tx.execute(
                        &format!("INSERT INTO {table} SELECT * FROM temp.{stage}"),
                        [],
                    )
                    .map_err(db_error)?;
                }
            }
            metadata(&tx, bundle.name, holder, meta)?;
            tx.commit().map_err(db_error)?;
            Ok(true)
        })
    }
}

fn insert(
    connection: &Connection,
    name: &str,
    start: usize,
    rows: &[Parsed],
) -> Result<(), CacheError> {
    for (index, row) in rows.iter().enumerate() {
        let index = (start + index) as i64;
        connection
            .execute(
                "INSERT INTO temp.stage_row VALUES(?1,?2,?3)",
                params![name, index, row.row],
            )
            .map_err(db_error)?;
        for (key, value) in &row.keys.exact {
            connection
                .execute(
                    "INSERT INTO temp.stage_key VALUES(?1,?2,?3,?4)",
                    params![name, kind(*key), value.to_ascii_lowercase(), index],
                )
                .map_err(db_error)?;
        }
        connection
            .execute(
                "INSERT INTO temp.stage_words(bundle,n,words) VALUES(?1,?2,?3)",
                params![name, index, row.keys.words.join(" ")],
            )
            .map_err(db_error)?;
    }
    Ok(())
}

fn metadata(
    connection: &Connection,
    name: &str,
    holder: Option<&str>,
    meta: &Meta,
) -> Result<(), CacheError> {
    connection
        .execute(
            "INSERT INTO bundle VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(name) DO UPDATE SET \
        bytes=excluded.bytes,rows=excluded.rows,fresh_until=excluded.fresh_until,etag=excluded.etag,modified\
        =excluded.modified",
            params![name, meta.bytes as i64, meta.rows as i64, meta.fresh_until, meta.etag, meta.modified],
        )
        .map_err(db_error)?;
    let rejections = serde_json::to_string(&meta.rejections).map_err(|_| CacheError::Encoding)?;
    connection.execute(
        "INSERT INTO bundle_rejection VALUES(?1,?2) ON CONFLICT(name) DO UPDATE SET rejections=excluded.rejections",
        params![name, rejections],
    ).map_err(db_error)?;
    if let (Some(revision), Some(retrieved)) = (&meta.content_revision, meta.retrieved_at) {
        connection
            .execute(
                "INSERT INTO bundle_content VALUES(?1,?2,?3)",
                params![name, revision, retrieved],
            )
            .map_err(db_error)?;
    }
    if let Some(holder) = holder {
        connection
            .execute(
                "DELETE FROM bundle_sync WHERE name=?1 AND holder=?2",
                params![name, holder],
            )
            .map_err(db_error)?;
    }
    Ok(())
}
