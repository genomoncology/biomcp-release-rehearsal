//! Place a body table once, immediately after the first source group that cites it.
use super::{BodyRow, blocks, body, parts::Part, table};
use crate::xml_element::Element;
use biomcp_fetch::BadReply;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn body(
    node: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<(Vec<Part>, Vec<BodyRow>), BadReply> {
    let tables = blocks::descendants(node.0, "table-wrap")?;
    let mut identities = BTreeMap::new();
    for (index, table) in tables.iter().enumerate() {
        if let Some(id) = table.attr("id")?
            && identities.insert(id, index).is_some()
        {
            return Err(BadReply);
        }
    }
    let mut emitted = BTreeSet::new();
    let mut parts = Vec::new();
    let mut rows = body::Rows::default();
    for (name, node) in blocks::body_nodes(node.0)? {
        if matches!(name.as_str(), "table-wrap" | "table-wrap-group") {
            continue;
        }
        let start = parts.len();
        for group in body::groups(&name, node, problems)? {
            let cited: Vec<_> = group
                .table_refs
                .iter()
                .filter_map(|id| identities.get(id.as_str()).copied())
                .collect();
            let path = group.path.clone();
            body::append(&mut parts, group, &mut rows, problems)?;
            for index in cited {
                if emitted.insert(index) {
                    body::append(
                        &mut parts,
                        table::group(*tables.get(index).ok_or(BadReply)?, &path)?,
                        &mut rows,
                        problems,
                    )?;
                }
            }
        }
        if name == "sec" {
            let end = parts.len();
            for (index, part) in parts.iter_mut().enumerate().skip(start) {
                part.continuation = index + 1 < end;
            }
        }
    }
    for (index, table) in tables.into_iter().enumerate() {
        if !emitted.contains(&index) {
            body::append(&mut parts, table::group(table, &[])?, &mut rows, problems)?;
        }
    }
    Ok((parts, rows.values))
}
