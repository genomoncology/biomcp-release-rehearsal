//! Copy missing published bundles without modifying their predecessor file.
use crate::{
    CacheError, Deadline, SchemaOwner,
    cache::{
        Cache, check_current_owners, check_predecessor, db_error, sql_deadline, write_transaction,
    },
};
use rusqlite::Connection;
use std::path::Path;
#[derive(Clone, Copy)]
struct Table {
    name: &'static str,
    key: &'static str,
    columns: &'static str,
}
const TABLES: &[Table] = &[
    Table {
        name: "bundle_rejection",
        key: "name",
        columns: "name,rejections",
    },
    Table {
        name: "bundle",
        key: "name",
        columns: "name,bytes,rows,fresh_until,etag,modified",
    },
    Table {
        name: "bundle_row",
        key: "bundle",
        columns: "bundle,n,row",
    },
    Table {
        name: "bundle_key",
        key: "bundle",
        columns: "bundle,kind,key,n",
    },
    Table {
        name: "bundle_words",
        key: "bundle",
        columns: "bundle,n,words",
    },
    Table {
        name: "bundle_content",
        key: "name",
        columns: "name,revision,retrieved_at",
    },
];
impl Cache {
    pub(super) fn import_bundle(
        &self,
        path: &Path,
        name: &str,
        owners: &[SchemaOwner],
        deadline: Deadline<'_>,
    ) -> Result<bool, CacheError> {
        let mut uri = url::Url::from_file_path(path).map_err(|_| CacheError::Limit)?;
        uri.query_pairs_mut().append_pair("mode", "ro");
        let connection = self.owned_connection()?;
        sql_deadline(&connection, deadline)?;
        check_current_owners(&connection, owners)?;
        connection
            .execute("ATTACH DATABASE ?1 AS predecessor", [uri.as_str()])
            .map_err(|error| db_error(error))?;
        let Some(_version) = stage(&connection, name, owners, deadline)? else {
            return Ok(false);
        };
        connection
            .execute_batch("DETACH DATABASE predecessor")
            .map_err(|error| db_error(error))?;
        self.publish_sql(deadline, || {
            sql_deadline(&connection, deadline)?;
            let tx = write_transaction(&connection)?;
            check_current_owners(&tx, owners)?;
            // Current publication wins, including a concurrent publisher that finished while staging.
            if exists(&tx, "main", name)? {
                return Ok(true);
            }
            for table in TABLES {
                sql_deadline(&tx, deadline)?;
                tx.execute(
                    &format!(
                        "INSERT INTO main.{}({}) SELECT {} FROM temp.stage_{}",
                        table.name, table.columns, table.columns, table.name
                    ),
                    [],
                )
                .map_err(|error| db_error(error))?;
            }
            verify(&tx, name, deadline)?;
            tx.commit().map_err(|error| db_error(error))?;
            Ok(true)
        })
    }
}
fn exists(connection: &Connection, schema: &str, name: &str) -> Result<bool, CacheError> {
    connection
        .query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {schema}.bundle WHERE name=?1)"),
            [name],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))
}
fn stage(
    connection: &Connection,
    name: &str,
    owners: &[SchemaOwner],
    deadline: Deadline<'_>,
) -> Result<Option<u32>, CacheError> {
    let snapshot = connection
        .unchecked_transaction()
        .map_err(|error| db_error(error))?;
    check_predecessor(&snapshot, owners)?;
    if !source_table(&snapshot, "bundle")? || !exists(&snapshot, "predecessor", name)? {
        return Ok(None);
    }
    let version = snapshot
        .query_row(
            "SELECT version FROM predecessor.meta WHERE owner='biomcp-fetch'",
            [],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))?;
    let bytes: u32 = snapshot
        .query_row(
            "SELECT bytes FROM predecessor.bundle WHERE name=?1",
            [name],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))?;
    if u64::from(bytes) > super::MAX_BUNDLE_BYTES {
        return Err(CacheError::Limit);
    }
    for table in TABLES {
        stage_table(&snapshot, *table, name, version, deadline)?;
    }
    let complete: bool = snapshot
        .query_row(
            "SELECT (SELECT rows FROM temp.stage_bundle)=(SELECT count(*) FROM temp.stage_bundle_row)",
            [],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))?;
    if !complete {
        return Err(CacheError::Internal);
    }
    snapshot.commit().map_err(|error| db_error(error))?;
    Ok(Some(version))
}
fn source_table(connection: &Connection, table: &str) -> Result<bool, CacheError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM predecessor.sqlite_master WHERE name=?1 AND type='table')",
            [table],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))
}
fn stage_table(
    connection: &Connection,
    table: Table,
    name: &str,
    version: u32,
    deadline: Deadline<'_>,
) -> Result<(), CacheError> {
    let present = source_table(connection, table.name)?;
    if !present
        && !matches!(
            (table.name, version),
            ("bundle_content", 1..=3) | ("bundle_rejection", 1..=4)
        )
    {
        return Err(CacheError::Version);
    }
    let source = if present { "predecessor" } else { "main" };
    connection
        .execute_batch(&format!(
            "CREATE TEMP TABLE stage_{} AS SELECT {} FROM {source}.{} WHERE 0",
            table.name, table.columns, table.name
        ))
        .map_err(|error| db_error(error))?;
    if !present {
        return Ok(());
    }
    sql_deadline(connection, deadline)?;
    let mut select = connection
        .prepare(&format!(
            "SELECT {} FROM predecessor.{} WHERE {}=?1",
            table.columns, table.name, table.key
        ))
        .map_err(|error| db_error(error))?;
    let columns = select.column_count();
    let placeholders = (1..=columns)
        .map(|n| format!("?{n}"))
        .collect::<Vec<_>>()
        .join(",");
    let mut insert = connection
        .prepare(&format!(
            "INSERT INTO temp.stage_{} VALUES ({placeholders})",
            table.name
        ))
        .map_err(|error| db_error(error))?;
    let mut rows = select.query([name]).map_err(|error| db_error(error))?;
    while let Some(row) = rows.next().map_err(|error| db_error(error))? {
        if deadline.expired() {
            return Err(CacheError::Busy);
        }
        let values = (0..columns)
            .map(|n| row.get_ref(n).map(rusqlite::types::ToSqlOutput::Borrowed))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db_error(error))?;
        insert
            .execute(rusqlite::params_from_iter(values))
            .map_err(|error| db_error(error))?;
    }
    Ok(())
}

fn verify(connection: &Connection, name: &str, deadline: Deadline<'_>) -> Result<(), CacheError> {
    for table in TABLES {
        sql_deadline(connection, deadline)?;
        let published = format!(
            "SELECT {},count(*) FROM main.{} WHERE {}=?1 GROUP BY {}",
            table.columns, table.name, table.key, table.columns
        );
        let staged = format!(
            "SELECT {},count(*) FROM temp.stage_{} GROUP BY {}",
            table.columns, table.name, table.columns
        );
        let different: bool = connection
            .query_row(
                &format!("SELECT EXISTS({published} EXCEPT {staged}) OR EXISTS({staged} EXCEPT {published})"),
                [name],
                |r| r.get(0),
            )
            .map_err(|error| db_error(error))?;
        if different {
            return Err(CacheError::Internal);
        }
    }
    Ok(())
}
