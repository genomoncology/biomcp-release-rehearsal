use super::raw::{Catalog, Native, WireNative};
use crate::mapped::{Problems, record};
use biomcp_fetch::{
    BadReply, KeyKind, Keys, Row, bundle_sha256, decode_bundle_document, encode_bundle_row,
};
use biomcp_types::{Problem, Source, WikiPathwaysCatalogReceipt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CatalogRow {
    pub native: Option<Native>,
    pub receipt: WikiPathwaysCatalogReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<Problem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<bool>,
}
fn mapped(row: WireNative, p: &mut Problems) -> Option<Native> {
    if !biodata::Text::new(&row.species).is_ok_and(|s| s.as_str() == row.species.as_ref())
        || row.url.as_str() != format!("https://www.wikipathways.org/instance/{}", row.id)
        || row.datanodes.is_some() != row.annotations.is_some()
    {
        return p.required("identity", None);
    }
    Some(Native {
        id: row.id,
        url: row.url,
        species: row.species,
        name: p.source_field("name", row.name),
        revision: p.source_field("revision", row.revision),
        authors: p.field("authors", row.authors),
        description: p.field("description", row.description),
        cited_in: p.field("citedIn", row.cited_in),
        datanodes: p.field("datanodes", row.datanodes),
        annotations: p.field("annotations", row.annotations),
    })
}
impl Row for CatalogRow {
    const MAX_BYTES: u64 = 8 * 1024 * 1024;
    fn decode_document(body: &[u8]) -> Result<Vec<String>, BadReply> {
        if body.len() as u64 > Self::MAX_BYTES {
            return Err(BadReply);
        }
        let catalog: Catalog = decode_bundle_document(body)?;
        if catalog.rows.is_empty() {
            return Err(BadReply);
        }
        let (mut ids, mut duplicates) = (BTreeSet::new(), BTreeSet::new());
        for row in &catalog.rows {
            if let Some(row) = row.as_ref()
                && !ids.insert(row.id.to_string())
            {
                duplicates.insert(row.id.to_string());
            }
        }
        let sha256 = bundle_sha256(body);
        let receipt = WikiPathwaysCatalogReceipt {
            sha256: sha256.parse().map_err(|_| BadReply)?,
            bytes: body.len() as u64,
            revision: format!("sha256:{sha256}").parse().map_err(|_| BadReply)?,
            human_rows: catalog
                .rows
                .iter()
                .filter(|r| {
                    r.as_ref()
                        .is_some_and(|r| r.species.as_ref() == "Homo sapiens")
                })
                .count() as u64,
            total_rows: catalog.rows.len() as u64,
        };
        catalog
            .rows
            .into_iter()
            .map(|row| {
                let mut p = Problems {
                    source: Source::WikiPathways,
                    record: None,
                    list: Vec::new(),
                };
                let row = p.required_field("identity", Some(row));
                let search = row.as_ref().map(|r| r.datanodes.is_some());
                p.record = row.as_ref().and_then(|r| record(&r.id));
                let native = row.and_then(|row| {
                    if duplicates.contains(&row.id.to_string()) {
                        p.required("identity", None)
                    } else {
                        mapped(row, &mut p)
                    }
                });
                encode_bundle_row(&Self {
                    native,
                    receipt: receipt.clone(),
                    problems: Some(p.list),
                    search,
                })
            })
            .collect()
    }
    fn keys(&self) -> Keys {
        let mut exact = vec![(KeyKind::Band, "catalog".into())];
        if let Some(native) = &self.native {
            exact.push((KeyKind::Id, native.id.to_string().into()));
        }
        Keys {
            exact,
            words: vec![],
        }
    }
    fn check_part(&self, part: &str) -> Result<(), BadReply> {
        let search = self
            .search
            .or_else(|| self.native.as_ref().map(|r| r.datanodes.is_some()));
        if search.is_none() || search == Some(part == "search") && matches!(part, "search" | "get")
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
