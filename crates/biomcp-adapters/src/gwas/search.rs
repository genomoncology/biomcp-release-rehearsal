//! One native association conjunction; no local prefix intersection.
use super::{ASSOCIATIONS, Page, raw};
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Leg, RequestError};
use biomcp_types::GwasSearch;
use std::collections::BTreeMap;
pub(super) mod identity;

const NATIVE_PAGE_SIZE: u32 = 10;

/// Read a bounded p-value-ascending window, sorting locally only for a complete match.
pub fn search(leg: &Leg<'_>, q: &GwasSearch) -> Result<Page, RequestError> {
    if q.limit() <= NATIVE_PAGE_SIZE || q.offset() != 0 {
        return native(leg, q, selectors(q), (q.offset() % q.limit()) as usize);
    }
    // Sorted native pages have unstable ties. Only sort locally after retrieving
    // a complete bounded match; a partial result retains the native global order.
    let mut values = selectors(q);
    values.remove("sort");
    values.remove("direction");
    values.insert("size".into(), NATIVE_PAGE_SIZE.to_string());
    let mut combined = native(leg, q, values.clone(), 0)?;
    if combined.total > u64::from(q.limit()) {
        return native(leg, q, selectors(q), 0);
    }
    let pages: Vec<_> = (1..combined.total.div_ceil(NATIVE_PAGE_SIZE.into())).collect();
    let mut seen: std::collections::BTreeSet<_> =
        combined.rows.iter().map(|row| row.value.id).collect();
    for result in leg.batch(&pages, |child, page| {
        let mut values = values.clone();
        values.insert("page".into(), page.to_string());
        native(child, q, values, 0)
    }) {
        let page = result?;
        if combined.total != page.total {
            return Err(biomcp_fetch::bad_reply(
                biomcp_types::Source::Gwas,
                Some(200),
            ));
        }
        combined.refusals.include(page.refusals);
        combined.problems.extend(page.problems);
        combined.received += page.received;
        for row in page.rows {
            if !seen.insert(row.value.id) {
                return Err(biomcp_fetch::bad_reply(
                    biomcp_types::Source::Gwas,
                    Some(200),
                ));
            }
            combined.rows.push(row);
        }
    }
    if u64::from(combined.received) != combined.total {
        return Err(biomcp_fetch::bad_reply(
            biomcp_types::Source::Gwas,
            Some(200),
        ));
    }
    // Optional metadata and refused identities must retain the original native
    // page's usable rows and diagnostics when exact local ordering is unprovable.
    if combined.rows.len() as u64 != combined.total
        || combined
            .rows
            .iter()
            .any(|row| row.value.exact_p_value.is_none())
    {
        return native(leg, q, selectors(q), 0);
    }
    combined.rows.sort_by(|a, b| {
        a.value
            .exact_p_value
            .cmp(&b.value.exact_p_value)
            .then_with(|| a.value.id.get().cmp(&b.value.id.get()))
    });
    Ok(combined)
}

fn native(
    leg: &Leg<'_>,
    q: &GwasSearch,
    values: BTreeMap<String, String>,
    skip: usize,
) -> Result<Page, RequestError> {
    let params: Vec<_> = values
        .iter()
        .map(|(k, v)| (k.as_str(), Arg::Text(v)))
        .collect();
    let response = leg
        .call(
            &ASSOCIATIONS,
            Input::Params(&params),
            biomcp_fetch::Read::<raw::Response>::json()
                .check(&|reply| admission(reply, q, &values))
                .cache_if(&raw::Response::identities_admitted),
        )?
        .value;
    Ok(super::page(response, skip))
}

fn selectors(q: &GwasSearch) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for (key, value) in [
        ("rs_id", q.variant().map(ToString::to_string)),
        ("efo_trait", q.trait_label().map(ToString::to_string)),
        ("efo_id", q.trait_id().map(ToString::to_string)),
        ("mapped_gene", q.gene().map(ToString::to_string)),
        ("accession_id", q.study().map(ToString::to_string)),
        ("pubmed_id", q.pubmed().map(ToString::to_string)),
    ] {
        if let Some(value) = value {
            values.insert(key.into(), value);
        }
    }
    if q.trait_id().is_some() {
        values.insert(
            "show_child_trait".into(),
            q.include_child_traits().to_string(),
        );
    }
    if q.gene().is_some() {
        values.insert("extended_geneset".into(), q.extended_geneset().to_string());
    }
    values.insert("sort".into(), "p_value".into());
    values.insert("direction".into(), "asc".into());
    values.insert("page".into(), (q.offset() / q.limit()).to_string());
    values.insert("size".into(), q.limit().to_string());
    values
}
fn admission(
    r: &raw::Response,
    q: &GwasSearch,
    values: &BTreeMap<String, String>,
) -> Result<(), BadReply> {
    if Some(r.page.number.to_string()) != values.get("page").cloned()
        || Some(r.page.size.to_string()) != values.get("size").cloned()
        || !identity::self_link(r.links.current.href.as_str(), values)
    {
        return Err(BadReply);
    }
    let rows = r
        .embedded
        .as_ref()
        .map(|e| e.associations.as_slice())
        .unwrap_or_default();
    if !rows.iter().all(|row| identity::row(&row.selectors, q)) {
        return Err(BadReply);
    }
    // Exact parts govern order even when native doubles underflow.
    if !values.contains_key("sort") {
        return Ok(());
    }
    let mut previous = None;
    for row in rows.iter().filter_map(|row| row.value.as_ref()) {
        if let Some(exact) = row.exact_p_value() {
            if previous.as_ref().is_some_and(|last| last > &exact) {
                return Err(BadReply);
            }
            previous = Some(exact);
        }
    }
    Ok(())
}
